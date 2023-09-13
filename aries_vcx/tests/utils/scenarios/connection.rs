use aries_vcx::errors::error::VcxResult;
use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
use aries_vcx::handlers::util::AnyInvitation;
use aries_vcx::protocols::connection::{Connection, GenericConnection};
use aries_vcx::protocols::mediated_connection::pairwise_info::PairwiseInfo;
use aries_vcx::transport::Transport;
use async_trait::async_trait;
use messages::msg_fields::protocols::connection::invitation::{Invitation, InvitationContent};
use messages::msg_fields::protocols::out_of_band::invitation::OobService;
use messages::msg_fields::protocols::out_of_band::OobGoalCode;
use messages::msg_types::connection::{ConnectionType, ConnectionTypeV1};
use messages::msg_types::Protocol;
use url::Url;
use uuid::Uuid;

use crate::utils::test_agent::TestAgent;
use aries_vcx::common::ledger::transactions::into_did_doc;

async fn establish_connection_from_invite(
    alice: &mut TestAgent,
    faber: &mut TestAgent,
    invitation: AnyInvitation,
    inviter_pairwise_info: PairwiseInfo,
) -> (GenericConnection, GenericConnection) {
    // TODO: Temporary, delete
    struct DummyHttpClient;

    #[async_trait]
    impl Transport for DummyHttpClient {
        async fn send_message(&self, _msg: Vec<u8>, _service_endpoint: Url) -> VcxResult<()> {
            Ok(())
        }
    }

    let invitee_pairwise_info = PairwiseInfo::create(&alice.profile.inject_wallet()).await.unwrap();
    let invitee = Connection::new_invitee("".to_owned(), invitee_pairwise_info)
        .accept_invitation(&alice.profile.inject_indy_ledger_read(), invitation.clone())
        .await
        .unwrap()
        .prepare_request("http://dummy.org".parse().unwrap(), vec![])
        .await
        .unwrap();
    let request = invitee.get_request().clone();

    let inviter = Connection::new_inviter("".to_owned(), inviter_pairwise_info)
        .into_invited(invitation.id())
        .handle_request(
            &faber.profile.inject_wallet(),
            request,
            "http://dummy.org".parse().unwrap(),
            vec![],
            &DummyHttpClient,
        )
        .await
        .unwrap();
    let response = inviter.get_connection_response_msg();

    let invitee = invitee
        .handle_response(&alice.profile.inject_wallet(), response, &DummyHttpClient)
        .await
        .unwrap();
    let ack = invitee.get_ack();

    let inviter = inviter.acknowledge_connection(&ack.into()).unwrap();

    (invitee.into(), inviter.into())
}

pub async fn create_connections_via_oob_invite(
    alice: &mut TestAgent,
    faber: &mut TestAgent,
) -> (GenericConnection, GenericConnection) {
    let oob_sender = OutOfBandSender::create()
        .set_label("test-label")
        .set_goal_code(OobGoalCode::P2PMessaging)
        .set_goal("To exchange message")
        .append_service(&OobService::Did(faber.institution_did.clone()))
        .append_handshake_protocol(Protocol::ConnectionType(ConnectionType::V1(
            ConnectionTypeV1::new_v1_0(),
        )))
        .unwrap();
    let invitation = AnyInvitation::Oob(oob_sender.oob.clone());
    let ddo = into_did_doc(&alice.profile.inject_indy_ledger_read(), &invitation)
        .await
        .unwrap();
    // TODO: Create a key and write on ledger instead
    let inviter_pairwise_info = PairwiseInfo {
        pw_did: ddo.clone().id.clone(),
        pw_vk: ddo.recipient_keys().unwrap().first().unwrap().to_string(),
    };
    establish_connection_from_invite(alice, faber, invitation, inviter_pairwise_info).await
}

pub async fn create_connections_via_public_invite(
    alice: &mut TestAgent,
    faber: &mut TestAgent,
) -> (GenericConnection, GenericConnection) {
    let content = InvitationContent::builder_public()
        .label("faber".to_owned())
        .did(faber.institution_did.clone())
        .build();

    let public_invite = AnyInvitation::Con(
        Invitation::builder()
            .id("test_invite_id".to_owned())
            .content(content)
            .build(),
    );
    let ddo = into_did_doc(&alice.profile.inject_indy_ledger_read(), &public_invite)
        .await
        .unwrap();
    // TODO: Create a key and write on ledger instead
    let inviter_pairwise_info = PairwiseInfo {
        pw_did: ddo.clone().id.clone(),
        pw_vk: ddo.recipient_keys().unwrap().first().unwrap().to_string(),
    };
    establish_connection_from_invite(alice, faber, public_invite.clone(), inviter_pairwise_info).await
}

pub async fn create_connections_via_pairwise_invite(
    alice: &mut TestAgent,
    faber: &mut TestAgent,
) -> (GenericConnection, GenericConnection) {
    let inviter_pairwise_info = PairwiseInfo::create(&faber.profile.inject_wallet()).await.unwrap();
    let invite = {
        let id = Uuid::new_v4().to_string();
        let content = InvitationContent::builder_pairwise()
            .label("".to_string())
            .recipient_keys(vec![inviter_pairwise_info.pw_vk.clone()])
            .service_endpoint("http://dummy.org".parse().unwrap())
            .build();

        let invite = Invitation::builder().id(id).content(content).build();
        AnyInvitation::Con(invite)
    };

    establish_connection_from_invite(alice, faber, invite, inviter_pairwise_info).await
}
