use std::collections::HashMap;

use did_parser_nom::DidUrl;

macro_rules! test_cases_positive {
    ($($name:ident: $input:expr, $expected_did:expr, $expected_method:expr, $expected_namespace:expr, $expected_id:expr, $expected_path:expr, $expected_fragment:expr, $expected_queries:expr)*) => {
        $(
            #[test]
            fn $name() {
                let parsed_did = DidUrl::parse($input.to_string()).unwrap();

                assert_eq!(parsed_did.did(), $expected_did, "DID");
                assert_eq!(parsed_did.method(), $expected_method, "Method");
                assert_eq!(parsed_did.namespace(), $expected_namespace, "Namespace");
                assert_eq!(parsed_did.id(), $expected_id, "ID");
                assert_eq!(parsed_did.path(), $expected_path, "Path");
                assert_eq!(parsed_did.fragment(), $expected_fragment, "Fragment");
                assert_eq!(parsed_did.queries(), $expected_queries, "Queries");
            }
        )*
    };
}

test_cases_positive! {
    test_case1:
        "did:example:namespace:123456789abcdefghi",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        None,
        HashMap::new()

    test_case2:
        "did:example:namespace:123456789abcdefghi/path",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        Some("/path"),
        None,
        HashMap::new()

    test_case3:
        "did:example:namespace:123456789abcdefghi/path?query1=value1&query2=value2",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        Some("/path"),
        None,
        {
            vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "value2".to_string()),
            ].into_iter().collect()
        }

    test_case4:
        "did:example:namespace:123456789abcdefghi/path?query=value#fragment",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        Some("/path"),
        Some("fragment"),
        {
            vec![
                ("query".to_string(), "value".to_string()),
            ].into_iter().collect()
        }

    test_case5:
        "did:example:namespace:123456789abcdefghi?query1=value1&query2=value2#fragment",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        Some("fragment"),
        {
            vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "value2".to_string()),
            ].into_iter().collect()
        }

    test_case6:
        "did:example:namespace:123456789abcdefghi#fragment",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        Some("fragment"),
        HashMap::new()

    test_case7:
        "did:example:namespace:123456789abcdefghi?query=value",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        None,
        {
            vec![
                ("query".to_string(), "value".to_string()),
            ].into_iter().collect()
        }

    test_case8:
        "did:example:namespace:123456789abcdefghi/path#fragment",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        Some("/path"),
        Some("fragment"),
        HashMap::new()

    test_case9:
        "did:example:namespace:123456789abcdefghi#fragment",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        Some("fragment"),
        HashMap::new()

    test_case10:
        "did:example:namespace:123456789abcdefghi?query=value",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        None,
        {
            let mut queries = HashMap::new();
            queries.extend(vec![("query".to_string(), "value".to_string())]);
            queries
        }

    test_case11:
        "did:example:namespace:123456789abcdefghi/path?query=value",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        Some("/path"),
        None,
        {
            vec![
                ("query".to_string(), "value".to_string()),
            ].into_iter().collect()
        }

    test_case12:
        "did:example:namespace:123456789abcdefghi/path?query1=value1&query2=value2#fragment",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        Some("/path"),
        Some("fragment"),
        {
            vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "value2".to_string()),
            ].into_iter().collect()
        }

    test_case13:
        "did:example:namespace:123456789abcdefghi?query1=value1?query2=value2",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        None,
        {
            vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "value2".to_string()),
            ].into_iter().collect()
        }

    test_case14:
        "did:example:namespace:123456789abcdefghi?query=",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        None,
        {
            vec![
                ("query".to_string(), "".to_string()),
            ].into_iter().collect()
        }

    test_case15:
        "did:example:namespace:123456789abcdefghi?query1=value1&query2=#fragment",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        Some("fragment"),
        {
            vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "".to_string()),
            ].into_iter().collect()
        }

    test_case16:
        "did:example:namespace:123456789abcdefghi?query1=value1&query2=value2#fragment",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        Some("fragment"),
        {
            vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "value2".to_string()),
            ].into_iter().collect()
        }

    test_case17:
        "/path",
        None,
        None,
        None,
        None,
        Some("/path"),
        None,
        HashMap::new()

    test_case18:
        "?query=value",
        None,
        None,
        None,
        None,
        None,
        None,
        {
            vec![
                ("query".to_string(), "value".to_string()),
            ].into_iter().collect()
        }

    test_case19:
        "#fragment",
        None,
        None,
        None,
        None,
        None,
        Some("fragment"),
        HashMap::new()

    test_case20:
        "/path?query=value",
        None,
        None,
        None,
        None,
        Some("/path"),
        None,
        {
            vec![
                ("query".to_string(), "value".to_string()),
            ].into_iter().collect()
        }

    test_case21:
        "/path#fragment",
        None,
        None,
        None,
        None,
        Some("/path"),
        Some("fragment"),
        HashMap::new()

    test_case22:
        "did:web:w3c-ccg.github.io:user:alice",
        Some("did:web:w3c-ccg.github.io:user:alice"),
        Some("web"),
        None,
        Some("w3c-ccg.github.io:user:alice"),
        None,
        None,
        HashMap::new()

    test_case23:
        "2ZHFFhzA2XtTD6hJqzL7ux#1",
        Some("2ZHFFhzA2XtTD6hJqzL7ux"),
        None,
        None,
        Some("2ZHFFhzA2XtTD6hJqzL7ux"),
        None,
        Some("1"),
        HashMap::new()

    test_case24:
        "did:example:namespace:123456789abcdefghi?query1=val;ue1",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        None,
        {
            vec![
                ("query1".to_string(), "val;ue1".to_string()),
            ].into_iter().collect()
        }

    test_case25:
        "did:example:namespace:123456789abcdefghi?quer;y1=value1",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        None,
        {
            vec![
                ("quer;y1".to_string(), "value1".to_string()),
            ].into_iter().collect()
        }

    test_case26:
        "did:example:namespace:123456789abcdefghi?query1=val=ue1",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        None,
        Some("namespace:123456789abcdefghi"),
        None,
        None,
        {
            vec![
                ("query1".to_string(), "val=ue1".to_string()),
            ].into_iter().collect()
        }

    test_case27:
        "did:sov:5nDyJVP1NrcPAttP3xwMB9/anoncreds/v0/REV_REG_DEF/56495/npdb/TAG1",
        Some("did:sov:5nDyJVP1NrcPAttP3xwMB9"),
        Some("sov"),
        None,
        Some("5nDyJVP1NrcPAttP3xwMB9"),
        Some("/anoncreds/v0/REV_REG_DEF/56495/npdb/TAG1"),
        None,
        HashMap::new()
    test_case28:
        "did:cheqd:testnet:d8ac0372-0d4b-413e-8ef5-8e8f07822b2c/resources/40829caf-b415-4b1d-91a3-b56dfb6374f4",
        Some("did:cheqd:testnet:d8ac0372-0d4b-413e-8ef5-8e8f07822b2c"),
        Some("cheqd"),
        Some("testnet"),
        Some("d8ac0372-0d4b-413e-8ef5-8e8f07822b2c"),
        Some("/resources/40829caf-b415-4b1d-91a3-b56dfb6374f4"),
        None,
        HashMap::new()
    test_case29:
        "did:cheqd:mainnet:zF7rhDBfUt9d1gJPjx7s1J?resourceName=universityDegree&resourceType=anonCredsCredDef",
        Some("did:cheqd:mainnet:zF7rhDBfUt9d1gJPjx7s1J"),
        Some("cheqd"),
        Some("mainnet"),
        Some("zF7rhDBfUt9d1gJPjx7s1J"),
        None,
        None,
        {
            vec![
                ("resourceName".to_string(), "universityDegree".to_string()),
                ("resourceType".to_string(), "anonCredsCredDef".to_string()),
            ].into_iter().collect()
        }
    test_case30:
        "did:cheqd:testnet:36e695a3-f133-46ec-ac1e-79900a927f67?resourceType=anonCredsStatusList&resourceName=Example+schema-default-0&resourceVersionTime=2024-12-10T04%3A13%3A50.000Z",
        Some("did:cheqd:testnet:36e695a3-f133-46ec-ac1e-79900a927f67"),
        Some("cheqd"),
        Some("testnet"),
        Some("36e695a3-f133-46ec-ac1e-79900a927f67"),
        None,
        None,
        {
            vec![
                ("resourceName".to_string(), "Example schema-default-0".to_string()),
                ("resourceType".to_string(), "anonCredsStatusList".to_string()),
                ("resourceVersionTime".to_string(), "2024-12-10T04:13:50.000Z".to_string()),
            ].into_iter().collect()
        }
    test_case31:
        "did:example:123?foo+bar=123&bar%20foo=123%20123&h3%21%210%20=w%40rld%3D%3D",
        Some("did:example:123"),
        Some("example"),
        None,
        Some("123"),
        None,
        None,
        {
            vec![
                ("foo bar".to_string(), "123".to_string()),
                ("bar foo".to_string(), "123 123".to_string()),
                ("h3!!0 ".to_string(), "w@rld==".to_string())
            ].into_iter().collect()
        }
}
