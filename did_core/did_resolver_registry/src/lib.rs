pub mod error;

use std::collections::HashMap;

use async_trait::async_trait;
use did_resolver::{
    did_doc::schema::did_doc::DidDocument,
    did_parser_nom::Did,
    error::GenericError,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};
use error::DidResolverRegistryError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type GenericResolver = dyn DidResolvableAdaptorTrait + Send + Sync;

#[derive(Default)]
pub struct ResolverRegistry {
    resolvers: HashMap<String, Box<GenericResolver>>,
}

pub struct DidResolvableAdaptor<T: DidResolvable> {
    inner: T,
}

#[async_trait]
pub trait DidResolvableAdaptorTrait: Send + Sync {
    async fn resolve(
        &self,
        did: &Did,
        options: HashMap<String, Value>,
    ) -> Result<DidResolutionOutput, GenericError>;
}

#[async_trait]
impl<T: DidResolvable + Send + Sync> DidResolvableAdaptorTrait for DidResolvableAdaptor<T>
where
    T::DidResolutionOptions: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    async fn resolve(
        &self,
        did: &Did,
        options: HashMap<String, Value>,
    ) -> Result<DidResolutionOutput, GenericError> {
        let options: T::DidResolutionOptions = if options.is_empty() {
            Default::default()
        } else {
            let json_map = options.into_iter().collect();
            serde_json::from_value(Value::Object(json_map))?
        };
        let result_inner = self.inner.resolve(did, &options).await?;

        let did_document_inner_hashmap = serde_json::to_value(result_inner.did_document)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();

        let did_document: DidDocument =
            serde_json::from_value(Value::Object(did_document_inner_hashmap))?;

        Ok(DidResolutionOutput::builder(did_document)
            .did_resolution_metadata(result_inner.did_resolution_metadata)
            .did_document_metadata(result_inner.did_document_metadata)
            .build())
    }
}

impl ResolverRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_resolver<T>(mut self, method: String, resolver: T) -> Self
    where
        T: DidResolvable + 'static + Send + Sync,
        for<'de> <T as DidResolvable>::DidResolutionOptions:
            Send + Sync + Serialize + Deserialize<'de>,
    {
        let adaptor = DidResolvableAdaptor { inner: resolver };
        self.resolvers.insert(method, Box::new(adaptor));
        self
    }

    pub fn unregister_resolver(mut self, method: &str) -> Self {
        self.resolvers.remove(method);
        self
    }

    pub async fn resolve(
        &self,
        did: &Did,
        options: &HashMap<String, Value>,
    ) -> Result<DidResolutionOutput, GenericError> {
        let method = did
            .method()
            .ok_or(DidResolverRegistryError::UnsupportedMethod)?;
        match self.resolvers.get(method) {
            Some(resolver) => resolver.resolve(did, options.clone()).await,
            None => Err(Box::new(DidResolverRegistryError::UnsupportedMethod)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, pin::Pin};

    use async_trait::async_trait;
    use did_resolver::did_doc::schema::did_doc::DidDocument;
    use mockall::automock;

    use super::*;

    #[allow(unused)] // false positive. used for automock
    struct DummyDidResolver;

    #[async_trait]
    #[automock]
    impl DidResolvable for DummyDidResolver {
        type DidResolutionOptions = ();

        async fn resolve(
            &self,
            did: &Did,
            _options: &Self::DidResolutionOptions,
        ) -> Result<DidResolutionOutput, GenericError> {
            Ok(DidResolutionOutput::builder(DidDocument::new(did.clone())).build())
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
        let did = Did::parse("did:example:1234".to_string()).unwrap();
        let method = did.method().unwrap().to_string();

        let mut mock_resolver = MockDummyDidResolver::new();
        mock_resolver
            .expect_resolve()
            // .with(eq(did.clone()), eq(DidResolutionOptions::default()))
            .times(1)
            .return_once(move |_, _| {
                let future = async move {
                    Err::<DidResolutionOutput, GenericError>(Box::new(DummyResolverError))
                };
                Pin::from(Box::new(future))
            });

        let registry = ResolverRegistry::new()
            .register_resolver::<MockDummyDidResolver>(method, mock_resolver);

        let result = registry.resolve(&did, &HashMap::new()).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(
            error.downcast_ref::<DummyResolverError>().is_some(),
            "Error is not of type DummyResolverError"
        )
    }

    #[tokio::test]
    async fn test_resolve_success() {
        let did = "did:example:1234";
        let parsed_did = Did::parse(did.to_string()).unwrap();
        let parsed_did_cp = parsed_did.clone();
        let method = parsed_did.method().unwrap().to_string();

        let mut mock_resolver = MockDummyDidResolver::new();
        mock_resolver
            .expect_resolve()
            // .with(eq(parsed_did.clone()), eq(DidResolutionOptions::default()))
            .times(1)
            .return_once(move |_, _| {
                let future = async move {
                    Ok::<DidResolutionOutput, GenericError>(
                        DidResolutionOutput::builder(DidDocument::new(parsed_did_cp)).build(),
                    )
                };
                Pin::from(Box::new(future))
            });

        let registry = ResolverRegistry::new()
            .register_resolver::<MockDummyDidResolver>(method, mock_resolver);

        let result = registry.resolve(&parsed_did, &HashMap::new()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_and_unregister_resolver() {
        let method = "example".to_string();
        let mock_resolver = MockDummyDidResolver::new();

        let mut registry = ResolverRegistry::new();
        assert_eq!(registry.resolvers.len(), 0);

        registry =
            registry.register_resolver::<MockDummyDidResolver>(method.clone(), mock_resolver);
        assert_eq!(registry.resolvers.len(), 1);
        assert!(registry.resolvers.contains_key(&method));

        registry = registry.unregister_resolver(&method);
        assert_eq!(registry.resolvers.len(), 0);
        assert!(!registry.resolvers.contains_key(&method));
    }

    #[tokio::test]
    async fn test_resolve_unsupported_method() {
        let did = Did::parse("did:unknown:1234".to_string()).unwrap();

        let registry = ResolverRegistry::new();
        let result = registry.resolve(&did, &HashMap::new()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            matches!(
                error.downcast_ref::<DidResolverRegistryError>(),
                Some(DidResolverRegistryError::UnsupportedMethod)
            ),
            "Error is not of type DidResolverRegistryError"
        );
    }

    #[tokio::test]
    async fn test_resolve_after_registering_resolver() {
        let did = "did:example:1234";
        let parsed_did = Did::parse(did.to_string()).unwrap();
        let parsed_did_cp = parsed_did.clone();
        let method = parsed_did.method().unwrap().to_string();

        let mut mock_resolver = MockDummyDidResolver::new();
        mock_resolver
            .expect_resolve()
            // .with(eq(parsed_did.clone()), eq(DidResolutionOptions::default()))
            .times(1)
            .return_once(move |_, _| {
                let future = async move {
                    Ok::<DidResolutionOutput, GenericError>(
                        DidResolutionOutput::builder(DidDocument::new(parsed_did_cp)).build(),
                    )
                };
                Pin::from(Box::new(future))
            });

        let mut registry = ResolverRegistry::new();

        let result_before = registry.resolve(&parsed_did, &HashMap::new()).await;
        assert!(result_before.is_err());
        let error_before = result_before.unwrap_err();
        assert!(
            matches!(
                error_before.downcast_ref::<DidResolverRegistryError>(),
                Some(DidResolverRegistryError::UnsupportedMethod)
            ),
            "Error is not of type DidResolverRegistryError"
        );

        registry = registry.register_resolver::<MockDummyDidResolver>(method, mock_resolver);

        let result_after = registry.resolve(&parsed_did, &HashMap::new()).await;
        assert!(result_after.is_ok());
    }
}
