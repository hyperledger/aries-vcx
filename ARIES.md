* #### Connection Protocol [`https://didcomm.org/connections/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol)
    * [Inviter API](libvcx/src/api_lib/connection.rs) 
    * [Invitee API](libvcx/src/api_lib/connection.rs)
    * [Implementation](./libvcx/src/aries/handlers/connection)

* #### Out of Band: [`https://didcomm.org/out-of-band/1.1/*`](https://github.com/hyperledger/aries-rfcs/blob/main/features/0434-outofband)
  * [Inviter API](libvcx/src/api_lib/out_of_band.rs)
  * [Invitee API](libvcx/src/api_lib/out_of_band.rs)
  * [Implementation](./libvcx/src/aries/handlers/out_of_band)

* #### Basic Message: [`https://didcomm.org/basicmessage/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0095-basic-message)
    * [Implementation](./libvcx/src/aries/handlers/connection/)
    * [Send API](libvcx/src/api_lib/connection.rs)
    * [Download API](libvcx/src/api_lib/utils.rs)
    
* #### Credential Issuance [`https://didcomm.org/issue-credential/1.0/*`](https://github.com/hyperledger/aries-rfcs/blob/master/features/0036-issue-credential)
    * [Issuer API](libvcx/src/api_lib/issuer_credential.rs)  
    * [Holder API](libvcx/src/api_lib/credential.rs)
    * [Implementation](./libvcx/src/aries/handlers/issuance)  

* #### Credential Presentation: [`https://didcomm.org/present-proof/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof)
    * [Verifier API](libvcx/src/api_lib/proof.rs)  
    * [Prover API](libvcx/src/api_lib/disclosed_proof.rs)
    * [Implementation](./libvcx/src/aries/handlers/proof_presentation) 

* #### Trust Ping: [`https://didcomm.org/trust_ping/1.0/*`](https://github.com/hyperledger/aries-rfcs/blob/master/features/0048-trust-ping/README.md)
    * [API](libvcx/src/api_lib/connection.rs)
    * [Implementation](./libvcx/src/aries/handlers/trust_ping)
    
* #### Discover Features: [`https://didcomm.org/discover-features/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features)
    * [API](libvcx/src/api_lib/connection.rs)
    * [Implementation](./libvcx/src/aries/handlers/connection)
