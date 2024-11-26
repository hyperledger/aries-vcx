use std::collections::HashMap;

use async_trait::async_trait;
use did_resolver::{
    did_doc::schema::did_doc::DidDocument,
    did_parser_nom::Did,
    error::GenericError,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};
use tokio::sync::Mutex;
use tonic::transport::{Channel, Endpoint};

use crate::{
    error::DidCheqdResult,
    proto::cheqd::{
        did::v2::{query_client::QueryClient as DidQueryClient, QueryDidDocRequest},
        resource::v2::query_client::QueryClient as ResourceQueryClient,
    },
};

const MAINNET_NAMESPACE: &str = "mainnet";
const MAINNET_DEFAULT_GRPC: &str = "https://grpc.cheqd.net:443";
const TESTNET_NAMESPACE: &str = "testnet";
const TESTNET_DEFAULT_GRPC: &str = "https://grpc.cheqd.network:443";

pub struct DidCheqdResolverConfiguration {
    networks: Vec<NetworkConfiguration>,
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

pub struct NetworkConfiguration {
    grpc_url: String,
    namespace: String,
}

impl NetworkConfiguration {
    pub fn mainnet() -> Self {
        Self {
            grpc_url: String::from(MAINNET_DEFAULT_GRPC),
            namespace: String::from(MAINNET_NAMESPACE),
        }
    }

    pub fn testnet() -> Self {
        Self {
            grpc_url: String::from(TESTNET_DEFAULT_GRPC),
            namespace: String::from(TESTNET_NAMESPACE),
        }
    }
}

#[derive(Clone)]
struct CheqdGrpcClient {
    did: DidQueryClient<Channel>,
    _resources: ResourceQueryClient<Channel>,
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
        let doc = self.resolve_did(did).await?;
        Ok(DidResolutionOutput::builder(doc).build())
    }
}

impl DidCheqdResolver {
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
            .unwrap(); // TODO

        // initialize new
        let conn = Endpoint::new(network_config.grpc_url.clone())
            .unwrap() // TODO
            .connect()
            .await
            .unwrap(); // TODO

        let did_client = DidQueryClient::new(conn.clone());
        let resource_client = ResourceQueryClient::new(conn);

        let client = CheqdGrpcClient {
            did: did_client,
            _resources: resource_client,
        };

        lock.insert(network.to_owned(), client.clone());

        Ok(client)
    }

    pub async fn resolve_did(&self, did: &Did) -> DidCheqdResult<DidDocument> {
        let network = did.namespace().unwrap_or(MAINNET_NAMESPACE);
        let mut client = self.client_for_network(network).await.unwrap();
        // TODO - mainnet vs testnet
        // TODO - return doc metadata
        // TODO - doc versions
        let did = did.did().to_owned();

        let request = tonic::Request::new(QueryDidDocRequest { id: did });
        let response = client.did.did_doc(request).await.unwrap();

        let query_response = response.into_inner();
        let query_doc_res = query_response.value.unwrap(); // TODO
        let query_doc = query_doc_res.did_doc.unwrap(); // TODO

        DidDocument::try_from(query_doc)
    }
}
