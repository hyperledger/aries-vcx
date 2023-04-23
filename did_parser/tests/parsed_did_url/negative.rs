use did_parser::ParsedDIDUrl;

macro_rules! test_cases_negative {
    ($($name:ident: $input:expr)*) => {
        $(
            #[test]
            fn $name() {
                println!("Testing {:?}", ParsedDIDUrl::parse($input.to_string()));
                assert!(ParsedDIDUrl::parse($input.to_string()).is_err());
            }
        )*
    };
}

test_cases_negative! {
    test_failure_case1: ""
    test_failure_case2: "not-a-did"
    test_failure_case3: "did:example"
    test_failure_case4: "did:example:123456789abcdefghi;param="
    test_failure_case5: "did:example:123456789abcdefghi?query="
    test_failure_case6: "did:example:123456789abcdefghi/path?query1=value1&query2"
    test_failure_case7: "/"
    test_failure_case8: "/?"
    test_failure_case9: "/#"
    test_failure_case10: ";"
    test_failure_case11: ";?"
    test_failure_case12: ";#"
}
