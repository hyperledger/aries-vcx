pub mod error;

use std::collections::HashMap;

use did_resolver::{
    did_parser::ParsedDID,
    error::GenericError,
    traits::resolvable::{
        resolution_options::DIDResolutionOptions, resolution_output::DIDResolutionOutput,
        DIDResolvable,
    },
};
use error::DIDResolverRegistryError;

pub struct ResolverRegistry {
    resolvers: HashMap<String, Box<dyn DIDResolvable>>,
}

impl ResolverRegistry {
    pub fn new() -> Self {
        ResolverRegistry {
            resolvers: HashMap::new(),
        }
    }

    pub fn register_resolver(&mut self, method: String, resolver: Box<dyn DIDResolvable>) {
        self.resolvers.insert(method, resolver);
    }

    pub fn unregister_resolver(&mut self, method: &str) {
        self.resolvers.remove(method);
    }

    pub async fn resolve(
        &self,
        did: &ParsedDID,
        options: &DIDResolutionOptions,
    ) -> Result<DIDResolutionOutput, GenericError> {
        let method = did.method();
        match self.resolvers.get(method) {
            Some(resolver) => resolver.resolve(did, options).await,
            None => Err(Box::new(DIDResolverRegistryError::UnsupportedMethod)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use did_resolver::did_doc_builder::schema::did_doc::DIDDocumentBuilder;
    use mockall::{automock, predicate::eq};
    use std::{error::Error, pin::Pin};

    struct DummyDIDResolver;

    #[async_trait]
    #[automock]
    impl DIDResolvable for DummyDIDResolver {
        async fn resolve(
            &self,
            did: &ParsedDID,
            _options: &DIDResolutionOptions,
        ) -> Result<DIDResolutionOutput, GenericError> {
            Ok(DIDResolutionOutput::builder(
                DIDDocumentBuilder::new(ParsedDID::parse(did.did().to_string()).unwrap()).build(),
            )
            .build())
        }
    }

    #[derive(Debug)]
    struct DummyResolverError;

    impl std::fmt::Display for DummyResolverError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Dummy resolver error")
        }
    }

    impl Error for DummyResolverError {}

    #[tokio::test]
    async fn test_resolve_error() {
        let did = ParsedDID::parse("did:example:1234".to_string()).unwrap();
        let method = did.method().to_string();

        let mut mock_resolver = MockDummyDIDResolver::new();
        mock_resolver
            .expect_resolve()
            .with(eq(did.clone()), eq(DIDResolutionOptions::default()))
            .times(1)
            .return_once(move |_, _| {
                let future = async move {
                    Err::<DIDResolutionOutput, GenericError>(Box::new(DummyResolverError))
                };
                Pin::from(Box::new(future))
            });

        let mut registry = ResolverRegistry::new();
        registry.register_resolver(method, Box::new(mock_resolver));

        let result = registry
            .resolve(&did, &DIDResolutionOptions::default())
            .await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        if let Some(_) = error.downcast_ref::<DummyResolverError>() {
            assert!(true);
        } else {
            assert!(false, "Error is not of type DummyResolverError");
        }
    }

    #[tokio::test]
    async fn test_resolve_success() {
        let did = "did:example:1234";
        let parsed_did = ParsedDID::parse(did.to_string()).unwrap();
        let method = parsed_did.method().to_string();

        let mut mock_resolver = MockDummyDIDResolver::new();
        mock_resolver
            .expect_resolve()
            .with(eq(parsed_did.clone()), eq(DIDResolutionOptions::default()))
            .times(1)
            .return_once(move |_, _| {
                let future = async move {
                    Ok::<DIDResolutionOutput, GenericError>(
                        DIDResolutionOutput::builder(
                            DIDDocumentBuilder::new(ParsedDID::parse(did.to_string()).unwrap())
                                .build(),
                        )
                        .build(),
                    )
                };
                Pin::from(Box::new(future))
            });

        let mut registry = ResolverRegistry::new();
        registry.register_resolver(method, Box::new(mock_resolver));

        let result = registry
            .resolve(&parsed_did, &DIDResolutionOptions::default())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_and_unregister_resolver() {
        let method = "example".to_string();
        let mock_resolver = MockDummyDIDResolver::new();

        let mut registry = ResolverRegistry::new();
        assert_eq!(registry.resolvers.len(), 0);

        registry.register_resolver(method.clone(), Box::new(mock_resolver));
        assert_eq!(registry.resolvers.len(), 1);
        assert!(registry.resolvers.contains_key(&method));

        registry.unregister_resolver(&method);
        assert_eq!(registry.resolvers.len(), 0);
        assert!(!registry.resolvers.contains_key(&method));
    }

    #[tokio::test]
    async fn test_resolve_unsupported_method() {
        let did = ParsedDID::parse("did:unknown:1234".to_string()).unwrap();

        let registry = ResolverRegistry::new();
        let result = registry
            .resolve(&did, &DIDResolutionOptions::default())
            .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        if let Some(err) = error.downcast_ref::<DIDResolverRegistryError>() {
            assert!(matches!(err, DIDResolverRegistryError::UnsupportedMethod));
        } else {
            assert!(false, "Error is not of type DIDResolverRegistryError");
        }
    }

    #[tokio::test]
    async fn test_resolve_after_registering_resolver() {
        let did = "did:example:1234";
        let parsed_did = ParsedDID::parse(did.to_string()).unwrap();
        let method = parsed_did.method().to_string();

        let mut mock_resolver = MockDummyDIDResolver::new();
        mock_resolver
            .expect_resolve()
            .with(eq(parsed_did.clone()), eq(DIDResolutionOptions::default()))
            .times(1)
            .return_once(move |_, _| {
                let future = async move {
                    Ok::<DIDResolutionOutput, GenericError>(
                        DIDResolutionOutput::builder(
                            DIDDocumentBuilder::new(ParsedDID::parse(did.to_string()).unwrap())
                                .build(),
                        )
                        .build(),
                    )
                };
                Pin::from(Box::new(future))
            });

        let mut registry = ResolverRegistry::new();

        let result_before = registry
            .resolve(&parsed_did, &DIDResolutionOptions::default())
            .await;
        assert!(result_before.is_err());
        let error_before = result_before.unwrap_err();
        if let Some(err) = error_before.downcast_ref::<DIDResolverRegistryError>() {
            assert!(matches!(err, DIDResolverRegistryError::UnsupportedMethod));
        } else {
            assert!(false, "Error is not of type DIDResolverRegistryError");
        }

        registry.register_resolver(method, Box::new(mock_resolver));

        let result_after = registry
            .resolve(&parsed_did, &DIDResolutionOptions::default())
            .await;
        assert!(result_after.is_ok());
    }
}
