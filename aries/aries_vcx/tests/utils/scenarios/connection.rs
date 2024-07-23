use aries_vcx::{
    handlers::{out_of_band::sender::OutOfBandSender, util::AnyInvitation},
    protocols::{
        connection::{invitee::any_invitation_into_did_doc, Connection, GenericConnection},
        mediated_connection::pairwise_info::PairwiseInfo,
    },
};
use aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::base_ledger::{
    AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use messages::{
    msg_fields::protocols::{
        connection::invitation::{Invitation, InvitationContent},
        out_of_band::{invitation::OobService, OobGoalCode},
    },
    msg_types::{
        connection::{ConnectionType, ConnectionTypeV1},
        Protocol,
    },
};
use uuid::Uuid;

use crate::utils::test_agent::TestAgent;

async fn establish_connection_from_invite(
    alice: &mut TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    faber: &mut TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    invitation: AnyInvitation,
    inviter_pairwise_info: PairwiseInfo,
) -> (GenericConnection, GenericConnection) {
    let invitee_pairwise_info = PairwiseInfo::create(&alice.wallet).await.unwrap();
    let invitee = Connection::new_invitee("".to_owned(), invitee_pairwise_info)
        .accept_invitation(&alice.ledger_read, invitation.clone())
        .await
        .unwrap()
        .prepare_request("http://dummy.org".parse().unwrap(), vec![])
        .await
        .unwrap();
    let request = invitee.get_request().clone();

    let inviter = Connection::new_inviter("".to_owned(), inviter_pairwise_info)
        .into_invited(invitation.id())
        .handle_request(
            &faber.wallet,
            request,
            "http://dummy.org".parse().unwrap(),
            vec![],
        )
        .await
        .unwrap();
    let response = inviter.get_connection_response_msg();

    let invitee = invitee
        .handle_response(&alice.wallet, response)
        .await
        .unwrap();
    let ack = invitee.get_ack();

    let inviter = inviter.acknowledge_connection(&ack.into()).unwrap();

    (invitee.into(), inviter.into())
}

pub async fn create_connections_via_oob_invite(
    alice: &mut TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    faber: &mut TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
) -> (GenericConnection, GenericConnection) {
    let oob_sender = OutOfBandSender::create()
        .set_label("test-label")
        .set_goal_code(OobGoalCode::P2PMessaging)
        .set_goal("To exchange message")
        .append_service(&OobService::Did(faber.institution_did.to_string()))
        .append_handshake_protocol(Protocol::ConnectionType(ConnectionType::V1(
            ConnectionTypeV1::new_v1_0(),
        )))
        .unwrap();
    let invitation = AnyInvitation::Oob(oob_sender.oob.clone());
    let ddo = any_invitation_into_did_doc(&alice.ledger_read, &invitation)
        .await
        .unwrap();
    // TODO: Create a key and write on ledger instead
    let inviter_pairwise_info = PairwiseInfo {
        pw_did: ddo.clone().id,
        pw_vk: ddo.recipient_keys().unwrap().first().unwrap().to_string(),
    };
    establish_connection_from_invite(alice, faber, invitation, inviter_pairwise_info).await
}

pub async fn create_connections_via_public_invite(
    alice: &mut TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    faber: &mut TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
) -> (GenericConnection, GenericConnection) {
    let content = InvitationContent::builder_public()
        .label("faber".to_owned())
        .did(faber.institution_did.to_string())
        .build();

    let public_invite = AnyInvitation::Con(
        Invitation::builder()
            .id("test_invite_id".to_owned())
            .content(content)
            .build(),
    );
    let ddo = any_invitation_into_did_doc(&alice.ledger_read, &public_invite)
        .await
        .unwrap();
    // TODO: Create a key and write on ledger instead
    let inviter_pairwise_info = PairwiseInfo {
        pw_did: ddo.clone().id,
        pw_vk: ddo.recipient_keys().unwrap().first().unwrap().to_string(),
    };
    establish_connection_from_invite(alice, faber, public_invite.clone(), inviter_pairwise_info)
        .await
}

pub async fn create_connections_via_pairwise_invite(
    alice: &mut TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    faber: &mut TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
) -> (GenericConnection, GenericConnection) {
    let inviter_pairwise_info = PairwiseInfo::create(&faber.wallet).await.unwrap();
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
