* Connection:
    * [Aries Connection Spec](https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol)
    * Inviter
        * [API](../libvcx/api/connection.rs) 
        * [State diagram](states/aries/connection-inviter.puml) 
    * Invitee
        * [API](../libvcx/api/connection.rs) 
        * [State diagram](states/aries/connection-invitee.puml) 

* Credential Issuance:
    * [Aries Issue Cred Spec](https://github.com/hyperledger/aries-rfcs/blob/master/features/0036-issue-credential)
    * Issuer
        * [API](../libvcx/api/issuer_credential.rs) 
        * [State diagram](states/aries/issuer-credential.puml) 
    * Holder
        * [API](../libvcx/api/credential.rs) 
        * [State diagram](states/aries/credential.puml) 

* Credential Presentation:
    * [Aries Present Proof Spec](https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof)
    * Verifier
        * [API](../libvcx/api/proof.rs) 
        * [State diagram](states/aries/proof.puml) 
    * Prover
        * [API](../libvcx/api/disclosed_proof.rs) 
        * [State diagram](states/aries/disclosed-proof.puml) 
