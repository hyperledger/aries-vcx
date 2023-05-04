use did_parser::ParsedDid;

macro_rules! test_cases_negative {
    ($($name:ident: $input:expr)*) => {
        $(
            #[test]
            fn $name() {
                println!("Testing {}", $input);
                assert!(ParsedDid::parse($input.to_string()).is_err());
            }
        )*
    };
}

test_cases_negative! {
    test_failure_case1: ""
    test_failure_case2: "not-a-did"
    test_failure_case3: "did:example"
}
