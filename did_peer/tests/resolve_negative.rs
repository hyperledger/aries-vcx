mod fixtures;

use did_peer::{
    error::DidPeerError,
    resolver::{
        options::{ExtraFieldsOptions, PublicKeyEncoding},
        PeerDidResolver,
    },
};
use did_resolver::traits::resolvable::{resolution_options::DidResolutionOptions, DidResolvable};
use tokio::test;

async fn resolve_error(peer_did: &str) -> DidPeerError {
    let options = DidResolutionOptions::new(
        ExtraFieldsOptions::new().set_public_key_encoding(PublicKeyEncoding::Multibase),
    );
    *PeerDidResolver
        .resolve(&peer_did.parse().unwrap(), &options)
        .await
        .unwrap_err()
        .downcast::<DidPeerError>()
        .unwrap()
}

#[test]
async fn test_resolve_numalgo_2_invalid_0() {
    let peer_did = "did:peer:2\
        .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
        .Vz6666YqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
        .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0";
    assert!(matches!(
        resolve_error(peer_did).await,
        DidPeerError::PublicKeyError(_)
    ));
}

#[test]
async fn test_resolve_numalgo_2_invalid_1() {
    let peer_did = "did:peer:2\
        .Ez7777sY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
        .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
        .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0";
    assert!(matches!(
        resolve_error(peer_did).await,
        DidPeerError::PublicKeyError(_)
    ));
}

#[test]
async fn test_resolve_numalgo_2_invalid_2() {
    let peer_did = "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc.\
                    Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.\
                    Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg.Sasdf123";
    assert!(matches!(
        resolve_error(peer_did).await,
        DidPeerError::Base64DecodingError(_)
    ));
}

#[test]
async fn test_resolve_numalgo_2_invalid_3() {
    let peer_did = "did:peer:2\
        .Cz6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
        .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
        .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0";
    assert!(matches!(
        resolve_error(peer_did).await,
        DidPeerError::DidValidationError(_)
    ));
}

#[test]
async fn test_resolve_numalgo_2_invalid_4() {
    let peer_did = "did:peer:2\
        .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
        .Vz6MkqRYqQiS\
        .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0";
    assert!(matches!(
        resolve_error(peer_did).await,
        DidPeerError::PublicKeyError(_)
    ));
}

#[test]
async fn test_resolve_numalgo_2_invalid_5() {
    let peer_did = "did:peer:2\
        .Ez6LSbysY2\
        .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
        .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
        .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0";
    assert!(matches!(
        resolve_error(peer_did).await,
        DidPeerError::PublicKeyError(_)
    ));
}

#[test]
async fn test_resolve_numalgo_2_invalid_6() {
    let peer_did = "did:peer:2\
        .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
        .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7Vcccccccccc\
        .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
        .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0";
    assert!(matches!(
        resolve_error(peer_did).await,
        DidPeerError::PublicKeyError(_)
    ));
}

#[test]
async fn test_resolve_numalgo_2_invalid_7() {
    let peer_did = "did:peer:2\
        .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCcccccccc\
        .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
        .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
        .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXX0";
    assert!(matches!(
        resolve_error(peer_did).await,
        DidPeerError::PublicKeyError(_)
    ));
}
