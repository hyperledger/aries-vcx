mod fixtures;

use did_peer::peer_did_resolver::{
    options::{ExtraFieldsOptions, PublicKeyEncoding},
    resolver::PeerDidResolver,
};
use did_resolver::traits::resolvable::{resolution_options::DidResolutionOptions, DidResolvable};
use tokio::test;

macro_rules! resolve_negative_test {
    ($test_name:ident, $peer_did:expr) => {
        #[test]
        async fn $test_name() {
            let resolver = PeerDidResolver::new();
            let options = DidResolutionOptions::new()
                .set_extra(ExtraFieldsOptions::new().set_public_key_encoding(PublicKeyEncoding::Multibase));
            let ddo_result = resolver.resolve(&$peer_did.parse().unwrap(), &options).await;
            assert!(ddo_result.is_err());
        }
    };
}

resolve_negative_test!(
    test_resolve_numalgo_2_invalid_0,
    "did:peer:2\
    .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
    .Vz6666YqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
    .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0"
);

resolve_negative_test!(
    test_resolve_numalgo_2_invalid_1,
    "did:peer:2\
    .Ez7777sY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
    .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
    .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0"
);

resolve_negative_test!(
    test_resolve_numalgo_2_invalid_2,
    "did:peer:2\
    .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
    .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
    .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
    .Sasdf123"
);

resolve_negative_test!(
    test_resolve_numalgo_2_invalid_3,
    "did:peer:2\
    .Cz6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
    .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
    .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0"
);

resolve_negative_test!(
    test_resolve_numalgo_2_invalid_4,
    "did:peer:2\
    .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
    .Vz6MkqRYqQiS\
    .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0"
);

resolve_negative_test!(
    test_resolve_numalgo_2_invalid_5,
    "did:peer:2\
    .Ez6LSbysY2\
    .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
    .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
    .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0"
);

resolve_negative_test!(
    test_resolve_numalgo_2_invalid_6,
    "did:peer:2\
    .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
    .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7Vcccccccccc\
    .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
    .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0"
);

resolve_negative_test!(
    test_resolve_numalgo_2_invalid_7,
    "did:peer:2\
    .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCcccccccc\
    .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
    .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
    .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0"
);
