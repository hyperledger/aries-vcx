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
    unqualified_invalid_len:
        "2ZHFFhtTD6hJqzux"
    indy_non_method_specific_id_char_in_namespace:
        "did:indy:s@vrin:7Tqg6BwSSWapxgUDm9KKgg"
    indy_multiple_namespaces_invalid_char_in_method_specific_id:
        "did:indy:sovrin:alpha:%0zqg6BwS.Wapxg-Dm9K_gg"
    sov_invalid_len:
        "did:sov:2wJPyULfLLnYTEFYzByf"
    sov_invalid_char:
        "did:sov:2wJPyULfOLnYTEFYzByfUR"
    sov_unqualified_invalid_len:
        "2wJPyULfLLnYTEFYzByf"
    sov_unqualified_invalid_char:
        "2wJPyULfOLnYTEFYzByfUR"
    key_non_mb_value_char:
        "did:key:zWA8Ta6fesJIxeYku6cbA"
    key_non_base58_btc_encoded:
        "did:key:6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
    cheqd_no_namespace_did:
        "did:cheqd:de9786cd-ec53-458c-857c-9342cf264f80"
    cheqd_empty_namespace_did:
        "did:cheqd::de9786cd-ec53-458c-857c-9342cf264f80"
    cheqd_sub_namespace_did:
        "did:cheqd:mainnet:foo:de9786cd-ec53-458c-857c-9342cf264f80"
    cheqd_invalid_namespace_character:
        "did:cheqd:m@innet:de9786cd-ec53-458c-857c-9342cf264f80"
    cheqd_short_indy_style_id:
        "did:cheqd:mainnet:TAwT8WVt3dz2DBAifwuS"
    cheqd_long_indy_style_id:
        "did:cheqd:mainnet:TAwT8WVt3dz2DBAifwuSknT"
    cheqd_non_base58_indy_style_char:
        "did:cheqd:mainnet:TAwT8WVt0dz2DBAifwuSkn"
    cheqd_invalid_uuid_style_id_1:
        "did:cheqd:mainnet:de9786cd-ec53-458c-857c-9342cf264f8"
    cheqd_invalid_uuid_style_id_2:
        "did:cheqd:mainnet:de9786cd-ec53-458c-857c9342cf264f80"
    cheqd_invalid_uuid_style_id_3:
        "did:cheqd:mainnet:de9786cd-ec53-458c-857c9342cf2-64f80"
    cheqd_non_alpha_uuid_style_char:
        "did:cheqd:mainnet:qe9786cd-ec53-458c-857c-9342cf264f80"
}
