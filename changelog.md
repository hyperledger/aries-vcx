[ Hyperledger  + AbsaOSS ]

# Upcoming 0.9.0
* Add `4.0` protocol - objects returned from LibVCX API are in Aries format [PR](https://github.com/hyperledger/indy-sdk/pull/2196)
* Support revocations when using `3.0` / `4.0` protocol [PR](https://github.com/AbsaOSS/libvcx/pull/24)
* Changed interface and behaviour of vcx_update_webhook_url which uses `UpdateComMethod` Vcx2Agency message type to let agency know about new webhook url for agent.
* Add `vcx_delete_credential` method to delete credential from wallet.
* Methods `vcx_*_update_state_with_message` are not automatically updating status of message in [agency](https://github.com/hyperledger/indy-sdk/pull/2200)
* Fixed double escaping and sending of generic msgs. [PR](https://github.com/hyperledger/indy-sdk/pull/2202)
* [Update VCX iOS wrapper](https://github.com/hyperledger/indy-sdk/pull/2187) Fixes of vcx_disclosed_proof_get_requests and addition of APIs `connectionGetState`, `connectionUpdateState`, `connectionRelease`, `credentialRelease`, `proofGetRequests`, `proofGetState`, `proofUpdateState`, `proofRelease` 
* LibVCX Python fix for accepting TAA [PR](https://github.com/hyperledger/indy-sdk/pull/2175)
* LibVCX NodeJS fix for sending basic message [PR](https://github.com/hyperledger/indy-sdk/pull/2162)
* Add support for predicates when requesting proof [PR](https://github.com/hyperledger/indy-sdk/pull/2119)
* Add iOS wrapper APIs `updateWebhookUrl`, `errorCMessage`, `connectionGetPwDid`, `connectionGetTheirPwDid`
* Fix Java wrapper bugs in `issuerSendCredentialCB`, `vcxProofGetStateCB` [PR](https://github.com/AbsaOSS/libvcx/pull/11)
* Add Java demo [PR](https://github.com/AbsaOSS/libvcx/pull/16)
* Add Java wrapper APIs `issuerRevokeCredential`, `vcxUpdateWebhookUrl` [PR](https://github.com/AbsaOSS/libvcx/pull/12)
* Make `comment` inside Credential optional [PR](https://github.com/AbsaOSS/libvcx/pull/18)
* Fix behaviour of wallet import [PR](https://github.com/AbsaOSS/libvcx/pull/9)
* Add API `vcx_credentialdef_rotate_rev_reg_def` to rotate revocation registries [PR](https://github.com/AbsaOSS/libvcx/pull/4/files) 
so that credential definition can be used after its revocation registry accumulator gets full.
* Update Node VCX Wrapper to support NodeJS 12 [PR](https://github.com/AbsaOSS/libvcx/pull/6)
* Fix caching bug when multiple revokable credentials are issued under the same revocation revocation registry to a 
single vcx client [PR](https://github.com/AbsaOSS/libvcx/pull/5)
* Enable batch revocations by adding APIs `vcx_credentialdef_publish_revocations`, `vcx_issuer_revoke_credential_local` 
[PR](https://github.com/AbsaOSS/libvcx/pull/8)
* Remove dummy cloud agency from the repository [PR](https://github.com/AbsaOSS/libvcx/pull/2)
* Update project readme, delete unnecessary / old documentation [PR](https://github.com/AbsaOSS/libvcx/pull/32) 


[ Hyperledger ]

## 0.8.0
* Bugfixes
* Fixed compatibility between proprietary (`protocol_version`: `2.0`/`1.0`) and aries protocols (`protocol_version`: `3.0`).

## 0.7.0
* Removed `connection_handle` from functions to get protocol messages.
* Added ability to accept a duplicate connection by redirecting to the already existing one instead of forming a duplicate connection.
* Added a new function `vcx_disclosed_proof_decline_presentation_request` to explicitly reject a presentation request.
* Added a new function `vcx_connection_info` to get information about connection.
* Bugfixes

## 0.6.2
* Implemented Basic Message RFC (IS-1189)
* Updated library to support "names" parameter in Proof Request Revealed Attributes (IS-1381)
* others minor bugfixes

## 0.6.1
* Bugfixes

## 0.6.0
* LibVCX Aries support:
    * Implemented Trust Ping RFC (IS-1435).
        * added `vcx_connection_send_ping` function to send `Ping` message on remote connection.
        * handle inbound `Ping` message after connection is established (use `vcx_connection_update_state` or `vcx_connection_update_state_with_message`).
    * Implemented Discover Features RFC (IS-1155)
        * added `vcx_connection_send_discovery_features` function to send discovery features request on remote connection.
        * handle inbound `Query` and `Disclose` messages after connection is established (use `vcx_connection_update_state` or `vcx_connection_update_state_with_message`).
    * Implemented Service Decorator RFC (IS-1449)
    * Added new Vcx setting: `actors` which specifies the set of protocols application supports (is used for Discover Features protocol handling).

## 0.5.0
* LibVCX Aries support:
    * Now you can keep old code without doing any changes and use Aries protocols if you have not parsed any messages. If you need more information -- see the migration guide.
    * Implemented Connection RFC (IS-1180)
    * Implemented Credential Issuance RFC (IS-1393)
    * Implemented Credential Presentation RFC (IS-1394)
    * Integrated Connection Protocol into Dummy Cloud Agent (IS-1392)

## 0.4.2
* *EXPERIMENTAL*
  Extended provisioning config to accept optional `did_method` filed. This field should be used to create fully qualified DIDs.
  The format of identifiers used on CredentialIssuance and ProofPresentation will determine based on the type of remote DID.
* Bugfixes

## 0.4.1
* Supported endorsing of transactions in Libvcx.
    * `vcx_*_prepare_for_endorser` - functions for `schema` and `credentialdef` which build transaction and crete internal object in differed state.
    * `vcx_*_update_state` and `vcx_*_get_state` - functions to update/get state of `schema`/`credentialdef` internal object.
    * `vcx_endorse_transaction` - function to endorse a transaction.
* Supported sign/verify with payment address functionality in Libvcx.
    * `vcx_wallet_sign_with_address` - to sign a message with a payment address.
    * `vcx_wallet_verify_with_address` - to verify a signature with a payment address.
* Extended Libvcx initialization config to accept pool configuration.
* Bugfixes

* 0.4.0
* Added a set of new APIs around credentials and proofs that work with messages that should be exchanged without handling the transport of those messages.
This removes the dependency on an agency/cloud-agent and allows the user of the SDK to transport those messages themselves.
There are two types of functions:
    * `vcx_*_get_request_msg` - gets a message that can be sent to the specified connection.
    * `vcx_*_update_state_with_message` - checks for any state change from the given message and updates the state attribute.
* Added new *EXPEREMENTAL* functions to get requirements and price for a ledger request.
    * `vcx_get_request_price` - returns request minimal request price for performing an action in case the requester can do it.
* Updated Indy-SDK CI/CD pipelines to test, to build and to publish Android artifacts for Libvcx.
* Bugfixes:
    * Android Crash upon logging
    * others minor bugfixes

* 0.3.2
    * Bugfixes

* 0.3.1
    * Added new `*_update_state_with_message` functions for connections, proofs, cred defs.
    * Bugfixes

* 0.3.0
    * Added new functions to support work with `Transaction Author Agreement` concept.
        * `vcx_get_ledger_author_agreement` to retrieve author agreement and acceptance mechanisms set on the Ledger.
        * `vcx_set_active_txn_author_agreement_meta` to set some accepted agreement as active and to use it for transaction sending.
    * Updated Libvcx behavior to use *EXPERIMENTAL* `Cache API` for faster-getting schemas and credential definitions.
    * Bugfixes

* 0.2.4
    * Bugfixes

* 0.2.3
* Updated Vcx to support community A2A protocol.
Added `protocol_type` field to VCX provisioning config with indicates A2A message format will be used.
    * `1.0` means the current protocol.
    * `2.0` means community (IN PROGRESS) protocol which in the current state includes implementation of the following HIPEs:
        * Message Types - https://github.com/hyperledger/indy-hipe/tree/master/text/0021-message-types
        * Message Threading - https://github.com/hyperledger/indy-hipe/tree/master/text/0027-message-id-and-threading
        * Wire Message - https://github.com/hyperledger/indy-hipe/tree/master/text/0028-wire-message-format.
* Added function `vcx_get_current_error` to get additional information for last error occurred in Libvcx.
* Bugfixes

* 0.2.2
    * Bugfixes

* 0.2.1
    * Bugfixes

* 0.2.0
    * Initial release
