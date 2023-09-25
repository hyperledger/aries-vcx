# messages
The `messages` crate provides data structures representing the various messages used in Aries protocols. This `README` explores the architecture behind the crate and module organization to clarify the individual pieces and design decisions within the crate.

### Glossary
- `message content`: represents the protocol specific set of fields of a message
- `message decorators`: represents the decorator fields of a message not directly linked to the processing of a message within a protocol
- `message family` / `protocol`: represents the name of an Aries protocol, irrespective of its version (the `present-proof` part of `https://didcomm.org/present-proof/1.0/propose-presentation`)
- `message kind`: represents the name of a particular message within a specific protocol (the `propose-presentation` part of `https://didcomm.org/present-proof/1.0/propose-presentation`)
- `message type`: refers to the fully defined type of the message (which can translate to an entire, functional, `@type` field of an Aries message); it is comprised of a `protocol`, `major version`, `minor version` and `message kind`.

### Message structure
All messages, from all protocols and all versions, are part of the parent `AriesMessage` enum. The individual messages from a specific version of a specific protocol are then implemented in a cascade, as follows:

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

**NOTE:** Decorators fields are not exclusive to the `decorators` structure. Protocol specifications often use fields like the `~attach` decorator which is vital
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

Similarly to `AriesMessage`, the `Protocol` enum is represented in a cascading fashion such as:

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
Contains data structures for decorators. 

Unlike messages and their `@type` field, decorators get their version associated within their name `~thread/1`. Since only major versions are used, swapping a decorators version in a message represents a breaking change and would have to be explicitly defined in the message's content/decorators data structure. No resolution is required.

### Extending the crate
Adding new messages to the crate should be fairly easy to do, even without understanding all the inner workings. The main concepts needed are:  
- the `AriesMessage` enum encapsulates all messages
- messages are serialized/deserialized conditionally based on their `@type` field.
- the `@type` field gets deserialized using the `Protocol` enum, through which all
supported protocols can be resolved.
- the `PROTOCOL_REGISTRY` contains entries for all supported protocol versions, which is how minor version resolution is handled

With that in mind, a crude list of steps for extending the crate would be:
- changes in the `msg_types` module
    - adding/extending data types to represent the new protocol / protocol version
    - if adding the first version of a new protocol, extend the `Protocol` enum
    - add an entry to the `PROTOCOL_REGISTRY`
- changes in the `msg_fields` module
    - adding/extending data types to represent the message content and message decorators
- if adding the first version of a new protocol, extend the `AriesMessage` enum