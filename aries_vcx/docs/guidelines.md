# Guidelines for implementing protocol state machines

## No 'Sent' states
Do not create states to track whether a reply has been sent to counterparty. It is responsibility of
`aries-vcx` consumers to keep track of whether response message has been sent to counterparty. From state
machine perspective, creation of a protocol message implies a reply to that message can be processed.

Example: Do not create states such as `PresentationSent` (in context of `presentation-protocol` in role of prover).
Instead, only create state `PresentationPrepared` which enables `aries-vcx` consumers to get the presentation message
and send it to the counterparty. Subsequently, it should be possible to further drive state machine in  `PresentationPrepared` 
using `presentation-ack` message received from verifier. 

## Separate final states
While Aries RFCs commonly refer to single final state to convey nothing more can happen after exchange
of certain messages - for example in diagram [here](https://github.com/hyperledger/aries-rfcs/blob/main/features/0453-issue-credential-v2/README.md)
- on implementation level we want to distinguish different internal states signalling different circumstances (which 
simply happen to share the property of being final).

Example: instead of having `Final` state which would maintain `status: Success | Failed | Rejected`, we
should have 3 separate states `Success`, `Failed`, `Rejected`.

## Receives Aries messages always move state
Processing a message from counterparty must move state. Even if we are dealing just with acknowledgement type of message.

Example: In `issue-credential` protocol, do not enter `Finished` simply after building verifiable credential.
You might rather have `CredentialBuilt` and only enter `Finished` state after receiving `ack` message, which
is part of `issue-credential` protocol - as opposed of simply tracking acknowledgement within the `Finished` state.

## Use intermediate states when needed
Aries RFCs describe the protocol state from perspective of external viewer. While we need to adhere the protocol
in terms of what messages does the counterparty accept at a given moment of the protocol lifecycle, as library
developers, we are also concerned about the APIs we expose and its properties. It proved to be useful to create
intermediate states that do not exhibit a strict bijection with states described by aries RFCs.

Example: In [`present-proof 1.0`](https://github.com/hyperledger/aries-rfcs/blob/main/features/0037-present-proof/README.md)
protocol, there's following states described:
```
request-received
proposal-sent
presentation-sent
reject-sent
done
```
However, in practice it's useful to have not only state `request-received` but also `presentation-prepared`. It's
also useful to have multiple final states than just `done`, for example `Verified`, `NotVerified`, `Failed`.

On the flip side, as per first point in this list `"No 'Sent' states"` - it's better to avoid creating "sent" states like
the suggested byt the RFCs
```
proposal-sent
presentation-sent
reject-sent
```

However, note that in with our custom states, it must be always possible to determine where we are within the protocol
as described by the RFC - in other words, what messages we accept in given moment. For example, our 
state `presentation-prepared` would reflect RFC state `presentation-sent` where we are ready 
to receive `presentation-ack` from verifier. 
Equally, our custom states `Verified`, `NotVerified`, `Failed` map to RFC state `done`.

## Deterministic transitions
Transition should either fail and leave the current state unmodified, or transition to predetermined new state.

Example: in our legacy implementations, some transitions lead to multiple states, and the actual
new state was determined in runtime. The typical case was that transition either succeeded and moved to
the "happy-case" next state; or failed and moved to "failed" state. 

## Problem reports
State machines should not automatically submit problem-reports. If a problem occurs upon execution of a transition,
it should fail with error (per previous point) and should be left up to state machine user to decide: retry,
do nothing, move to failed state, move to failed state and send problem report.
