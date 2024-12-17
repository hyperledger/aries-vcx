use did_cheqd::resolution::resolver::{DidCheqdResolver, DidCheqdResolverConfiguration};
use did_resolver::traits::resolvable::DidResolvable;
use serde_json::{json, Value};

#[tokio::test]
async fn test_resolve_known_mainnet_did_vector() {
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
async fn test_resolve_known_testnet_did_vector() {
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

#[tokio::test]
async fn test_resolve_known_mainnet_resource_vector() {
    let url = "did:cheqd:mainnet:e18756b4-25e6-42bb-b1e9-ea48cbe3c360/resources/\
               e8af40f9-3df2-40dc-b50d-d1a7e764b52d"
        .parse()
        .unwrap();

    let expected_content = json!({
    "name": "Test cheqd anoncreds",
    "version": "1.0",
    "attrNames": ["test"]
    });
    let expected_meta = json!({
    "alsoKnownAs": [{ "description": "did-url", "uri": "did:cheqd:mainnet:e18756b4-25e6-42bb-b1e9-ea48cbe3c360/resources/e8af40f9-3df2-40dc-b50d-d1a7e764b52d" }],
    "resourceUri": "did:cheqd:mainnet:e18756b4-25e6-42bb-b1e9-ea48cbe3c360/resources/e8af40f9-3df2-40dc-b50d-d1a7e764b52d",
    "resourceCollectionId": "e18756b4-25e6-42bb-b1e9-ea48cbe3c360",
    "resourceId": "e8af40f9-3df2-40dc-b50d-d1a7e764b52d",
    "resourceName": "Test cheqd anoncreds-Schema",
    "resourceType": "anonCredsSchema",
    "mediaType": "application/json",
    "resourceVersion": "1.0",
    "created": "2024-09-26T10:25:07Z",
    "checksum": "01a38743e6f482c998ee8a5b84e1c7e116623a6c9b58c16125eebdf254d24da5"
    });

    let resolver = DidCheqdResolver::new(DidCheqdResolverConfiguration::default());
    let output = resolver.resolve_resource(&url).await.unwrap();
    let json_content: Value = serde_json::from_slice(&output.content).unwrap();
    assert_eq!(json_content, expected_content);
    let json_meta = serde_json::to_value(output.metadata).unwrap();
    assert_eq!(json_meta, expected_meta);
}

#[tokio::test]
async fn test_resolve_known_testnet_resource_query() {
    // https://testnet-explorer.cheqd.io/transactions/222FF2D023C2C9A097BB38F3875F072DF8DEC7B0CBD46AC3459C9B4C3C74382F

    let name = "275990cc056b46176a7122cfd888f46a2bd8e3d45a71d5ff20764a874ed02edd";
    let typ = "anonCredsStatusList";
    let time = "2024-12-04T22:15:20Z";
    let url = format!(
        "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b?resourceName={name}&\
         resourceType={typ}&resourceVersionTime={time}"
    )
    .parse()
    .unwrap();

    let expected_content = json!({
        "currentAccumulator": "21 125DF938B3B772619CB43E561D69004CF09667376E9CD53C818D84860BAE3D1D9 21 11ECFC5F9B469AC74E2A0E329F86C6E60B423A53CAC5AE7A4DBE7A978BFFC0DA1 6 6FAD628FED470FF640BF2C5DB57C2C18D009645DBEF15D4AF710739D2AD93E2D 4 22093A3300411B059B8BB7A8C3296A2ED9C4C8E00106C3B2BAD76E25AC792063 6 71D70ECA81BCE610D1C22CADE688AF4A122C8258E8B306635A111D0A35A7238A 4 1E80F38ABA3A966B8657D722D4E956F076BB2F5CCF36AA8942E65500F8898FF3",
        "revRegDefId": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/4f265d83-4657-4c37-ba80-c66cc399457e",
        "revocationList": [1,1,1,0,0]
    });
    let expected_meta = json!({
    "alsoKnownAs": [{ "description": "did-url", "uri": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/d08596a8-c655-45cd-88d7-ac27e8f7d183" }],
    "resourceUri": "did:cheqd:testnet:8bbd2026-03f5-42c7-bf80-09f46fc4d67b/resources/d08596a8-c655-45cd-88d7-ac27e8f7d183",
    "resourceCollectionId": "8bbd2026-03f5-42c7-bf80-09f46fc4d67b",
    "resourceId": "d08596a8-c655-45cd-88d7-ac27e8f7d183",
    "resourceName": name,
    "resourceType": typ,
    "mediaType": "application/json",
    "resourceVersion": "1669c51f-a382-4a35-a3cc-10f6a278950e",
    "created": "2024-12-04T22:15:18Z",
    "checksum": "0c9b32ad86c21001fb158e0b19ef6ade10f054d8b0a63cc49f12efc46bcd6ce4",
    "nextVersionId": "8e93fa1c-6ee8-4416-8aeb-8ff52cc676ab",
    "previousVersionId": "942f1817-a592-44c2-b5c2-bb6579527da5"
    });

    let resolver = DidCheqdResolver::new(DidCheqdResolverConfiguration::default());
    let output = resolver.resolve_resource(&url).await.unwrap();
    let json_content: Value = serde_json::from_slice(&output.content).unwrap();
    assert_eq!(json_content, expected_content);
    let json_meta = serde_json::to_value(output.metadata).unwrap();
    assert_eq!(json_meta, expected_meta);
}
