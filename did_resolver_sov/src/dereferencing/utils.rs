use std::io::Cursor;

use did_resolver::{
    did_doc::schema::{
        did_doc::DidDocument, service::Service, verification_method::VerificationMethod,
    },
    did_parser::DidUrl,
    traits::{
        dereferenceable::{
            dereferencing_metadata::DidDereferencingMetadata,
            dereferencing_output::DidDereferencingOutput,
        },
        resolvable::resolution_output::DidResolutionOutput,
    },
};
use serde::Serialize;

use crate::error::DidSovError;

pub fn service_by_id<F, E: Default>(services: &[Service<E>], predicate: F) -> Option<&Service<E>>
where
    F: Fn(&str) -> bool,
{
    services.iter().find(|svc| predicate(svc.id().as_ref()))
}

pub fn verification_by_id<F>(
    authentications: &[VerificationMethod],
    predicate: F,
) -> Option<&VerificationMethod>
where
    F: Fn(&str) -> bool,
{
    authentications
        .iter()
        .find(|auth| predicate(auth.id().did_url()))
}

fn content_stream_from<E: Default + Serialize>(
    did_document: &DidDocument<E>,
    did_url: &DidUrl,
) -> Result<Cursor<Vec<u8>>, DidSovError> {
    let fragment = did_url.fragment().ok_or_else(|| {
        DidSovError::InvalidDid(format!("No fragment provided in the DID URL {}", did_url))
    })?;

    let did_url_string = did_url.to_string();
    let fragment_string = format!("#{}", fragment);
    let id_matcher = |id: &str| id == did_url_string || id.ends_with(&fragment_string);

    let value = match (
        service_by_id(did_document.service(), id_matcher),
        verification_by_id(did_document.verification_method(), id_matcher),
    ) {
        (Some(service), None) => serde_json::to_value(service)?,
        (None, Some(authentication)) => serde_json::to_value(authentication)?,
        (None, None) => {
            return Err(DidSovError::NotFound(format!(
                "Fragment '{}' not found in the DID document",
                fragment
            )));
        }
        (Some(_), Some(_)) => {
            return Err(DidSovError::InvalidDid(format!(
                "Fragment '{}' is ambiguous",
                fragment
            )));
        }
    };
    Ok(Cursor::new(value.to_string().into_bytes()))
}

// TODO: Currently, only fragment dereferencing is supported
pub(crate) fn dereference_did_document<E: Default + Serialize>(
    resolution_output: &DidResolutionOutput<E>,
    did_url: &DidUrl,
) -> Result<DidDereferencingOutput<Cursor<Vec<u8>>>, DidSovError> {
    let content_stream = content_stream_from(resolution_output.did_document(), did_url)?;

    let content_metadata = resolution_output.did_document_metadata().clone();

    let dereferencing_metadata = DidDereferencingMetadata::builder()
        .content_type("application/did+json".to_string())
        .build();

    Ok(DidDereferencingOutput::builder(content_stream)
        .content_metadata(content_metadata)
        .dereferencing_metadata(dereferencing_metadata)
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;

    use did_resolver::did_doc::schema::did_doc::DidDocumentBuilder;
    use did_resolver::did_parser::DidUrl;
    use did_resolver::traits::resolvable::resolution_output::DidResolutionOutput;
    use serde_json::Value;

    fn example_did_document_builder() -> DidDocumentBuilder<()> {
        let verification_method = VerificationMethod::builder(
            DidUrl::parse("did:example:123456789abcdefghi#keys-1".to_string()).unwrap(),
            "did:example:123456789abcdefghi"
                .to_string()
                .try_into()
                .unwrap(),
            "Ed25519VerificationKey2018".to_string(),
        )
        .add_extra_field(
            "publicKeyBase58".to_string(),
            Value::String("H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string()),
        )
        .build();

        let agent_service = Service::builder(
            "did:example:123456789abcdefghi#agent".parse().unwrap(),
            "https://agent.example.com/8377464".try_into().unwrap(),
            Default::default(),
        )
        .add_service_type("AgentService".to_string())
        .unwrap()
        .build();

        let messaging_service = Service::builder(
            "did:example:123456789abcdefghi#messages".parse().unwrap(),
            "https://example.com/messages/8377464".try_into().unwrap(),
            Default::default(),
        )
        .add_service_type("MessagingService".to_string())
        .unwrap()
        .build();

        DidDocument::builder(Default::default())
            .add_verification_method(verification_method)
            .add_service(agent_service)
            .add_service(messaging_service)
    }

    fn example_resolution_output() -> DidResolutionOutput<()> {
        DidResolutionOutput::builder(example_did_document_builder().build()).build()
    }

    #[test]
    fn test_content_stream_from() {
        let did_document = example_did_document_builder().build();
        let did_url = DidUrl::parse("did:example:123456789abcdefghi#keys-1".to_string()).unwrap();
        let content_stream = content_stream_from(&did_document, &did_url).unwrap();
        let content_value: Value = serde_json::from_reader(content_stream).unwrap();

        let expected = serde_json::json!(
            {
                "id": "did:example:123456789abcdefghi#keys-1",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:example:123456789abcdefghi",
                "publicKeyBase58": "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV"
            }
        );
        assert_eq!(content_value, expected);
    }

    #[test]
    fn test_dereference_did_document() {
        let resolution_output = example_resolution_output();
        let did_url = DidUrl::parse("did:example:123456789abcdefghi#keys-1".to_string()).unwrap();
        let dereferencing_output = dereference_did_document(&resolution_output, &did_url).unwrap();

        let content_value: Value =
            serde_json::from_reader(dereferencing_output.content_stream().clone()).unwrap();

        let expected = serde_json::json!(
            {
                "id": "did:example:123456789abcdefghi#keys-1",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:example:123456789abcdefghi",
                "publicKeyBase58": "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV"
            }
        );
        assert_eq!(content_value, expected);

        let content_metadata = dereferencing_output.content_metadata();
        assert_eq!(content_metadata, resolution_output.did_document_metadata());

        let dereferencing_metadata = dereferencing_output.dereferencing_metadata();
        assert_eq!(
            dereferencing_metadata.content_type(),
            Some(&"application/did+json".to_string())
        );
    }

    #[test]
    fn test_dereference_did_document_not_found() {
        let resolution_output = example_resolution_output();
        let did_url =
            DidUrl::parse("did:example:123456789abcdefghi#non-existent".to_string()).unwrap();
        let result = dereference_did_document(&resolution_output, &did_url);
        assert!(matches!(result, Err(DidSovError::NotFound(_))));
    }

    #[test]
    fn test_dereference_did_document_ambiguous() {
        let did_document = {
            let did_document_builder = example_did_document_builder();
            let additional_service = Service::builder(
                "did:example:123456789abcdefghi#keys-1".parse().unwrap(),
                "https://example.com/duplicated/8377464".try_into().unwrap(),
                Default::default(),
            )
            .add_service_type("DuplicatedService".to_string())
            .unwrap()
            .build();
            did_document_builder.add_service(additional_service).build()
        };

        let resolution_output = DidResolutionOutput::builder(did_document).build();
        let did_url = DidUrl::parse("did:example:123456789abcdefghi#keys-1".to_string()).unwrap();
        let result = dereference_did_document(&resolution_output, &did_url);
        assert!(matches!(result, Err(DidSovError::InvalidDid(_))));
    }
}
