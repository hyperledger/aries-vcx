use std::collections::HashMap;

use async_trait::async_trait;
use bytes::Bytes;
use did_resolver::{
    did_doc::schema::did_doc::DidDocument,
    did_parser_nom::Did,
    error::GenericError,
    shared_types::did_document_metadata::DidDocumentMetadata,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};
use http_body_util::combinators::UnsyncBoxBody;
use hyper_tls::HttpsConnector;
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::TokioExecutor,
};
use tokio::sync::Mutex;
use tonic::{transport::Uri, Status};

use crate::{
    error::{DidCheqdError, DidCheqdResult},
    proto::cheqd::{
        did::v2::{query_client::QueryClient as DidQueryClient, QueryDidDocRequest},
        resource::v2::query_client::QueryClient as ResourceQueryClient,
    },
};

/// default namespace for the cheqd "mainnet". as it would appear in a DID.
pub const MAINNET_NAMESPACE: &str = "mainnet";
/// default gRPC URL for the cheqd "mainnet".
pub const MAINNET_DEFAULT_GRPC: &str = "https://grpc.cheqd.net:443";
/// default namespace for the cheqd "testnet". as it would appear in a DID.
pub const TESTNET_NAMESPACE: &str = "testnet";
/// default gRPC URL for the cheqd "testnet".
pub const TESTNET_DEFAULT_GRPC: &str = "https://grpc.cheqd.network:443";

/// Configuration for the [DidCheqdResolver] resolver
pub struct DidCheqdResolverConfiguration {
    /// Configuration for which networks are resolvable
    pub networks: Vec<NetworkConfiguration>,
}

impl Default for DidCheqdResolverConfiguration {
    fn default() -> Self {
        Self {
            networks: vec![
                NetworkConfiguration::mainnet(),
                NetworkConfiguration::testnet(),
            ],
        }
    }
}

/// Configuration for a cheqd network. Defining details such as where to resolve DIDs from.
pub struct NetworkConfiguration {
    /// the cheqd nodes gRPC URL
    pub grpc_url: String,
    /// the namespace of the network - as it would appear in a DID (did:cheqd:namespace:123)
    pub namespace: String,
}

impl NetworkConfiguration {
    /// default configuration for cheqd mainnet
    pub fn mainnet() -> Self {
        Self {
            grpc_url: String::from(MAINNET_DEFAULT_GRPC),
            namespace: String::from(MAINNET_NAMESPACE),
        }
    }

    /// default configuration for cheqd testnet
    pub fn testnet() -> Self {
        Self {
            grpc_url: String::from(TESTNET_DEFAULT_GRPC),
            namespace: String::from(TESTNET_NAMESPACE),
        }
    }
}

type HyperClient = Client<HttpsConnector<HttpConnector>, UnsyncBoxBody<Bytes, Status>>;

#[derive(Clone)]
struct CheqdGrpcClient {
    did: DidQueryClient<HyperClient>,
    // FUTURE - not used yet
    _resources: ResourceQueryClient<HyperClient>,
}

pub struct DidCheqdResolver {
    networks: Vec<NetworkConfiguration>,
    network_clients: Mutex<HashMap<String, CheqdGrpcClient>>,
}

#[async_trait]
impl DidResolvable for DidCheqdResolver {
    type DidResolutionOptions = ();

    async fn resolve(
        &self,
        did: &Did,
        _: &Self::DidResolutionOptions,
    ) -> Result<DidResolutionOutput, GenericError> {
        Ok(self.resolve_did(did).await?)
    }
}

impl DidCheqdResolver {
    /// Assemble a new resolver with the given config.
    ///
    /// [DidCheqdResolverConfiguration::default] can be used if default mainnet & testnet
    /// configurations are suitable.
    pub fn new(configuration: DidCheqdResolverConfiguration) -> Self {
        Self {
            networks: configuration.networks,
            network_clients: Default::default(),
        }
    }

    /// lazily get the client, initializing if not already
    async fn client_for_network(&self, network: &str) -> DidCheqdResult<CheqdGrpcClient> {
        let mut lock = self.network_clients.lock().await;
        if let Some(client) = lock.get(network) {
            return Ok(client.clone());
        }

        let network_config = self
            .networks
            .iter()
            .find(|n| n.namespace == network)
            .ok_or(DidCheqdError::NetworkNotSupported(network.to_owned()))?;

        let client = native_tls_hyper_client()?;
        let origin: Uri = network_config.grpc_url.parse().map_err(|e| {
            DidCheqdError::BadConfiguration(format!(
                "GRPC URL is not a URI: {} {e}",
                network_config.grpc_url
            ))
        })?;

        let did_client = DidQueryClient::with_origin(client.clone(), origin.clone());
        let resource_client = ResourceQueryClient::with_origin(client, origin);

        let client = CheqdGrpcClient {
            did: did_client,
            _resources: resource_client,
        };

        lock.insert(network.to_owned(), client.clone());

        Ok(client)
    }

    /// Resolve a cheqd DID.
    pub async fn resolve_did(&self, did: &Did) -> DidCheqdResult<DidResolutionOutput> {
        let method = did.method();
        if method != Some("cheqd") {
            return Err(DidCheqdError::MethodNotSupported(format!("{method:?}")));
        }

        let network = did.namespace().unwrap_or(MAINNET_NAMESPACE);
        let mut client = self.client_for_network(network).await?;
        let did = did.did().to_owned();

        let request = tonic::Request::new(QueryDidDocRequest { id: did });
        let response = client.did.did_doc(request).await?;

        let query_response = response.into_inner();
        let query_doc_res = query_response.value.ok_or(DidCheqdError::InvalidResponse(
            "DIDDoc query did not return a value".into(),
        ))?;
        let query_doc = query_doc_res.did_doc.ok_or(DidCheqdError::InvalidResponse(
            "DIDDoc query did not return a DIDDoc".into(),
        ))?;

        let mut output_builder = DidResolutionOutput::builder(DidDocument::try_from(query_doc)?);

        if let Some(query_metadata) = query_doc_res.metadata {
            // FUTURE - append linked resources to metadata
            output_builder = output_builder
                .did_document_metadata(DidDocumentMetadata::try_from(query_metadata)?);
        }

        Ok(output_builder.build())
    }
}

/// Assembles a hyper client which:
/// * uses native TLS
/// * supports HTTP2 only (gRPC)
fn native_tls_hyper_client() -> DidCheqdResult<HyperClient> {
    let tls = native_tls::TlsConnector::builder()
        .request_alpns(&["h2"])
        .build()
        .map_err(|e| {
            DidCheqdError::BadConfiguration(format!("Failed to build TlsConnector: {e}"))
        })?;
    let mut http = HttpConnector::new();
    http.enforce_http(false);
    let connector = HttpsConnector::from((http, tls.into()));

    Ok(Client::builder(TokioExecutor::new())
        .http2_only(true)
        .build(connector))
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_fails_if_wrong_method() {
        let did = "did:notcheqd:abc".parse().unwrap();
        let resolver = DidCheqdResolver::new(Default::default());
        let e = resolver.resolve_did(&did).await.unwrap_err();
        assert!(matches!(e, DidCheqdError::MethodNotSupported(_)));
    }

    #[tokio::test]
    async fn test_resolve_fails_if_no_network_config() {
        let did = "did:cheqd:devnet:Ps1ysXP2Ae6GBfxNhNQNKN".parse().unwrap();
        let resolver = DidCheqdResolver::new(Default::default());
        let e = resolver.resolve_did(&did).await.unwrap_err();
        assert!(matches!(e, DidCheqdError::NetworkNotSupported(_)));
    }

    #[tokio::test]
    async fn test_resolve_fails_if_bad_network_uri() {
        let did = "did:cheqd:devnet:Ps1ysXP2Ae6GBfxNhNQNKN".parse().unwrap();
        let config = DidCheqdResolverConfiguration {
            networks: vec![NetworkConfiguration {
                grpc_url: "@baduri://.".into(),
                namespace: "devnet".into(),
            }],
        };

        let resolver = DidCheqdResolver::new(config);
        let e = resolver.resolve_did(&did).await.unwrap_err();
        assert!(matches!(e, DidCheqdError::BadConfiguration(_)));
    }
}
