pub mod states;

use ::uuid::Uuid;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::connection::{
        invitation::{Invitation, InvitationContent},
        request::Request,
        response::{Response, ResponseContent, ResponseDecorators},
        ConnectionData,
    },
    AriesMessage,
};
use url::Url;

use self::states::{
    completed::Completed, initial::Initial, invited::Invited, requested::Requested,
};
use super::{initiation_type::Inviter, pairwise_info::PairwiseInfo, Connection};
use crate::{
    common::signing::sign_connection_response,
    errors::error::VcxResult,
    handlers::util::{verify_thread_id, AnyInvitation},
    protocols::connection::trait_bounds::ThreadId,
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
    pub fn create_invitation(
        self,
        routing_keys: Vec<String>,
        service_endpoint: Url,
    ) -> InviterConnection<Invited> {
        let id = Uuid::new_v4().to_string();
        let content = InvitationContent::builder_pairwise()
            .label(self.source_id.clone())
            .recipient_keys(vec![self.pairwise_info.pw_vk.clone()])
            .routing_keys(routing_keys)
            .service_endpoint(service_endpoint)
            .build();

        let invite = Invitation::builder().id(id).content(content).build();

        let invitation = AnyInvitation::Con(invite);

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
        let id = thread_id.to_owned();

        let content = InvitationContent::builder_pairwise()
            .label(self.source_id.clone())
            .recipient_keys(vec![self.pairwise_info.pw_vk.clone()])
            .service_endpoint(
                "https://dummy.dummy/dummy"
                    .parse()
                    .expect("url should be valid"),
            )
            .build();

        let invite = Invitation::builder().id(id).content(content).build();

        let invitation = AnyInvitation::Con(invite);

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
        wallet: &impl BaseWallet,
        thread_id: String,
        new_pairwise_info: &PairwiseInfo,
        new_service_endpoint: Url,
        new_routing_keys: Vec<String>,
    ) -> VcxResult<Response> {
        let new_recipient_keys = vec![new_pairwise_info.pw_vk.clone()];
        let mut did_doc = AriesDidDoc::default();
        let did = new_pairwise_info.pw_did.clone();

        did_doc.set_id(new_pairwise_info.pw_did.clone());
        did_doc.set_service_endpoint(new_service_endpoint);
        did_doc.set_routing_keys(new_routing_keys);
        did_doc.set_recipient_keys(new_recipient_keys);

        let con_data = ConnectionData::new(did, did_doc);

        let id = Uuid::new_v4().to_string();

        let con_sig =
            sign_connection_response(wallet, &self.pairwise_info.pw_vk, &con_data).await?;

        let content = ResponseContent::builder().connection_sig(con_sig).build();

        let decorators = ResponseDecorators::builder()
            .thread(Thread::builder().thid(thread_id).build())
            .timing(Timing::builder().out_time(Utc::now()).build())
            .build();

        Ok(Response::builder()
            .id(id)
            .content(content)
            .decorators(decorators)
            .build())
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
    pub async fn handle_request(
        self,
        wallet: &impl BaseWallet,
        request: Request,
        new_service_endpoint: Url,
        new_routing_keys: Vec<String>,
    ) -> VcxResult<InviterConnection<Requested>> {
        trace!(
            "Connection::process_request >>> request: {:?}, service_endpoint: {}, routing_keys: \
             {:?}",
            request,
            new_service_endpoint,
            new_routing_keys,
        );

        // There must be some other way to validate the thread ID other than cloning the entire
        // Request
        verify_thread_id(self.thread_id(), &request.clone().into())?;
        request.content.connection.did_doc.validate()?;

        // Generate new pairwise info that will be used from this point on
        // and incorporate that into the response.
        let new_pairwise_info = PairwiseInfo::create(wallet).await?;
        let thread_id = request
            .decorators
            .thread
            .map(|t| t.thid)
            .unwrap_or(request.id);
        let did_doc = request.content.connection.did_doc;

        let content = self
            .build_response_content(
                wallet,
                thread_id,
                &new_pairwise_info,
                new_service_endpoint,
                new_routing_keys,
            )
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
    /// Returns pre-built [`Response`] message which shall be delivered to counterparty
    ///
    /// # Errors
    ///
    /// Will return an error if sending the response fails.
    pub fn get_connection_response_msg(&self) -> Response {
        self.state.signed_response.clone()
    }

    /// Acknowledges an invitee's connection by processing their first message
    /// and transitions to [`InviterConnection<Completed>`].
    ///
    /// # Errors
    ///
    /// Will error out if the message's thread ID does not match
    /// the ID of the thread context used in this connection.
    pub fn acknowledge_connection(
        self,
        msg: &AriesMessage,
    ) -> VcxResult<InviterConnection<Completed>> {
        verify_thread_id(self.state.thread_id(), msg)?;
        let state = Completed::new(
            self.state.did_doc,
            self.state.signed_response.decorators.thread.thid,
            None,
        );

        Ok(Connection {
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: self.initiation_type,
            state,
        })
    }
}
