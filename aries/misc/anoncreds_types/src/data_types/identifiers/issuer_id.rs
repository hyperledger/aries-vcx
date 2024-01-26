use crate::impl_anoncreds_object_identifier;

impl_anoncreds_object_identifier!(IssuerId);

#[cfg(test)]
mod test_issuer_identifiers {
    use super::*;

    #[test]
    fn should_validate_new_and_legacy_identifiers() {
        let valid_uri_identifier_1 = "did:uri:new";
        let valid_uri_identifier_2 = "did:indy:idunion:test:2MZYuPv2Km7Q1eD4GCsSb6";
        let valid_uri_identifier_3 = "did:indy:sovrin:staging:6cgbu8ZPoWTnR5Rv5JcSMB";
        let valid_uri_identifier_4 = "did:indy:sovrin:7Tqg6BwSSWapxgUDm9KKgg";
        let valid_uri_identifier_5 = "did:web:example.com#controller";
        let valid_uri_identifier_6 = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let invalid_uri_identifier = "::::";

        let valid_legacy_identifier_1 = "NcYxiDXkpYi6ov5FcYDi1e";
        let valid_legacy_identifier_2 = "VsKV7grR1BUE29mG2Fm2kX";

        let too_short_legacy_identifier = "abc";
        let illegal_base58_legacy_identifier_zero = "0000000000000000000000";
        let illegal_base58_legacy_identifier_captial_o = "OOOOOOOOOOOOOOOOOOOOOO";
        let illegal_base58_legacy_identifier_captial_i = "IIIIIIIIIIIIIIIIIIIIII";
        let illegal_base58_legacy_identifier_lower_l = "llllllllllllllllllllll";

        // Instantiating a new IssuerId validates it
        assert!(IssuerId::new(valid_uri_identifier_1).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_2).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_3).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_4).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_5).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_6).is_ok());

        assert!(IssuerId::new(invalid_uri_identifier).is_err());

        assert!(IssuerId::new(valid_legacy_identifier_1).is_ok());
        assert!(IssuerId::new(valid_legacy_identifier_2).is_ok());

        assert!(IssuerId::new(too_short_legacy_identifier).is_err());
        assert!(IssuerId::new(illegal_base58_legacy_identifier_zero).is_err());
        assert!(IssuerId::new(illegal_base58_legacy_identifier_captial_o).is_err());
        assert!(IssuerId::new(illegal_base58_legacy_identifier_captial_i).is_err());
        assert!(IssuerId::new(illegal_base58_legacy_identifier_lower_l).is_err());
    }
}
