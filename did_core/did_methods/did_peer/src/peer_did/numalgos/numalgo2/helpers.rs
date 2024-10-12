use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use did_doc::schema::did_doc::DidDocument;
use did_parser_nom::Did;
use public_key::Key;

use crate::{
    error::DidPeerError,
    peer_did::numalgos::numalgo2::{
        purpose::ElementPurpose,
        service_abbreviation::{deabbreviate_service, ServiceAbbreviatedDidPeer2},
        verification_method::get_verification_methods_by_key,
    },
    resolver::options::PublicKeyEncoding,
};

pub fn diddoc_from_peerdid2_elements(
    mut did_doc: DidDocument,
    did: &Did,
    public_key_encoding: PublicKeyEncoding,
) -> Result<DidDocument, DidPeerError> {
    let mut service_index: usize = 0;
    let mut vm_index: usize = 1;

    // Skipping one here because the first element is empty string
    for element in did.id()[1..].split('.').skip(1) {
        did_doc = add_attributes_from_element(
            element,
            did_doc,
            &mut service_index,
            &mut vm_index,
            did,
            public_key_encoding,
        )?;
    }

    Ok(did_doc)
}

fn add_attributes_from_element(
    element: &str,
    mut did_doc: DidDocument,
    service_index: &mut usize,
    vm_index: &mut usize,
    did: &Did,
    public_key_encoding: PublicKeyEncoding,
) -> Result<DidDocument, DidPeerError> {
    let purpose: ElementPurpose = element
        .chars()
        .next()
        .ok_or(DidPeerError::DidValidationError(format!(
            "No purpose code following element separator in '{}'",
            element
        )))?
        .try_into()?;
    let purposeless_element = &element[1..];

    if purpose == ElementPurpose::Service {
        did_doc = add_service_from_element(purposeless_element, did_doc, service_index)?;
    } else {
        did_doc = add_key_from_element(
            purposeless_element,
            did_doc,
            vm_index,
            did,
            public_key_encoding,
            purpose,
        )?;
    }

    Ok(did_doc)
}

fn add_service_from_element(
    element: &str,
    mut did_doc: DidDocument,
    service_index: &mut usize,
) -> Result<DidDocument, DidPeerError> {
    let decoded = STANDARD_NO_PAD.decode(element)?;
    let service: ServiceAbbreviatedDidPeer2 = serde_json::from_slice(&decoded)?;

    did_doc.add_service(deabbreviate_service(service, *service_index)?);
    *service_index += 1;

    Ok(did_doc)
}

fn add_key_from_element(
    element: &str,
    mut did_doc: DidDocument,
    vm_index: &mut usize,
    did: &Did,
    public_key_encoding: PublicKeyEncoding,
    purpose: ElementPurpose,
) -> Result<DidDocument, DidPeerError> {
    let key = Key::from_fingerprint(element)?;
    let vms = get_verification_methods_by_key(&key, did, public_key_encoding, vm_index)?;

    for vm in vms.into_iter() {
        let vm_reference = vm.id().to_owned();
        did_doc.add_verification_method(vm);
        // https://identity.foundation/peer-did-method-spec/#purpose-codes
        match purpose {
            ElementPurpose::Assertion => {
                did_doc.add_assertion_method_ref(vm_reference);
            }
            ElementPurpose::Encryption => {
                did_doc.add_key_agreement_ref(vm_reference);
            }
            ElementPurpose::Verification => {
                did_doc.add_authentication_ref(vm_reference);
            }
            ElementPurpose::CapabilityInvocation => {
                did_doc.add_capability_invocation_ref(vm_reference)
            }
            ElementPurpose::CapabilityDelegation => {
                did_doc.add_capability_delegation_ref(vm_reference)
            }
            _ => return Err(DidPeerError::UnsupportedPurpose(purpose.into())),
        }
    }

    Ok(did_doc)
}

#[cfg(test)]
mod tests {
    use did_doc::schema::service::typed::ServiceType;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_process_elements_empty_did() {
        let did: Did = "did:peer:2".parse().unwrap();

        let did_doc = diddoc_from_peerdid2_elements(
            DidDocument::new(did.clone()),
            &did,
            PublicKeyEncoding::Base58,
        )
        .unwrap();
        assert_eq!(did_doc.id().to_string(), did.to_string());
    }

    #[test]
    fn test_process_elements_with_multiple_elements() {
        let did: Did = "did:peer:2.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.\
             SeyJpZCI6IiNzZXJ2aWNlLTAiLCJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCJ9"
            .parse()
            .unwrap();

        let did_doc = diddoc_from_peerdid2_elements(
            DidDocument::new(did.clone()),
            &did,
            PublicKeyEncoding::Multibase,
        )
        .unwrap();

        assert_eq!(did_doc.id().to_string(), did.to_string());
        assert_eq!(did_doc.verification_method().len(), 1);
        assert_eq!(did_doc.service().len(), 1);
    }

    #[test]
    fn test_process_elements_error_on_invalid_element() {
        let did: Did = "did:peer:2.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.\
             SeyJpZCI6IiNzZXJ2aWNlLTAiLCJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCJ9.\
             Xinvalid"
            .parse()
            .unwrap();

        match diddoc_from_peerdid2_elements(
            DidDocument::new(did.clone()),
            &did,
            PublicKeyEncoding::Multibase,
        ) {
            Ok(_) => panic!("Expected Err, got Ok"),
            Err(e) => {
                assert!(matches!(e, DidPeerError::UnsupportedPurpose('X')));
            }
        }
    }

    #[test]
    fn test_process_service_element_one_service() {
        let purposeless_service_element =
            "eyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCJ9";
        let did: Did = format!("did:peer:2.S{}", purposeless_service_element)
            .parse()
            .unwrap();
        let mut index = 0;
        let ddo_builder = DidDocument::new(did);
        let did_doc =
            add_service_from_element(purposeless_service_element, ddo_builder, &mut index).unwrap();
        assert_eq!(did_doc.service().len(), 1);
        let service = did_doc.service().first().unwrap();
        assert_eq!(service.id().to_string(), "#service-0".to_string());
        assert_eq!(service.service_types(), vec!(ServiceType::DIDCommV2));
        assert_eq!(
            service.service_endpoint().to_string(),
            "https://example.com/endpoint".to_string()
        );
    }

    #[test]
    fn test_process_key_element() {
        let purposeless_key_element = "z6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V";
        let did: Did = format!("did:peer:2.V{}", purposeless_key_element)
            .parse()
            .unwrap();

        let ddo_builder = DidDocument::new(did.clone());
        let public_key_encoding = PublicKeyEncoding::Multibase;
        let did_doc = add_key_from_element(
            purposeless_key_element,
            ddo_builder,
            &mut 0,
            &did,
            public_key_encoding,
            ElementPurpose::Verification,
        )
        .unwrap();

        assert_eq!(did_doc.verification_method().len(), 1);
        let vm = did_doc.verification_method().first().unwrap();
        assert_eq!(vm.id().to_string(), "#key-0");
        assert_eq!(vm.controller().to_string(), did.to_string());
    }

    #[test]
    fn test_process_key_element_negative() {
        let did: Did = "did:peer:2".parse().unwrap();
        assert!(add_key_from_element(
            "z6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V",
            DidDocument::new(did.clone()),
            &mut 0,
            &did,
            PublicKeyEncoding::Multibase,
            ElementPurpose::Service
        )
        .is_err());
    }
}
