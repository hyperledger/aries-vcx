use std::{cmp::Ordering, collections::HashMap};

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use did_resolver::{
    did_doc::schema::did_doc::DidDocument,
    did_parser_nom::{Did, DidUrl},
    error::GenericError,
    shared_types::{
        did_document_metadata::DidDocumentMetadata,
        did_resource::{DidResource, DidResourceMetadata},
    },
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

use super::transformer::CheqdResourceMetadataWithUri;
use crate::{
    error::{DidCheqdError, DidCheqdResult},
    proto::cheqd::{
        did::v2::{query_client::QueryClient as DidQueryClient, QueryDidDocRequest},
        resource::v2::{
            query_client::QueryClient as ResourceQueryClient, Metadata as CheqdResourceMetadata,
            QueryCollectionResourcesRequest, QueryResourceRequest,
        },
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
    resources: ResourceQueryClient<HyperClient>,
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
            resources: resource_client,
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

    /// Resolve a cheqd DID resource & associated metadata from the given [DidUrl].
    /// Resolution is done according to the [DID-Linked Resources](https://w3c-ccg.github.io/DID-Linked-Resources/)
    /// specification, however only a subset of query types are supported currently:
    /// * by resource path: `did:example:<unique-identifier>/resources/<unique-identifier>`
    /// * by name & type: `did:cheqd:mainnet:zF7rhDBfUt9d1gJPjx7s1J?resourceName=universityDegree&
    ///   resourceType=anonCredsStatusList`
    /// * by name & type & time:
    ///   `did:cheqd:mainnet:zF7rhDBfUt9d1gJPjx7s1J?resourceName=universityDegree&
    ///   resourceType=anonCredsStatusList&versionTime=2022-08-21T08:40:00Z`
    pub async fn resolve_resource(&self, url: &DidUrl) -> DidCheqdResult<DidResource> {
        let method = url.method();
        if method != Some("cheqd") {
            return Err(DidCheqdError::MethodNotSupported(format!("{method:?}")));
        }

        let network = url.namespace().unwrap_or(MAINNET_NAMESPACE);
        let did_id = url
            .id()
            .ok_or(DidCheqdError::InvalidDidUrl(format!("missing ID {url}")))?;

        // 1. resolve by exact reference: /resources/asdf
        if let Some(path) = url.path() {
            let Some(resource_id) = path.strip_prefix("/resources/") else {
                return Err(DidCheqdError::InvalidDidUrl(format!(
                    "DID Resource URL has a path without `/resources/`: {path}"
                )));
            };

            return self
                .resolve_resource_by_id(did_id, resource_id, network)
                .await;
        }

        // 2. resolve by name & type & time (if any)
        let params = url.queries();
        let resource_name = params.get("resourceName");
        let resource_type = params.get("resourceType");
        let version_time = params.get("resourceVersionTime");

        let (Some(resource_name), Some(resource_type)) = (resource_name, resource_type) else {
            return Err(DidCheqdError::InvalidDidUrl(format!(
                "Resolver can only resolve by exact resource ID or name+type combination {url}"
            )))?;
        };
        // determine desired version_time, either from param, or *now*
        let version_time = match version_time {
            Some(v) => DateTime::parse_from_rfc3339(v)
                .map_err(|e| DidCheqdError::InvalidDidUrl(e.to_string()))?
                .to_utc(),
            None => Utc::now(),
        };

        self.resolve_resource_by_name_type_and_time(
            did_id,
            resource_name,
            resource_type,
            version_time,
            network,
        )
        .await
    }

    /// Resolve a resource from a collection (did_id) and network by an exact id.
    async fn resolve_resource_by_id(
        &self,
        did_id: &str,
        resource_id: &str,
        network: &str,
    ) -> DidCheqdResult<DidResource> {
        let mut client = self.client_for_network(network).await?;

        let request = QueryResourceRequest {
            collection_id: did_id.to_owned(),
            id: resource_id.to_owned(),
        };
        let response = client.resources.resource(request).await?;

        let query_response = response.into_inner();
        let query_response = query_response
            .resource
            .ok_or(DidCheqdError::InvalidResponse(
                "Resource query did not return a value".into(),
            ))?;
        let query_resource = query_response
            .resource
            .ok_or(DidCheqdError::InvalidResponse(
                "Resource query did not return a resource".into(),
            ))?;
        let query_metadata = query_response
            .metadata
            .ok_or(DidCheqdError::InvalidResponse(
                "Resource query did not return metadata".into(),
            ))?;
        let metadata = DidResourceMetadata::try_from(CheqdResourceMetadataWithUri {
            uri: format!(
                "did:cheqd:{network}:{}/resources/{}",
                query_metadata.collection_id, query_metadata.id
            ),
            meta: query_metadata,
        })?;

        Ok(DidResource {
            content: query_resource.data,
            metadata,
        })
    }

    /// Resolve a resource from a given collection (did_id) & network, that has a given name & type,
    /// as of a given time.
    async fn resolve_resource_by_name_type_and_time(
        &self,
        did_id: &str,
        name: &str,
        rtyp: &str,
        time: DateTime<Utc>,
        network: &str,
    ) -> DidCheqdResult<DidResource> {
        let mut client = self.client_for_network(network).await?;

        let response = client
            .resources
            .collection_resources(QueryCollectionResourcesRequest {
                collection_id: did_id.to_owned(),
                // FUTURE - pagination
                pagination: None,
            })
            .await?;

        let query_response = response.into_inner();
        let resources = query_response.resources;
        let mut filtered: Vec<_> =
            filter_resources_by_name_and_type(resources.iter(), name, rtyp).collect();
        filtered.sort_by(|a, b| desc_chronological_sort_resources(a, b));

        let resource_meta = find_resource_just_before_time(filtered.into_iter(), time);

        let Some(meta) = resource_meta else {
            return Err(DidCheqdError::ResourceNotFound(format!(
                "network: {network}, collection: {did_id}, name: {name}, type: {rtyp}, time: \
                 {time}"
            )));
        };

        self.resolve_resource_by_id(did_id, &meta.id, network).await
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

/// Filter for resources which have a matching name and type
fn filter_resources_by_name_and_type<'a>(
    resources: impl Iterator<Item = &'a CheqdResourceMetadata> + 'a,
    name: &'a str,
    rtyp: &'a str,
) -> impl Iterator<Item = &'a CheqdResourceMetadata> + 'a {
    resources.filter(move |r| r.name == name && r.resource_type == rtyp)
}

/// Sort resources chronologically by their created timestamps
fn desc_chronological_sort_resources(
    b: &CheqdResourceMetadata,
    a: &CheqdResourceMetadata,
) -> Ordering {
    let (a_secs, a_ns) = a
        .created
        .map(|v| {
            let v = v.normalized();
            (v.seconds, v.nanos)
        })
        .unwrap_or((0, 0));
    let (b_secs, b_ns) = b
        .created
        .map(|v| {
            let v = v.normalized();
            (v.seconds, v.nanos)
        })
        .unwrap_or((0, 0));

    match a_secs.cmp(&b_secs) {
        Ordering::Equal => a_ns.cmp(&b_ns),
        res => res,
    }
}

/// assuming `resources` is sorted by `.created` time in descending order, find
/// the resource which is closest to `before_time`, but NOT after.
///
/// Returns a reference to this resource if it exists.
///
/// e.g.:
/// resources: [{created: 20}, {created: 15}, {created: 10}, {created: 5}]
/// before_time: 14
/// returns: {created: 10}
///
/// resources: [{created: 20}, {created: 15}, {created: 10}, {created: 5}]
/// before_time: 4
/// returns: None
fn find_resource_just_before_time<'a>(
    resources: impl Iterator<Item = &'a CheqdResourceMetadata>,
    before_time: DateTime<Utc>,
) -> Option<&'a CheqdResourceMetadata> {
    let before_epoch = before_time.timestamp();

    for r in resources {
        let Some(created) = r.created else {
            continue;
        };

        let created_epoch = created.normalized().seconds;
        if created_epoch < before_epoch {
            return Some(r);
        }
    }

    None
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

    #[tokio::test]
    async fn test_resolve_resource_fails_if_wrong_method() {
        let url = "did:notcheqd:zF7rhDBfUt9d1gJPjx7s1J/resources/123"
            .parse()
            .unwrap();
        let resolver = DidCheqdResolver::new(Default::default());
        let e = resolver.resolve_resource(&url).await.unwrap_err();
        assert!(matches!(e, DidCheqdError::MethodNotSupported(_)));
    }

    #[tokio::test]
    async fn test_resolve_resource_fails_if_wrong_path() {
        let url = "did:cheqd:mainnet:zF7rhDBfUt9d1gJPjx7s1J/resource/123"
            .parse()
            .unwrap();
        let resolver = DidCheqdResolver::new(Default::default());
        let e = resolver.resolve_resource(&url).await.unwrap_err();
        assert!(matches!(e, DidCheqdError::InvalidDidUrl(_)));
    }

    #[tokio::test]
    async fn test_resolve_resource_fails_if_no_query() {
        let url = "did:cheqd:mainnet:zF7rhDBfUt9d1gJPjx7s1J".parse().unwrap();
        let resolver = DidCheqdResolver::new(Default::default());
        let e = resolver.resolve_resource(&url).await.unwrap_err();
        assert!(matches!(e, DidCheqdError::InvalidDidUrl(_)));
    }

    #[tokio::test]
    async fn test_resolve_resource_fails_if_incomplete_query() {
        let url = "did:cheqd:mainnet:zF7rhDBfUt9d1gJPjx7s1J?resourceName=asdf"
            .parse()
            .unwrap();
        let resolver = DidCheqdResolver::new(Default::default());
        let e = resolver.resolve_resource(&url).await.unwrap_err();
        assert!(matches!(e, DidCheqdError::InvalidDidUrl(_)));
    }

    #[tokio::test]
    async fn test_resolve_resource_fails_if_invalid_resource_time() {
        // use epoch instead of XML DateTime
        let url = "did:cheqd:mainnet:zF7rhDBfUt9d1gJPjx7s1J?resourceName=asdf&resourceType=fdsa&\
                   resourceVersionTime=12341234"
            .parse()
            .unwrap();
        let resolver = DidCheqdResolver::new(Default::default());
        let e = resolver.resolve_resource(&url).await.unwrap_err();
        assert!(matches!(e, DidCheqdError::InvalidDidUrl(_)));
    }
}
