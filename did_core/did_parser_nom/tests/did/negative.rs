use did_parser_nom::Did;

macro_rules! test_cases_negative {
    ($($name:ident: $input:expr)*) => {
        $(
            #[test]
            fn $name() {
                assert!(Did::parse($input.to_string()).is_err());
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
    unqalified_invalid_len:
        "2ZHFFhtTD6hJqzux"
    indy_non_method_specific_id_char_in_namespace:
        "did:indy:s@vrin:7Tqg6BwSSWapxgUDm9KKgg"
    indy_multiple_namespaces_invalid_char_in_method_specific_id:
        "did:indy:sovrin:alpha:%0zqg6BwS.Wapxg-Dm9K_gg"
    sov_invalid_len:
        "did:sov:2wJPyULfLLnYTEFYzByf"
    sov_invalid_char:
        "did:sov:2wJPyULfOLnYTEFYzByfUR"
    sov_unqalified_invalid_len:
        "2wJPyULfLLnYTEFYzByf"
    sov_unqalified_invalid_char:
        "2wJPyULfOLnYTEFYzByfUR"
    key_non_mb_value_char:
        "did:key:zWA8Ta6fesJIxeYku6cbA"
    key_non_base58_btc_encoded:
        "did:key:6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
}
