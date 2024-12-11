use did_parser_nom::DidUrl;

macro_rules! test_cases_negative {
    ($($name:ident: $input:expr)*) => {
        $(
            #[test]
            fn $name() {
                assert!(DidUrl::parse($input.to_string()).is_err());
            }
        )*
    };
}

test_cases_negative! {
    empty:
        ""
    random_string:
        "not-a-did"
    no_method_specific_id:
        "did:example"
    no_equal_sign_after_last_query:
        "did:example:123456789abcdefghi/path?query1=value1&query2"
    no_equal_sign_after_query:
        "did:example:123456789abcdefghi/path?query1"
    fragment_doubled:
        "did:example:123456789abcdefghi#fragment1#fragment2"
    fragment_invalid_char:
        "did:example:123456789abcdefghi#fr^gment"
    query_doubled:
        "did:example:123456789abcdefghi/path??"
    query_not_delimited:
        "did:example:123456789abcdefghi&query1=value1"
    query_invalid_char:
        "did:example:123456789abcdefghi?query1=v^lue1"
    query_unfinished_pct_encoding:
        "did:example:123456789?query=a%3&query2=b"
    query_invalid_space_char:
        "did:example:123456789?query=a b"
    relative_empty_path: "/"
    relative_empty_path_and_query: "/?"
    relative_empty_path_and_fragment: "/#"
    relative_semicolon: ";"
    relative_semicolon_query: ";?"
    relative_semicolon_fragment: ";#"
}
