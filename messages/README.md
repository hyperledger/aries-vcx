# messages
The `messages` crate provides data structures representing the various messages used in Aries protocols. This `README` explores the architecture behind the crate and module organization, making implementing new messages/protocols easier.

### Message strucutre
All messages, from all protocols and all versions, are part of the parent `AriesMessage` enum. The individual messages from a specific version of a specific protocol are then implemented in a cascade:

- AriesMessage
    - Connection
        - ConnectionV1
            - ConnectionV1_0
                - RequestV1_0
                - ResponseV1_0

            - ConnectionV1_1
                - RequestV1_1
                - ResponseV1_1

        - ConnectionV2
            - ConnectionV2_0
                - RequestV2_0
                - ResponseV2_0

            - ConnectionV2_1
                - RequestV2_1
                - ResponseV2_1
    - PresentProof
        - PresentProofV1
            - PresentProofV1_0
                - PresentationV1_0

**NOTE:** For simplicity, inner enums are avoided if there's no need for them (e.g: there's only one message in a protocol and/or there's only one protocol version implemented). Adding messages/protocol versions implies modifying existing enums to match their version.

Each individual message is then ultimately an alias to the `MsgParts` generic structure, which separates the message fields into categories:

- MsgParts
    - `id`: the `@id` field of the message
    - `content`: a structure containing the protocol specific fields
    - `decorators`: a structure of decorator fields processable irrespective of the protocol this message is part of

**NOTE:** Decorators fields are not exclusive to the `decorators` strucutre. Protocol specifications often use fields like the `~attach` decorator which is vital
for the protocol implementation and is therefore part of the `content` struct. The `decorators` structure is meant for decorator fields that can be processed irrespective of the protocol being used, such as `~thread`, `~timing`, etc.

### The `@type` field
Aries messages are differentiated by their `@type` field which contains the protocol, the protocol version and the specific message kind that a message is expected to be. Based on the `@type` field, we know what message structure to expect
from the rest of the fields.

The crate takes advantage of delayed serialization/deserialization so that we first
look at the `@type` field of a message and deduce what message structure to use for the rest of the fields.

The approach is similar with tagged serialization/deserialization in `serde`, with the caveat that we also do some version resolution as per Aries [semver rules](https://github.com/hyperledger/aries-rfcs/blob/main/concepts/0003-protocols/README.md#semver-rules-for-protocols).

As a result, simple `serde` tagged serialization/deserialization is not sufficient. We instead dedicate the `msg_types` module for this purpose.

### The `msg_types` module
We want to take the `@type` field and parse it to determine the exact protocol, version and message to process. The machinery is in place for that to happen through the delayed serialization/deserialization, but the protocol version resolution and all the possible variants for the `@type` field are located within
this module and encapsulated in the `Protocol` enum.

Similarly to `AriesMessage`, the `Protocol` enum is represented in a cascading fashion:

- Protocol
    - ConnectionType
        - ConnectionTypeV1
            - V1_0 -> ConnectionTypeV1_0
                - Request
                - Response
            - V1_1 -> ConnectionTypeV1_1
                - Request
                - Response

When deciding the minor version to use in a protocol, the protocol and version of the `@type` field is looked up in the `PROTOCOL_REGISTRY` (a lazily initialized map containing all protocols implemented). On success, the specified protocol version can be used as it is implemented. On failure, though, the minor version is decremented and looked up again until either the minor version reaches `0` or the lookup succeeds.

**NOTE:** The `Protocol` enum has values like: `Protocol::ConnectionType::ConnectionTypeV1::V1_0`. The exact message kind is not part of the enum but rather there is type-linking involved (hence the arrow ->). This allows the `Protocol` enum to represent only protocols and protocols version by itself, while also providing mechanisms for parsing the message kind and thus getting the exact message to further process.

### The `msg_fields` module
This module contains the actual messages data structures with all fields defined (apart from `@type`).
In practice, the module exports aliases for concrete definitions of `MsgParts`,
such as ```pub type IssueCredential = MsgParts<IssueCredentialContent, IssueCredentialDecorators>;```. 

The relevant submodule then contains the actual definitions of the `content` and `decorators` data structure (such as `IssueCredentialContent` and `IssueCredentialDecorators`).

### The `decorators` module
Contains data structure for decorators. 

Unlike messages and their `@type` field, decorators get their version associated within their name `~thread/1`. Since only major versions are used, swapping a decorators version in a message represents a breaking change and would have to be explicitly defined in the message's content/decorators data structure. No resolution is required.