* #### Connection Protocol [`https://didcomm.org/connections/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol)
    * [Inviter API](libvcx/src/api_lib/connection.rs) 
    * [Invitee API](libvcx/src/api_lib/connection.rs)
    * [Implementation](./libvcx/src/aries/handlers/connection/)
       - Missing implicit invitation   

* #### Basic Message: [`https://didcomm.org/basicmessage/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0095-basic-message)
    * [Implementation](./libvcx/src/aries/handlers/connection/)
    * [Send API](libvcx/src/api_lib/connection.rs)
    * [Download API](libvcx/src/api_lib/utils.rs)
    
* #### Credential Issuance [`https://didcomm.org/issue-credential/1.0/*`](https://github.com/hyperledger/aries-rfcs/blob/master/features/0036-issue-credential)
    * [Issuer API](libvcx/src/api_lib/issuer_credential.rs)  
    * [Holder API](libvcx/src/api_lib/credential.rs)
    * [Implementation](./libvcx/src/aries/handlers/issuance/)  
       - Missing initiation by holder using `propose-credential` message  

* #### Credential Presentation: [`https://didcomm.org/present-proof/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof)
    * [Verifier API](libvcx/src/api_lib/proof.rs)  
    * [Prover API](libvcx/src/api_lib/disclosed_proof.rs)
    * [Implementation](./libvcx/src/aries/handlers/proof_presentation)
       - Missing initiation or alternation of presentation by prover using `propose-presentation` message 

* #### Trust Ping: [`https://didcomm.org/trust_ping/1.0/*`](https://github.com/hyperledger/aries-rfcs/blob/master/features/0048-trust-ping/README.md)
    * [API](libvcx/src/api_lib/connection.rs)
    * [Implementation](./libvcx/src/aries/handlers/connection/)
    
* #### Discover Features: [`https://didcomm.org/discover-features/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features)
    * [API](libvcx/src/api_lib/connection.rs)
    * [Implementation](./libvcx/src/aries/handlers/connection/)
