use did_parser_nom::Did;

macro_rules! test_cases_negative {
    ($($name:ident: $input:expr)*) => {
        $(
            #[test]
            fn $name() {
                println!("Testing {}", $input);
                assert!(Did::parse($input.to_string()).is_err());
            }
        )*
    };
}

test_cases_negative! {
    test_failure_case1: ""
    test_failure_case2: "not-a-did"
    test_failure_case3: "did:example"
    test_failure_case4: "2ZHFFhtTD6hJqzux"
    test_failure_case5: "did:indy:s@vrin:7Tqg6BwSSWapxgUDm9KKgg"
    test_failure_case6: "did:indy:sovrin:alpha:%0zqg6BwS.Wapxg-Dm9K_gg"
    test_failure_case7: "did:sov:2wJPyULfLLnYTEFYzByf"
    test_failure_case8: "did:sov:2wJPyULfOLnYTEFYzByfUR"
    test_failure_case9: "2wJPyULfLLnYTEFYzByf"
    test_failure_case10: "2wJPyULfOLnYTEFYzByfUR"
    test_failure_case11: "did:key:zWA8Ta6fesJIxeYku6cbA"
}
