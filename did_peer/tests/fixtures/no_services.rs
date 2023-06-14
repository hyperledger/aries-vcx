pub static PEER_DID_NUMALGO_2_NO_SERVICES: &str = "did:peer:2\
.Ez6LSpSrLxbAhg2SHwKk7kwpsH7DM7QjFS5iK6qP87eViohud\
.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V";

pub static DID_DOC_NO_SERVICES: &str = r##"
    {
        "id": "did:peer:2.Ez6LSpSrLxbAhg2SHwKk7kwpsH7DM7QjFS5iK6qP87eViohud.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V",
        "verificationMethod": [
            {
                "id": "#6MkqRYqQ",
                "type": "Ed25519VerificationKey2020",
                "controller": "did:peer:2.Ez6LSpSrLxbAhg2SHwKk7kwpsH7DM7QjFS5iK6qP87eViohud.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V",
                "publicKeyMultibase": "z6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V"
            }
        ],
        "authentication": [],
        "assertionMethod": [],
        "keyAgreement": [
            {
                "id": "#6LSpSrLx",
                "type": "X25519KeyAgreementKey2020",
                "controller": "did:peer:2.Ez6LSpSrLxbAhg2SHwKk7kwpsH7DM7QjFS5iK6qP87eViohud.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V",
                "publicKeyMultibase": "z6LSpSrLxbAhg2SHwKk7kwpsH7DM7QjFS5iK6qP87eViohud"
            }
        ],
        "capabilityInvocation": [],
        "capabilityDelegation": []
    }
"##;
