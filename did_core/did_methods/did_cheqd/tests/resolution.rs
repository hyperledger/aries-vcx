use did_cheqd::resolution::resolver::{DidCheqdResolver, DidCheqdResolverConfiguration};

#[tokio::test]
async fn test_resolve_known_testnet_vector() {
    // let did = "did:cheqd:testnet:BttdoaxtC5JkYJoLeGV8ny".parse().unwrap();
    let did = "did:cheqd:mainnet:e536be60-880a-4d10-bd95-e84d13d7db6d"
        .parse()
        .unwrap();
    dbg!(&did);
    let resolver = DidCheqdResolver::new(DidCheqdResolverConfiguration::default());
    let doc = resolver.resolve_did(&did).await.unwrap();
    dbg!(doc);
}
