use did_parser_nom::DidUrl;

macro_rules! test_cases_negative {
    ($($name:ident: $input:expr)*) => {
        $(
            #[test]
            fn $name() {
                println!("Testing {}", $input);
                assert!(DidUrl::parse($input.to_string()).is_err());
            }
        )*
    };
}

test_cases_negative! {
    test_failure_case1: ""
    test_failure_case2: "not-a-did"
    test_failure_case3: "did:example"
    test_failure_case4: "did:example:123456789abcdefghi/path?query1=value1&query2"
    test_failure_case5: "/"
    test_failure_case6: "/?"
    test_failure_case7: "/#"
    test_failure_case8: ";"
    test_failure_case9: ";?"
    test_failure_case10: ";#"
    test_failure_case11: "did:example:123456789abcdefghi#fragment1#fragment2"
    test_failure_case12: "did:example:123456789abcdefghi&query1=value1"
    test_failure_case13: "did:example:123456789abcdefghi?query1=v^lue1"
    test_failure_case14: "did:example:123456789abcdefghi#fr^gment"
}
