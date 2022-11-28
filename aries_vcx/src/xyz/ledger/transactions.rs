use std::sync::Arc;

use messages::{did_doc::{service_resolvable::ServiceResolvable, service_aries::AriesService, DidDoc}, connection::invite::Invitation};

use crate::{core::profile::profile::Profile, error::VcxResult};

pub async fn resolve_service(profile: &Arc<dyn Profile>, service: &ServiceResolvable) -> VcxResult<AriesService> {
    let ledger = Arc::clone(profile).inject_ledger();
    match service {
        ServiceResolvable::AriesService(service) => Ok(service.clone()),
        ServiceResolvable::Did(did) => ledger.get_service(did).await,
    }
}

pub async fn add_new_did(profile: &Arc<dyn Profile>, submitter_did: &str, role: Option<&str>) -> VcxResult<(String, String)> {
    let (did, verkey) = profile.inject_wallet().create_and_store_my_did(None, None).await?;

    let ledger = Arc::clone(profile).inject_ledger();

    ledger.publish_nym(submitter_did, &did, Some(&verkey),None, role).await?;
    // validate response?

    Ok((did, verkey))
}

pub async fn into_did_doc(profile: &Arc<dyn Profile>, invitation: &Invitation) -> VcxResult<DidDoc> {
    let ledger = Arc::clone(profile).inject_ledger();
    let mut did_doc: DidDoc = DidDoc::default();
    let (service_endpoint, recipient_keys, routing_keys) = match invitation {
        Invitation::Public(invitation) => {
            did_doc.set_id(invitation.did.to_string());
            let service = ledger.get_service(&invitation.did).await.unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {}", err);
                AriesService::default()
            });
            (service.service_endpoint, service.recipient_keys, service.routing_keys)
        }
        Invitation::Pairwise(invitation) => {
            did_doc.set_id(invitation.id.0.clone());
            (
                invitation.service_endpoint.clone(),
                invitation.recipient_keys.clone(),
                invitation.routing_keys.clone(),
            )
        }
        Invitation::OutOfBand(invitation) => {
            did_doc.set_id(invitation.id.0.clone());
            let service = resolve_service(profile, &invitation.services[0]).await.unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {}", err);
                AriesService::default()
            });
            (service.service_endpoint, service.recipient_keys, service.routing_keys)
        }
    };
    did_doc.set_service_endpoint(service_endpoint);
    did_doc.set_recipient_keys(recipient_keys);
    did_doc.set_routing_keys(routing_keys);
    Ok(did_doc)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod test {
    use messages::a2a::MessageId;
    use messages::did_doc::test_utils::{_recipient_keys, _routing_keys, _service_endpoint};
    use messages::connection::invite::test_utils::_pairwise_invitation;

    use crate::xyz::test_utils::mock_profile;

    use super::*;

    #[tokio::test]
    async fn test_did_doc_from_invitation_works() {
        let mut did_doc = DidDoc::default();
        did_doc.set_id(MessageId::id().0);
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_recipient_keys(_recipient_keys());
        did_doc.set_routing_keys(_routing_keys());
        assert_eq!(did_doc, into_did_doc(&mock_profile(), &Invitation::Pairwise(_pairwise_invitation())).await.unwrap());
    }
}