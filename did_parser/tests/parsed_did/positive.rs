use did_parser::ParsedDID;

macro_rules! test_cases_positive {
    ($($name:ident: $input:expr, $expected_did:expr, $expected_method:expr, $expected_id:expr)*) => {
        $(
            #[test]
            fn $name() {
                let parsed_did = ParsedDID::parse($input.to_string()).unwrap();

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
        "example",
        "123456789abcdefghi"
}
