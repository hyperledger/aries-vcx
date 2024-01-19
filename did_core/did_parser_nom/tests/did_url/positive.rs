use std::collections::HashMap;

use did_parser_nom::DidUrl;

macro_rules! test_cases_positive {
    ($($name:ident: $input:expr, $expected_did:expr, $expected_method:expr, $expected_namespace:expr, $expected_id:expr, $expected_path:expr, $expected_fragment:expr, $expected_queries:expr)*) => {
        $(
            #[test]
            fn $name() {
                println!("Testing {}", $input);
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
        Some("namespace"),
        Some("123456789abcdefghi"),
        None,
        None,
        HashMap::new()

    test_case2:
        "did:example:namespace:123456789abcdefghi/path",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        Some("namespace"),
        Some("123456789abcdefghi"),
        Some("/path"),
        None,
        HashMap::new()

    test_case3:
        "did:example:namespace:123456789abcdefghi/path?query1=value1&query2=value2",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
        None,
        Some("fragment"),
        HashMap::new()

    test_case7:
        "did:example:namespace:123456789abcdefghi?query=value",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
        Some("/path"),
        Some("fragment"),
        HashMap::new()

    test_case9:
        "did:example:namespace:123456789abcdefghi#fragment",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        Some("namespace"),
        Some("123456789abcdefghi"),
        None,
        Some("fragment"),
        HashMap::new()

    test_case10:
        "did:example:namespace:123456789abcdefghi?query=value",
        Some("did:example:namespace:123456789abcdefghi"),
        Some("example"),
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
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
        Some("namespace"),
        Some("123456789abcdefghi"),
        None,
        None,
        {
            vec![
                ("query1".to_string(), "val=ue1".to_string()),
            ].into_iter().collect()
        }

    test_case27:
        "did:indy:sovrin:5nDyJVP1NrcPAttP3xwMB9/anoncreds/v0/REV_REG_DEF/56495/npdb/TAG1",
        Some("did:indy:sovrin:5nDyJVP1NrcPAttP3xwMB9"),
        Some("indy"),
        Some("sovrin"),
        Some("5nDyJVP1NrcPAttP3xwMB9"),
        Some("/anoncreds/v0/REV_REG_DEF/56495/npdb/TAG1"),
        None,
        HashMap::new()
}
