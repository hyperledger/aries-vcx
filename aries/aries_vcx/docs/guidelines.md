# Guidelines for implementing protocol state machines

## No 'Sent' states
Do not create states to track whether a reply has been sent to counterparty. It is responsibility of
`aries-vcx` consumers to keep track of whether response message has been sent to counterparty. From state
machine perspective, creation of a protocol message implies a reply to that message can be processed.

Example: Do not create states such as `PresentationSent` (in context of `presentation-protocol` in role of prover).
Instead, only create state `PresentationPrepared` which enables `aries-vcx` consumers to get the presentation message
and send it to the counterparty. Subsequently, it should be possible to further drive state machine in  `PresentationPrepared` 
using `presentation-ack` message received from verifier. 

Additional rationale: 
In past we used states such as `<MessageType>Sent`. However, simply saying we sent 
something doesn't really mean that much since we cannot guarantee it was also received, especially in a transport 
abstract manner. Therefore, we could model the state machines so that we work on `<MessageType>Generated` states, 
which we actually can guarantee, and then the consumer code can retrieve the message and send it (as many times as they want).
Then the state machine would advance when the reply to the sent message is received (if one is expected).

## Processing Aries messages moves state
Processing a message of valid type from counterparty must move state. Even if we are dealing just with acknowledgement type of message.

Example: In `issue-credential` protocol, do not enter `Finished` simply after building verifiable credential.
You might rather have `CredentialBuilt` and only enter `Finished` state after receiving `ack` message, which
is part of `issue-credential` protocol - as opposed of simply tracking acknowledgement within the `Finished` state.

## Create "internal" states as needed, recognize "external" RFC states
Aries RFCs describe the protocol state from perspective of external viewer. While we need to adhere the protocol
in terms of what messages does the counterparty accept at a given moment of the protocol lifecycle, as library
developers, we are also concerned about the APIs we expose and its properties. It proved to be useful to create
internal states which are not in bijections with the set of states described by aries RFCs.

### Internal VS external states 
We will refer to custom states as "internal" and states recognized by RFCs as "external". Since the "external" states
generally define what messages can be received when, for given "internal" state, it must be possible to determine the
current "external state".

### Internal intermediate states
There can also be internal states we could refer to as "internal intermediate". This is internal state which does not 
map to any "external" state and therefore in such state, no aries message can be received. Instead, an action from
state machine owner is expected. For example, in some presentation protocol, on the side of prover, we might have states
`PresentationRequestProcessed`, `PresentationGenerated`.
Here, `PresentationRequestProcessed` is intermediate state where no message from counterparty is expected. The only 
way to drive the state machine forward is by building the proof presentation (which must be initiated locally).

### Creating new states
You can create new state to distinguish various sub-states of a state defined by RFC. Typical example is around final
states which is in some protocol RFCs is described as single state which encodes variety of conditions. 
For example [`present-proof 1.0 procol RFC`](https://github.com/hyperledger/aries-rfcs/blob/main/features/0037-present-proof/README.md)
describes 1 final state `done`.
in practice it's cleaner to distinguish number of final states like `Verified`, `NotVerified`, `Failed`. Note that this
follows our previous rule
> it must be possible to determine the current "external state"

as each of these states could be mapped to RFC state `done`.

### Removing/renaming states
As per first point in this list `"No 'Sent' states"` we want to avoid encoding IO information whether a message
has been sent over wire, so we would not adopt external RFC state `presentation-sent` as one of our internal states.
Instead, we would create new state `presentation-built` which doesn't promise more it can - however following our
mapping rule, this internal state `presentation-built` maps to external state `presentation-sent`, assuming it's ready
to receive `presentation-ack` message from verifier.

## Separate final states
While Aries RFCs sometimes refer to single final state to convey nothing more can happen after exchange
of certain messages - for example in diagram [here](https://github.com/hyperledger/aries-rfcs/blob/main/features/0453-issue-credential-v2/README.md)
- on implementation level we want to distinguish different internal states signalling different circumstances (which
  simply happen to share the property of being final).

Example: instead of having `Final` state which would maintain `status: Success | Failed | Rejected`, we
should have 3 separate states `Success`, `Failed`, `Rejected`.

## Deterministic transitions
Transition should either fail and leave the current state unmodified, or transition to predetermined new state.

Example: in our legacy implementations, some transitions lead to multiple states, and the actual
new state was determined in runtime. The typical case was that transition either succeeded and moved to
the "happy-case" next state; or failed and moved to "failed" state. 

## Outsourced message sending 
State machines should not attempt to send messages over network. State machines process received messages, 
build responses, but should never perform message sending. 

## Problem reports
State machines should not automatically submit problem-reports. If a problem occurs upon execution of a transition,
it should fail with error (per previous point) and should be left up to state machine user to decide: retry,
do nothing, move to failed state, send a problem report.
