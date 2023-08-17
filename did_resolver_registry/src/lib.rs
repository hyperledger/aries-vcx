pub mod error;

use std::collections::HashMap;

use async_trait::async_trait;
use did_resolver::{
    did_doc::schema::did_doc::DidDocument,
    did_parser::Did,
    error::GenericError,
    traits::resolvable::{
        resolution_options::DidResolutionOptions, resolution_output::DidResolutionOutput,
        DidResolvable,
    },
};
use error::DidResolverRegistryError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// TODO: Use serde_json::Map instead
pub type GenericMap = HashMap<String, Value>;
pub type GenericResolver = dyn DidResolvableAdaptorTrait + Send + Sync;

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
        options: &DidResolutionOptions<HashMap<String, Value>>,
    ) -> Result<DidResolutionOutput<HashMap<String, Value>>, GenericError>;
}

#[async_trait]
impl<T: DidResolvable + Send + Sync> DidResolvableAdaptorTrait for DidResolvableAdaptor<T>
where
    T::ExtraFieldsService: Send + Sync + Serialize + for<'de> Deserialize<'de>,
    T::ExtraFieldsOptions: Send + Sync + Serialize + for<'de> Deserialize<'de>,
{
    async fn resolve(
        &self,
        did: &Did,
        options: &DidResolutionOptions<HashMap<String, Value>>,
    ) -> Result<DidResolutionOutput<HashMap<String, Value>>, GenericError> {
        let options_inner: T::ExtraFieldsOptions = if options.extra().is_empty() {
            Default::default()
        } else {
            serde_json::from_value(Value::Object(options.extra().clone().into_iter().collect()))?
        };
        let result_inner = self
            .inner
            .resolve(did, &DidResolutionOptions::new(options_inner))
            .await?;

        let did_document_inner_hashmap = serde_json::to_value(result_inner.did_document())
            .unwrap()
            .as_object()
            .unwrap()
            .clone();

        let did_document: DidDocument<HashMap<String, Value>> =
            serde_json::from_value(Value::Object(did_document_inner_hashmap))?;

        Ok(DidResolutionOutput::builder(did_document)
            .did_resolution_metadata(result_inner.did_resolution_metadata().clone())
            .did_document_metadata(result_inner.did_document_metadata().clone())
            .build())
    }
}

impl ResolverRegistry {
    pub fn new() -> Self {
        ResolverRegistry {
            resolvers: HashMap::new(),
        }
    }

    pub fn register_resolver<T>(mut self, method: String, resolver: T) -> Self
    where
        T: DidResolvable + 'static + Send + Sync,
        for<'de> <T as DidResolvable>::ExtraFieldsService:
            Send + Sync + Serialize + Deserialize<'de>,
        for<'de> <T as DidResolvable>::ExtraFieldsOptions:
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
        options: &DidResolutionOptions<GenericMap>,
    ) -> Result<DidResolutionOutput<GenericMap>, GenericError> {
        let method = did
            .method()
            .ok_or(DidResolverRegistryError::UnsupportedMethod)?;
        match self.resolvers.get(method) {
            Some(resolver) => resolver.resolve(did, options).await,
            None => Err(Box::new(DidResolverRegistryError::UnsupportedMethod)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use did_resolver::did_doc::schema::did_doc::DidDocumentBuilder;
    use mockall::{automock, predicate::eq};
    use std::{error::Error, pin::Pin};

    struct DummyDidResolver;

    #[async_trait]
    #[automock]
    impl DidResolvable for DummyDidResolver {
        type ExtraFieldsService = ();
        type ExtraFieldsOptions = ();

        async fn resolve(
            &self,
            did: &Did,
            _options: &DidResolutionOptions<()>,
        ) -> Result<DidResolutionOutput<()>, GenericError> {
            Ok(DidResolutionOutput::builder(
                DidDocumentBuilder::new(Did::parse(did.did().to_string()).unwrap()).build(),
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
        let did = Did::parse("did:example:1234".to_string()).unwrap();
        let method = did.method().unwrap().to_string();

        let mut mock_resolver = MockDummyDidResolver::new();
        mock_resolver
            .expect_resolve()
            .with(eq(did.clone()), eq(DidResolutionOptions::default()))
            .times(1)
            .return_once(move |_, _| {
                let future = async move {
                    Err::<DidResolutionOutput<()>, GenericError>(Box::new(DummyResolverError))
                };
                Pin::from(Box::new(future))
            });

        let registry = ResolverRegistry::new()
            .register_resolver::<MockDummyDidResolver>(method, mock_resolver.into());

        let result = registry
            .resolve(&did, &DidResolutionOptions::default())
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
        let parsed_did = Did::parse(did.to_string()).unwrap();
        let method = parsed_did.method().unwrap().to_string();

        let mut mock_resolver = MockDummyDidResolver::new();
        mock_resolver
            .expect_resolve()
            .with(eq(parsed_did.clone()), eq(DidResolutionOptions::default()))
            .times(1)
            .return_once(move |_, _| {
                let future = async move {
                    Ok::<DidResolutionOutput<()>, GenericError>(
                        DidResolutionOutput::builder(
                            DidDocumentBuilder::new(Did::parse(did.to_string()).unwrap()).build(),
                        )
                        .build(),
                    )
                };
                Pin::from(Box::new(future))
            });

        let registry = ResolverRegistry::new()
            .register_resolver::<MockDummyDidResolver>(method, mock_resolver.into());

        let result = registry
            .resolve(&parsed_did, &DidResolutionOptions::default())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_and_unregister_resolver() {
        let method = "example".to_string();
        let mock_resolver = MockDummyDidResolver::new();

        let mut registry = ResolverRegistry::new();
        assert_eq!(registry.resolvers.len(), 0);

        registry = registry
            .register_resolver::<MockDummyDidResolver>(method.clone(), mock_resolver.into());
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
        let result = registry
            .resolve(&did, &DidResolutionOptions::default())
            .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        if let Some(err) = error.downcast_ref::<DidResolverRegistryError>() {
            assert!(matches!(err, DidResolverRegistryError::UnsupportedMethod));
        } else {
            assert!(false, "Error is not of type DidResolverRegistryError");
        }
    }

    #[tokio::test]
    async fn test_resolve_after_registering_resolver() {
        let did = "did:example:1234";
        let parsed_did = Did::parse(did.to_string()).unwrap();
        let method = parsed_did.method().unwrap().to_string();

        let mut mock_resolver = MockDummyDidResolver::new();
        mock_resolver
            .expect_resolve()
            .with(eq(parsed_did.clone()), eq(DidResolutionOptions::default()))
            .times(1)
            .return_once(move |_, _| {
                let future = async move {
                    Ok::<DidResolutionOutput<()>, GenericError>(
                        DidResolutionOutput::builder(
                            DidDocumentBuilder::new(Did::parse(did.to_string()).unwrap()).build(),
                        )
                        .build(),
                    )
                };
                Pin::from(Box::new(future))
            });

        let mut registry = ResolverRegistry::new();

        let result_before = registry
            .resolve(&parsed_did, &DidResolutionOptions::default())
            .await;
        assert!(result_before.is_err());
        let error_before = result_before.unwrap_err();
        if let Some(err) = error_before.downcast_ref::<DidResolverRegistryError>() {
            assert!(matches!(err, DidResolverRegistryError::UnsupportedMethod));
        } else {
            assert!(false, "Error is not of type DidResolverRegistryError");
        }

        registry = registry.register_resolver::<MockDummyDidResolver>(method, mock_resolver.into());

        let result_after = registry
            .resolve(&parsed_did, &DidResolutionOptions::default())
            .await;
        assert!(result_after.is_ok());
    }
}
