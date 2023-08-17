use did_parser::Did;

macro_rules! test_cases_positive {
    ($($name:ident: $input:expr, $expected_did:expr, $expected_method:expr, $expected_id:expr)*) => {
        $(
            #[test]
            fn $name() {
                println!("Testing {}", $input);
                let parsed_did = Did::parse($input.to_string()).unwrap();

                assert_eq!(parsed_did.did(), $expected_did, "DID");
                assert_eq!(parsed_did.method(), $expected_method, "Method");
                assert_eq!(parsed_did.id(), $expected_id, "ID");
            }
        )*
    };
}

test_cases_positive! {
    test_case1:
        "did:example:123456789abcdefghi",
        "did:example:123456789abcdefghi",
        Some("example"),
        "123456789abcdefghi"
    test_case2:
        "did:web:w3c-ccg.github.io",
        "did:web:w3c-ccg.github.io",
        Some("web"),
        "w3c-ccg.github.io"
    test_case3:
        "2ZHFFhzA2XtTD6hJqzL7ux",
        "2ZHFFhzA2XtTD6hJqzL7ux",
        None,
        "2ZHFFhzA2XtTD6hJqzL7ux"
}
