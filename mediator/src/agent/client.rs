use std::sync::Arc;

use aries_vcx::handlers::util::AnyInvitation;
// use aries_vcx::protocols::connection::initiation_type::Invitee;
use aries_vcx::protocols::connection::invitee::states::initial::Initial as ClientInit;
// use aries_vcx::protocols::connection::invitee::states::invited::Invited;
use aries_vcx::protocols::connection::invitee::states::requested::Requested as ClientRequestSent;
use aries_vcx::protocols::connection::invitee::InviteeConnection;

use aries_vcx::protocols::mediated_connection::pairwise_info::PairwiseInfo;
// use aries_vcx::protocols::oob;
use aries_vcx::utils::mockdata::profile::mock_ledger::MockLedger;
use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::wallet::indy::IndySdkWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
// use diddoc_legacy::aries::service::AriesService;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use messages::msg_fields::protocols::out_of_band::invitation::OobService;

use super::Agent;
use crate::utils::prelude::*;

pub fn oob2did(oob: OOBInvitation) -> AriesDidDoc {
    let mut did_doc: AriesDidDoc = AriesDidDoc::default();
    did_doc.set_id(oob.id.clone());
    let oob_service = oob.content.services.first().expect("OOB needs a service").clone();

    match oob_service {
        OobService::AriesService(service) => {
            did_doc.set_service_endpoint(service.service_endpoint);
            did_doc.set_recipient_keys(service.recipient_keys);
            did_doc.set_routing_keys(service.routing_keys);
        }
        _ => panic!("Assuming fully clean AriesService variant only"),
    }
    did_doc
}

// client role utilities
impl Agent<IndySdkWallet> {
    /// Starts a new connection and tries to create request to the specified OOB invite endpoint
    pub async fn client_connect_req(&self, oob_invite: OOBInvitation) -> InviteeConnection<ClientRequestSent> {
        // let (did, vk) = self.wallet.create_and_store_my_did(None, None).await.unwrap();
        // let client_conn =
        //     InviteeConnection::<Initial>::new_invitee("foo".into(), PairwiseInfo { pw_did: did, pw_vk: vk });

        // fn accept_invitation(
        //     client_conn: InviteeConnection<Initial>,
        //     oob_invite: OOBInvitation,
        // ) -> InviteeConnection<Invited> {
        //     let did_doc = oob2did(oob_invite.clone());
        //     let state = Invited::new(did_doc, AnyInvitation::Oob(oob_invite));
        //     // Convert to `InvitedState`
        //     Connection {
        //         state,
        //         source_id: "foo".into(),
        //         pairwise_info: client_conn.pairwise_info().clone(),
        //         initiation_type: Invitee,
        //     }
        // }
        // let client_conn = accept_invitation(client_conn, oob_invite);
        // todo!()

        let (pw_did, pw_vk) = self.wallet.create_and_store_my_did(None, None).await.unwrap();

        let mock_ledger: Arc<dyn IndyLedgerRead> = Arc::new(MockLedger {}); // not good. will be dealt later. (can see brutish attempt above)
        let client_conn = InviteeConnection::<ClientInit>::new_invitee("foo".into(), PairwiseInfo { pw_did, pw_vk })
            .accept_invitation(&mock_ledger, AnyInvitation::Oob(oob_invite))
            .await
            .unwrap();

        let client_conn = client_conn
            .prepare_request("http://response.http.alt".parse().unwrap(), vec![])
            .await
            .unwrap();

        let msg_connection_request = client_conn.get_request().clone();
        info!("Client Connection Request: {:#?}", msg_connection_request);
        // invitee_requested
        //     .send_message(&self.wallet, &msg_connection_request.into(), &HttpClient)
        //     .await
        //     .unwrap();
        client_conn
    }
}
