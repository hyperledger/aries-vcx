use std::convert::Infallible;

use did_resolver::{
    did_doc::schema::did_doc::DidDocument,
    did_parser_nom::Did,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};
use did_resolver_web::resolution::resolver::DidWebResolver;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{
    body::{Bytes, Incoming},
    service::service_fn,
    Request, Response,
};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use tokio::{net::TcpListener, task::JoinSet};
use tokio_test::assert_ok;

const DID_DOCUMENT: &str = r#"
{
  "@context": [
    "https://www.w3.org/ns/did/v1",
    "https://w3id.org/security/suites/jws-2020/v1"
  ],
  "id": "did:web:example.com",
  "verificationMethod": [
    {
      "id": "did:web:example.com#key-0",
      "type": "JsonWebKey2020",
      "controller": "did:web:example.com",
      "publicKeyJwk": {
        "kty": "OKP",
        "crv": "Ed25519",
        "x": "0-e2i2_Ua1S5HbTYnVB0lj2Z2ytXu2-tYmDFf8f5NjU"
      }
    },
    {
      "id": "did:web:example.com#key-1",
      "type": "JsonWebKey2020",
      "controller": "did:web:example.com",
      "publicKeyJwk": {
        "kty": "OKP",
        "crv": "X25519",
        "x": "9GXjPGGvmRq9F6Ng5dQQ_s31mfhxrcNZxRGONrmH30k"
      }
    },
    {
      "id": "did:web:example.com#key-2",
      "type": "JsonWebKey2020",
      "controller": "did:web:example.com",
      "publicKeyJwk": {
        "kty": "EC",
        "crv": "P-256",
        "x": "38M1FDts7Oea7urmseiugGW7tWc3mLpJh6rKe7xINZ8",
        "y": "nDQW6XZ7b_u2Sy9slofYLlG03sOEoug3I0aAPQ0exs4"
      }
    }
  ],
  "authentication": [
    "did:web:example.com#key-0",
    "did:web:example.com#key-2"
  ],
  "assertionMethod": [
    "did:web:example.com#key-0",
    "did:web:example.com#key-2"
  ],
  "keyAgreement": [
    "did:web:example.com#key-1",
    "did:web:example.com#key-2"
  ]
}"#;

async fn mock_server_handler(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    let response = match req.uri().path() {
        "/.well-known/did.json" | "/user/alice/did.json" => {
            Response::new(Full::new(Bytes::from(DID_DOCUMENT)).boxed())
        }
        _ => Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("Not Found")).boxed())
            .unwrap(),
    };

    Ok(response)
}

async fn create_mock_server(port: u16) -> String {
    let listen_addr = format!("127.0.0.1:{port}");
    let tcp_listener = TcpListener::bind(listen_addr).await.unwrap();

    tokio::spawn(async move {
        let mut join_set = JoinSet::new();
        loop {
            let (stream, addr) = match tcp_listener.accept().await {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("failed to accept connection: {e}");
                    continue;
                }
            };

            let serve_connection = async move {
                println!("handling a request from {addr}");

                let result = Builder::new(TokioExecutor::new())
                    .serve_connection(TokioIo::new(stream), service_fn(mock_server_handler))
                    .await;

                if let Err(e) = result {
                    eprintln!("error serving {addr}: {e}");
                }

                println!("handled a request from {addr}");
            };

            join_set.spawn(serve_connection);
        }
    });

    "localhost".to_string()
}

#[tokio::test]
async fn test_did_web_resolver() {
    fn verify_did_document(did_document: &DidDocument) {
        assert_eq!(
            did_document.id().to_string(),
            "did:web:example.com".to_string()
        );
        assert_eq!(did_document.verification_method().len(), 3);
        assert_eq!(did_document.authentication().len(), 2);
        assert_eq!(did_document.assertion_method().len(), 2);
        assert_eq!(did_document.key_agreement().len(), 2);
    }

    let port = 3000;
    let host = create_mock_server(port).await;

    let did_web_resolver = DidWebResolver::http();

    let did_example_1 = Did::parse(format!("did:web:{}%3A{}", host, port)).unwrap();
    let did_example_2 = Did::parse(format!("did:web:{}%3A{}:user:alice", host, port)).unwrap();

    let DidResolutionOutput {
        did_document: ddo1, ..
    } = assert_ok!(did_web_resolver.resolve(&did_example_1, &()).await);
    verify_did_document(&ddo1);

    let DidResolutionOutput {
        did_document: ddo2, ..
    } = assert_ok!(did_web_resolver.resolve(&did_example_2, &()).await);
    verify_did_document(&ddo2);
}
