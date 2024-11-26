use did_cheqd::resolution::resolver::{DidCheqdResolver, DidCheqdResolverConfiguration};
use did_resolver::traits::resolvable::DidResolvable;
use serde_json::json;

#[tokio::test]
async fn test_resolve_known_mainnet_vector() {
    // sample from https://dev.uniresolver.io/
    let did = "did:cheqd:mainnet:Ps1ysXP2Ae6GBfxNhNQNKN".parse().unwrap();
    // NOTE: modifications from uni-resolver:
    // make serviceEndpoints into single item (not array)
    let expected_doc = json!({
      "@context": [
        "https://www.w3.org/ns/did/v1",
        "https://w3id.org/security/suites/ed25519-2020/v1"
      ],
      "id": "did:cheqd:mainnet:Ps1ysXP2Ae6GBfxNhNQNKN",
      "verificationMethod": [
        {
          "id": "did:cheqd:mainnet:Ps1ysXP2Ae6GBfxNhNQNKN#key1",
          "type": "Ed25519VerificationKey2020",
          "controller": "did:cheqd:mainnet:Ps1ysXP2Ae6GBfxNhNQNKN",
          "publicKeyMultibase": "z6Mkta7joRuvDh7UnoESdgpr9dDUMh5LvdoECDi3WGrJoscA"
        }
      ],
      "authentication": [
        "did:cheqd:mainnet:Ps1ysXP2Ae6GBfxNhNQNKN#key1"
      ],
      "service": [
        {
          "id": "did:cheqd:mainnet:Ps1ysXP2Ae6GBfxNhNQNKN#website",
          "type": "LinkedDomains",
          "serviceEndpoint": "https://www.cheqd.io/"
        },
        {
          "id": "did:cheqd:mainnet:Ps1ysXP2Ae6GBfxNhNQNKN#non-fungible-image",
          "type": "LinkedDomains",
          "serviceEndpoint": "https://gateway.ipfs.io/ipfs/bafybeihetj2ng3d74k7t754atv2s5dk76pcqtvxls6dntef3xa6rax25xe"
        },
        {
          "id": "did:cheqd:mainnet:Ps1ysXP2Ae6GBfxNhNQNKN#twitter",
          "type": "LinkedDomains",
          "serviceEndpoint": "https://twitter.com/cheqd_io"
        },
        {
          "id": "did:cheqd:mainnet:Ps1ysXP2Ae6GBfxNhNQNKN#linkedin",
          "type": "LinkedDomains",
          "serviceEndpoint": "https://www.linkedin.com/company/cheqd-identity/"
        }
      ]
    });

    let resolver = DidCheqdResolver::new(DidCheqdResolverConfiguration::default());
    let output = resolver.resolve(&did, &()).await.unwrap();
    let doc = output.did_document;
    assert_eq!(serde_json::to_value(doc.clone()).unwrap(), expected_doc);
    assert_eq!(doc, serde_json::from_value(expected_doc).unwrap());
}

#[tokio::test]
async fn test_resolve_known_testnet_vector() {
    // sample from https://dev.uniresolver.io/
    let did = "did:cheqd:testnet:55dbc8bf-fba3-4117-855c-1e0dc1d3bb47"
        .parse()
        .unwrap();
    // NOTE: modifications from uni-resolver:
    // * made controller a single item
    let expected_doc = json!({
      "@context": [
        "https://www.w3.org/ns/did/v1",
        "https://w3id.org/security/suites/ed25519-2020/v1"
      ],
      "id": "did:cheqd:testnet:55dbc8bf-fba3-4117-855c-1e0dc1d3bb47",
      "controller": "did:cheqd:testnet:55dbc8bf-fba3-4117-855c-1e0dc1d3bb47",
      "verificationMethod": [
        {
          "id": "did:cheqd:testnet:55dbc8bf-fba3-4117-855c-1e0dc1d3bb47#key-1",
          "type": "Ed25519VerificationKey2020",
          "controller": "did:cheqd:testnet:55dbc8bf-fba3-4117-855c-1e0dc1d3bb47",
          "publicKeyMultibase": "z6MkkVbyHJLLjdjU5B62DaJ4mkdMdUkttf9UqySSkA9bVTeZ"
        }
      ],
      "authentication": [
        "did:cheqd:testnet:55dbc8bf-fba3-4117-855c-1e0dc1d3bb47#key-1"
      ]
    });

    let resolver = DidCheqdResolver::new(DidCheqdResolverConfiguration::default());
    let output = resolver.resolve(&did, &()).await.unwrap();
    let doc = output.did_document;
    assert_eq!(serde_json::to_value(doc.clone()).unwrap(), expected_doc);
    assert_eq!(doc, serde_json::from_value(expected_doc).unwrap());
}
