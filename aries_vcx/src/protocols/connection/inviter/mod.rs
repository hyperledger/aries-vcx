pub mod states;

use std::sync::Arc;

use ::uuid::Uuid;
use chrono::Utc;
use messages2::decorators::thread::Thread;
use messages2::decorators::timing::Timing;
use messages2::msg_fields::protocols::connection::invitation::{
    Invitation, PairwiseInvitation, PairwiseInvitationContent, PwInvitationDecorators,
};
use messages2::msg_fields::protocols::connection::request::Request;
use messages2::msg_fields::protocols::connection::response::{Response, ResponseContent, ResponseDecorators};
use messages2::AriesMessage;
use url::Url;

use crate::handlers::util::{verify_thread_id, AnyInvitation};
use crate::protocols::connection::trait_bounds::ThreadId;
use crate::transport::Transport;
use crate::{common::signing::sign_connection_response, errors::error::VcxResult};

use self::states::{
    completed::Completed, initial::Initial, invited::Invited, requested::Requested, responded::Responded,
};
use super::{initiation_type::Inviter, pairwise_info::PairwiseInfo, Connection};
use aries_vcx_core::wallet::base_wallet::BaseWallet;

pub type InviterConnection<S> = Connection<Inviter, S>;

impl InviterConnection<Initial> {
    /// Creates a new [`InviterConnection<Initial>`].
    ///
    /// The connection can transition to [`InviterConnection<Invited>`] by
    /// either `create_invitation` or `into_invited`.
    pub fn new_inviter(source_id: String, pairwise_info: PairwiseInfo) -> Self {
        Self {
            source_id,
            state: Initial,
            pairwise_info,
            initiation_type: Inviter,
        }
    }

    /// Generates a pairwise [`Invitation`] and transitions to [`InviterConnection<Invited>`].
    pub fn create_invitation(self, routing_keys: Vec<String>, service_endpoint: Url) -> InviterConnection<Invited> {
        let id = Uuid::new_v4().to_string();
        let content = PairwiseInvitationContent::new(
            self.source_id.clone(),
            vec![self.pairwise_info.pw_vk.clone()],
            routing_keys,
            service_endpoint,
        );

        let decorators = PwInvitationDecorators::default();

        let invite = PairwiseInvitation::with_decorators(id, content, decorators);

        let invitation = AnyInvitation::Con(Invitation::Pairwise(invite));

        Connection {
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: self.initiation_type,
            state: Invited::new(invitation),
        }
    }

    /// This is implemented for retro-fitting the previous implementation
    /// where a [`Request`] could get processed directly from the initial state.
    ///
    /// If you want to generate a new inviter and not create an invitation through
    /// [`InviterConnection<Initial>::create_invitation`] then you can call this
    /// to transition to the [`InviterConnection<Invited>`] directly by passing the
    /// expected thread_id (external's [`Invitation`] id).
    ///
    /// However, the advised method of handling connection requests is to clone
    /// the [`InviterConnection<Invited>`] and continue the protocol for every
    /// [`Request`] received for the generated invitation, assuming more than one
    /// invitees are expected.
    //
    // This is a workaround and it's not necessarily pretty, but is implemented
    // for backwards compatibility.
    pub fn into_invited(self, thread_id: &str) -> InviterConnection<Invited> {
        let id = Uuid::new_v4().to_string();
        let content = PairwiseInvitationContent::new(
            self.source_id.clone(),
            vec![self.pairwise_info.pw_vk.clone()],
            vec![],
            "https:://dummy.dummy/dummy".parse().expect("url should be valid"),
        );

        let decorators = PwInvitationDecorators::default();

        let invite = PairwiseInvitation::with_decorators(id, content, decorators);

        let invitation = AnyInvitation::Con(Invitation::Pairwise(invite));

        Connection {
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: self.initiation_type,
            state: Invited::new(invitation),
        }
    }
}

impl InviterConnection<Invited> {
    // This should ideally belong in the Connection<Inviter, RequestedState>
    // but was placed here to retro-fit the previous API.
    async fn build_response_content(
        &self,
        wallet: &Arc<dyn BaseWallet>,
        request: &Request,
        new_pairwise_info: &PairwiseInfo,
    ) -> VcxResult<Response> {
        let new_recipient_keys = vec![new_pairwise_info.pw_vk.clone()];

        let id = Uuid::new_v4().to_string();

        let con_sig = sign_connection_response(wallet, &self.pairwise_info.pw_vk, &request.content.connection).await?;

        let content = ResponseContent::new(con_sig);

        let thread_id = request
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(request.id.as_str());

        let mut decorators = ResponseDecorators::new(Thread::new(thread_id.to_owned()));
        let mut timing = Timing::default();
        timing.out_time = Some(Utc::now());
        decorators.timing = Some(timing);

        Ok(Response::with_decorators(id, content, decorators))
    }

    /// Processes a [`Request`] and transitions to [`InviterConnection<Requested>`].
    ///
    /// # Errors
    ///
    /// Will return an error if either:
    ///     * the [`Request`]'s thread ID does not match with the expected thread ID from an invitation
    ///     * the [`Request`]'s DidDoc is not valid
    ///     * generating new [`PairwiseInfo`] fails
    pub async fn handle_request<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        mut request: Request,
        new_service_endpoint: Url,
        new_routing_keys: Vec<String>,
        transport: &T,
    ) -> VcxResult<InviterConnection<Requested>>
    where
        T: Transport,
    {
        trace!(
            "Connection::process_request >>> request: {:?}, service_endpoint: {}, routing_keys: {:?}",
            request,
            new_service_endpoint,
            new_routing_keys,
        );

        // There must be some other way to validate the thread ID other than cloning the entire Request
        verify_thread_id(self.thread_id(), &request.clone().into());

        // If the request's DidDoc validation fails, we generate and send a ProblemReport.
        // We then return early with the provided error.
        if let Err(err) = request.content.connection.did_doc.validate() {
            error!("Request DidDoc validation failed! Sending ProblemReport...");

            self.send_problem_report(
                wallet,
                &err,
                &request
                    .decorators
                    .thread
                    .as_ref()
                    .map(|t| t.thid.as_str())
                    .unwrap_or(request.id.as_str()),
                &request.content.connection.did_doc,
                transport,
            )
            .await;

            Err(err)?;
        }

        // Generate new pairwise info that will be used from this point on
        // and incorporate that into the response.
        let new_pairwise_info = PairwiseInfo::create(wallet).await?;
        let did_doc = request.content.connection.did_doc.clone();

        request
            .content
            .connection
            .did_doc
            .set_service_endpoint(new_service_endpoint);
        request.content.connection.did_doc.set_routing_keys(new_routing_keys);
        let content = self
            .build_response_content(wallet, &request, &new_pairwise_info)
            .await?;

        let state = Requested::new(content, did_doc);

        Ok(Connection {
            source_id: self.source_id,
            pairwise_info: new_pairwise_info,
            initiation_type: self.initiation_type,
            state,
        })
    }

    /// Returns the [`Invitation`] generated by this inviter.
    ///
    /// NOTE: Calling [`InviterConnection<Initial>::into_invited()`]
    /// creates a dummy invitation behind the scenes.
    ///
    /// So this method will return garbage in that case.
    /// `into_invited` is implemented for backwards compatibility
    /// and should be avoided when possible.
    pub fn get_invitation(&self) -> &AnyInvitation {
        &self.state.invitation
    }
}

impl InviterConnection<Requested> {
    /// Sends a [`Response`] to the invitee and transitions to [`InviterConnection<Responded>`].
    ///
    /// # Errors
    ///
    /// Will return an error if sending the response fails.
    pub async fn send_response<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        transport: &T,
    ) -> VcxResult<InviterConnection<Responded>>
    where
        T: Transport,
    {
        trace!(
            "Connection::send_response >>> signed_response: {:?}",
            &self.state.signed_response
        );

        let thread_id = self.state.signed_response.decorators.thread.thid.clone();

        self.send_message(wallet, &self.state.signed_response.clone().into(), transport)
            .await?;

        let state = Responded::new(self.state.did_doc, thread_id);

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: self.initiation_type,
        })
    }
}

impl InviterConnection<Responded> {
    /// Acknowledges an invitee's connection by processing their first message
    /// and transitions to [`InviterConnection<Completed>`].
    ///
    /// # Errors
    ///
    /// Will error out if the message's thread ID does not match
    /// the ID of the thread context used in this connection.
    pub fn acknowledge_connection(self, msg: &AriesMessage) -> VcxResult<InviterConnection<Completed>> {
        verify_thread_id(self.state.thread_id(), msg)?;
        let state = Completed::new(self.state.did_doc, self.state.thread_id, None);

        Ok(Connection {
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: self.initiation_type,
            state,
        })
    }
}
