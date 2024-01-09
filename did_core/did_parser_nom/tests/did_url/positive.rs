use std::collections::HashMap;

use did_parser_nom::DidUrl;

macro_rules! test_cases_positive {
    ($($name:ident: $input:expr, $expected_did:expr, $expected_method:expr, $expected_id:expr, $expected_path:expr, $expected_fragment:expr, $expected_queries:expr)*) => {
        $(
            #[test]
            fn $name() {
                println!("Testing {}", $input);
                let parsed_did = DidUrl::parse($input.to_string()).unwrap();

                assert_eq!(parsed_did.did(), $expected_did, "DID");
                assert_eq!(parsed_did.method(), $expected_method, "Method");
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
        "did:example:123456789abcdefghi",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        None,
        None,
        HashMap::new()

    test_case2:
        "did:example:123456789abcdefghi/path",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        Some("/path"),
        None,
        HashMap::new()

    test_case3:
        "did:example:123456789abcdefghi/path?query1=value1&query2=value2",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        Some("/path"),
        None,
        {
            let mut queries = HashMap::new();
            queries.extend(vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "value2".to_string()),
            ]);
            queries
        }

    test_case4:
        "did:example:123456789abcdefghi/path?query=value#fragment",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        Some("/path"),
        Some("fragment"),
        {
            let mut queries = HashMap::new();
            queries.extend(vec![("query".to_string(), "value".to_string())]);
            queries
        }

    test_case5:
        "did:example:123456789abcdefghi?query1=value1&query2=value2#fragment",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        None,
        Some("fragment"),
        {
            let mut queries = HashMap::new();
            queries.extend(vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "value2".to_string())
            ]);
            queries
        }

    test_case6:
        "did:example:123456789abcdefghi#fragment",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        None,
        Some("fragment"),
        HashMap::new()

    test_case7:
        "did:example:123456789abcdefghi?query=value",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        None,
        None,
        {
            let mut queries = HashMap::new();
            queries.insert("query".to_string(), "value".to_string());
            queries
        }

    test_case8:
        "did:example:123456789abcdefghi/path#fragment",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        Some("/path"),
        Some("fragment"),
        HashMap::new()

    test_case9:
        "did:example:123456789abcdefghi#fragment",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        None,
        Some("fragment"),
        HashMap::new()

    test_case10:
        "did:example:123456789abcdefghi?query=value",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        None,
        None,
        {
            let mut queries = HashMap::new();
            queries.extend(vec![("query".to_string(), "value".to_string())]);
            queries
        }

    test_case11:
        "did:example:123456789abcdefghi/path?query=value",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        Some("/path"),
        None,
        {
            let mut query = HashMap::new();
            query.insert("query".to_string(), "value".to_string());
            query
        }

    test_case12:
        "did:example:123456789abcdefghi/path?query1=value1&query2=value2#fragment",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        Some("/path"),
        Some("fragment"),
        {
            let mut queries = HashMap::new();
            queries.extend(vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "value2".to_string()),
            ]);
            queries
        }

    test_case13:
        "did:example:123456789abcdefghi?query=",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        None,
        None,
        {
            let mut queries = HashMap::new();
            queries.extend(vec![
                ("query".to_string(), "".to_string()),
            ]);
            queries
        }

    test_case14:
        "did:example:123456789abcdefghi?query=value&query2=#fragment",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        None,
        Some("fragment"),
        {
            let mut queries = HashMap::new();
            queries.extend(vec![
                ("query".to_string(), "value".to_string()),
                ("query2".to_string(), "".to_string()),
            ]);
            queries
        }

    test_case15:
        "did:example:123456789abcdefghi?query1=value1&query2=value2#fragment",
        Some("did:example:123456789abcdefghi"),
        Some("example"),
        Some("123456789abcdefghi"),
        None,
        Some("fragment"),
        {
            let mut queries = HashMap::new();
            queries.extend(vec![
                ("query1".to_string(), "value1".to_string()),
                ("query2".to_string(), "value2".to_string()),
            ]);
            queries
        }

    test_case16:
        "/path",
        None,
        None,
        None,
        Some("/path"),
        None,
        HashMap::new()

    test_case17:
        "?query=value",
        None,
        None,
        None,
        None,
        None,
        {
            let mut queries = HashMap::new();
            queries.insert("query".to_string(), "value".to_string());
            queries
        }

    test_case18:
        "#fragment",
        None,
        None,
        None,
        None,
        Some("fragment"),
        HashMap::new()

    test_case19:
        "/path?query=value",
        None,
        None,
        None,
        Some("/path"),
        None,
        {
            let mut queries = HashMap::new();
            queries.insert("query".to_string(), "value".to_string());
            queries
        }

    test_case20:
        "/path#fragment",
        None,
        None,
        None,
        Some("/path"),
        Some("fragment"),
        HashMap::new()

    // TODO: How the hell are we suppposed to distinguish a did:web DID URL with path
    // from a namespace
    // test_case21:
    //     "did:web:w3c-ccg.github.io:user:alice",
    //     Some("did:web:w3c-ccg.github.io:user:alice"),
    //     Some("web"),
    //     Some("w3c-ccg.github.io:user:alice"),
    //     None,
    //     None,
    //     HashMap::new()

    test_case22:
        "2ZHFFhzA2XtTD6hJqzL7ux#1",
        Some("2ZHFFhzA2XtTD6hJqzL7ux"),
        None,
        Some("2ZHFFhzA2XtTD6hJqzL7ux"),
        None,
        Some("1"),
        HashMap::new()
}
