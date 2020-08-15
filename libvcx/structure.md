VCX Structure

/src/api
 - connection.rs
 - credential.rs
 - ..
 pub extern methods exposed into wrapper. This is VCX User interface
 These files reflect VCX objects + some extra APIs such as
 - utils.rs (interesting! vcx_agent_update_info)
 
/src
 - connection.rs
 - credential.rs
 - credential-def.rs
 - ..
These are high level internals and the crossroad between V1, V2 and Aries. VCX API layer levrages structures and methods defined here. The requests are then forward to either V1, V2 or Aries concrete implementations.
Confusingly, these files also include old implementations - V1 and V2. Aries implementation is in separate directory.

/src/v3    ARIES stuff

/src/utils   - Random stuff. Some picks:
- threadpool.rs - setting vcx threadpool (is this reused in vcx)
- qualifier.rs - fully qualified dids
- openssl.rs - encoding strings as BigNumber(sha256(string))
- httpclient.rs - sending http requests, mentions of SSL

/src/util/libindy - libindy fasade

/src/object_cache - threadsafe storage, used for storing handles for connections, credentials, etc.

/src/messages 
- mod.rs                - Messages for Client-Agency communication (Evernym agency protocol)
- agent_utils.rs        - Agency onboarding, cool utils like update_agent_info(), send_message_to_agency()
- create_key.rs         - Creating agent in agency
- get_message.rs        - Downloading messages from agency, including Aries
- invite.rs             - Legacy V1/V2 invitations (a2a)
- message_type.rs       - Legacy V1/V2 message families (a2a)
- payload.rs            - V1/V2 encryption/decryption (definition of msg types such as "credential-offer", or "CRED_OFFER")
- send_message.rs       - V1/V2 legacy message sending (in this case, proprietary Client-Agency protocol is used to instruct agency to send message to someone)
- thread.rs             - (primitive) Used across V1/V2/Aries
- uppdate_connection.rs - (c2a protocol) used across V1/V2/Aries, deleting a connection

