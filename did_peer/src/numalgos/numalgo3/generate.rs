use did_parser::Did;
use sha256::digest;

use crate::{
    error::DidPeerError,
    peer_did::{numalgos::numalgo3::Numalgo3, PeerDid},
};

pub fn generate_numalgo3(did: &Did) -> Result<PeerDid<Numalgo3>, DidPeerError> {
    let numalgoless_id = did.id().chars().skip(2).collect::<String>();
    let numalgoless_id_hashed = digest(numalgoless_id);
    PeerDid::<Numalgo3>::parse(format!("did:peer:3.{}", numalgoless_id_hashed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_numalgo3() {
        let peer_did_2 = Did::parse("did:peer:2\
            .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
            .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
            .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
            .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaWRjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX0".to_string()).unwrap();
        assert_eq!(
            PeerDid::<Numalgo3>::parse(
                "did:peer:3.0e857e93798921e83cfc2ef8bee9cafc25f15f4c9c7bee5ed9a9c62b56a62cca".to_string()
            )
            .unwrap(),
            generate_numalgo3(&peer_did_2).unwrap()
        );
    }
}
