use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;

use crate::{
    error::DidPeerError,
    peer_did::{
        numalgos::{numalgo2::Numalgo2, numalgo3::Numalgo3},
        FromDidDoc, PeerDid,
    },
};

impl FromDidDoc for Numalgo3 {
    fn from_did_doc(
        did_document: DidDocument<ExtraFieldsSov>,
    ) -> Result<PeerDid<Numalgo3>, DidPeerError> {
        PeerDid::<Numalgo2>::from_did_doc(did_document)?.to_numalgo3()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_numalgo3() {
        let peer_did_2 = PeerDid::<Numalgo2>::parse("did:peer:2\
            .Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc\
            .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
            .Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg\
            .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaWRjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX0".to_string()).unwrap();
        assert_eq!(
            PeerDid::<Numalgo3>::parse(
                "did:peer:3.0e857e93798921e83cfc2ef8bee9cafc25f15f4c9c7bee5ed9a9c62b56a62cca"
                    .to_string()
            )
            .unwrap(),
            peer_did_2.to_numalgo3().unwrap()
        );
    }
}
