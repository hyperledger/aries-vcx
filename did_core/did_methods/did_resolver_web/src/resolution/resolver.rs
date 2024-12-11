use async_trait::async_trait;
use did_resolver::{
    did_parser_nom::Did,
    error::GenericError,
    shared_types::did_document_metadata::DidDocumentMetadata,
    traits::resolvable::{
        resolution_metadata::DidResolutionMetadata, resolution_output::DidResolutionOutput,
        DidResolvable,
    },
};
use http_body_util::{combinators::BoxBody, BodyExt as _};
use hyper::{
    body::Bytes,
    http::uri::{self, Scheme},
    Uri,
};
use hyper_tls::HttpsConnector;
use hyper_util::{
    client::legacy::{
        connect::{Connect, HttpConnector},
        Client,
    },
    rt::TokioExecutor,
};

use crate::error::DidWebError;

pub struct DidWebResolver<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    client: Client<C, BoxBody<Bytes, GenericError>>,
    scheme: Scheme,
}

impl DidWebResolver<HttpConnector> {
    pub fn http() -> DidWebResolver<HttpConnector> {
        DidWebResolver {
            client: Client::builder(TokioExecutor::new())
                .build::<_, BoxBody<Bytes, GenericError>>(HttpConnector::new()),
            scheme: Scheme::HTTP,
        }
    }
}

impl DidWebResolver<HttpsConnector<HttpConnector>> {
    pub fn https() -> DidWebResolver<HttpsConnector<HttpConnector>> {
        DidWebResolver {
            client: Client::builder(TokioExecutor::new())
                .build::<_, BoxBody<Bytes, GenericError>>(HttpsConnector::new()),
            scheme: Scheme::HTTPS,
        }
    }
}

impl<C> DidWebResolver<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    async fn fetch_did_document(&self, url: Uri) -> Result<String, DidWebError> {
        let res = self.client.get(url).await?;

        if !res.status().is_success() {
            return Err(DidWebError::NonSuccessResponse(res.status()));
        }

        let body = res.into_body().collect().await?.to_bytes();

        String::from_utf8(body.to_vec()).map_err(|err| err.into())
    }
}

#[async_trait]
impl<C> DidResolvable for DidWebResolver<C>
where
    C: Connect + Send + Sync + Clone + 'static,
{
    type DidResolutionOptions = ();

    async fn resolve(
        &self,
        did: &Did,
        _options: &Self::DidResolutionOptions,
    ) -> Result<DidResolutionOutput, GenericError> {
        let method = did.method().ok_or_else(|| {
            DidWebError::InvalidDid("Attempted to resolve unqualified did".to_string())
        })?;
        if method != "web" {
            return Err(Box::new(DidWebError::MethodNotSupported(
                method.to_string(),
            )));
        }

        let did_parts: Vec<&str> = did.id().split(':').collect();

        if did_parts.is_empty() {
            return Err(Box::new(DidWebError::InvalidDid(did.id().to_string())));
        }

        let domain = did_parts[0].replace("%3A", ":");

        let path_parts = &did_parts[1..];
        let path_and_query = if path_parts.is_empty() {
            "/.well-known/did.json".to_string()
        } else {
            let path = path_parts.join("/");
            format!("/{}/did.json", path)
        };
        let url = uri::Builder::new()
            .scheme(self.scheme.clone())
            .authority(domain.as_str())
            .path_and_query(path_and_query.as_str())
            .build()?;

        let did_document = serde_json::from_str(&self.fetch_did_document(url).await?)?;

        let did_resolution_output = DidResolutionOutput::builder(did_document)
            .did_resolution_metadata(DidResolutionMetadata::default())
            .did_document_metadata(DidDocumentMetadata::default())
            .build();

        Ok(did_resolution_output)
    }
}
