pub mod states;

use std::sync::Arc;

use messages::{
    a2a::A2AMessage,
    protocols::connection::{
        invite::{Invitation, PairwiseInvitation},
        request::Request,
        response::{Response, SignedResponse},
    },
};

use self::states::{
    completed::Completed, initial::Initial, invited::Invited, requested::Requested, responded::Responded,
};
use super::{initiation_type::Inviter, pairwise_info::PairwiseInfo, Connection};
use crate::{
    common::signing::sign_connection_response, errors::error::VcxResult, handlers::util::verify_thread_id,
    plugins::wallet::base_wallet::BaseWallet, protocols::connection::trait_bounds::ThreadId, transport::Transport,
    utils::uuid,
};

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
    pub fn create_invitation(self, routing_keys: Vec<String>, service_endpoint: String) -> InviterConnection<Invited> {
        let invite = PairwiseInvitation::create()
            .set_id(&uuid::uuid())
            .set_label(&self.source_id)
            .set_recipient_keys(vec![self.pairwise_info.pw_vk.clone()])
            .set_routing_keys(routing_keys)
            .set_service_endpoint(service_endpoint);

        let invitation = Invitation::Pairwise(invite);

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
        let invite = PairwiseInvitation::create().set_id(thread_id);
        let invitation = Invitation::Pairwise(invite);

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
    async fn build_response(
        &self,
        wallet: &Arc<dyn BaseWallet>,
        request: &Request,
        new_pairwise_info: &PairwiseInfo,
        new_service_endpoint: String,
        new_routing_keys: Vec<String>,
    ) -> VcxResult<SignedResponse> {
        let new_recipient_keys = vec![new_pairwise_info.pw_vk.clone()];
        let response = Response::create()
            .set_did(new_pairwise_info.pw_did.to_string())
            .set_service_endpoint(new_service_endpoint)
            .set_keys(new_recipient_keys, new_routing_keys)
            .ask_for_ack()
            .set_thread_id(&request.get_thread_id())
            .set_out_time();

        sign_connection_response(wallet, &self.pairwise_info.pw_vk, response).await
    }

    /// Processes a [`Request`] and transitions to [`InviterConnection<Requested>`].
    ///
    /// # Errors
    ///
    /// Will return an error if either:
    ///     * the [`Request`]'s thread ID does not match with the expected thread ID from an
    ///       invitation
    ///     * the [`Request`]'s DidDoc is not valid
    ///     * generating new [`PairwiseInfo`] fails
    pub async fn handle_request<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        request: Request,
        new_service_endpoint: String,
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
        verify_thread_id(self.state.thread_id(), &A2AMessage::ConnectionRequest(request.clone()))?;

        // If the request's DidDoc validation fails, we generate and send a ProblemReport.
        // We then return early with the provided error.
        if let Err(err) = request.connection.did_doc.validate() {
            error!("Request DidDoc validation failed! Sending ProblemReport...");

            self.send_problem_report(
                wallet,
                &err,
                &request.get_thread_id(),
                &request.connection.did_doc,
                transport,
            )
            .await;

            Err(err)?;
        }

        // Generate new pairwise info that will be used from this point on
        // and incorporate that into the response.
        let new_pairwise_info = PairwiseInfo::create(wallet).await?;
        let signed_response = self
            .build_response(
                wallet,
                &request,
                &new_pairwise_info,
                new_service_endpoint,
                new_routing_keys,
            )
            .await?;

        let did_doc = request.connection.did_doc;
        let state = Requested::new(signed_response, did_doc);

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
    pub fn get_invitation(&self) -> &Invitation {
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

        let thread_id = self.state.signed_response.get_thread_id();

        self.send_message(wallet, &self.state.signed_response.to_a2a_message(), transport)
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
    pub fn acknowledge_connection(self, msg: &A2AMessage) -> VcxResult<InviterConnection<Completed>> {
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
