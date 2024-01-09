# Simple Message Relay
The simple message relay is a basic implementation of a mediator/relay service which can be used for testing agent-to-agent comms.

*This relay should never be used in production/public environments, as it intentionally lacks user authorization and data persistence for testing simplicity purposes.*

Like a mediator, an agent can provide their HTTP/s "user endpoint" to peers during DIDComm connection protocols. An agent can then poll to collect incoming messages that have been sent to their endpoint.

The simplic design makes this service ideal for testing situations where a message-receiving endpoint is required by an agent, without the overhead of bootstrapping and connecting with an official Aries RFC based mediator.

# Service Setup
Within this directory, the service can be ran with `cargo`:
```
cargo run
```

Or from the aries-vcx repo base directory:
```
cargo run --bin simple_message_relay
```

This will start the relay serving on localhost port `8420`.

## Public Endpoints
The service can be exposed publicly by running the service on a machine with a public IP address exposed, ensuring that the service's port is exposed to incoming traffic.

Alternatively, the port can easily be exposed using [ngrok](https://ngrok.com/).
```
ngrok http 8420
```

# Service Usage
## Inform Peers
To use the service as an aries agent, the agent should pick a unique ID (representing their user account) and encode it within their endpoint for the service:

```
{base_url}/send_user_message/{user_id}

// e.g.

http://localhost:8420/send_user_message/1234-1234-1234-1234
```

this "user endpoint" can then be used as the agent's `service_endpoint` in connection protocols, such as providing it into aries_vcx `Connection<Invitee, Invited>::send_request(.., service_endpoint: Url, ...)` API. This will inform the peer that they should send messages to this address.

Peer agents can use this endpoint as if it was any other DIDComm http endpoint.

## Collect Incoming Messages
After a peer sends the agent a message to the endpoint, they can be collected by calling `GET` on the following endpoint for the user:

```
{base_url}/pop_user_message/{user_id}

// e.g.

http://localhost:8420/pop_user_message/1234-1234-1234-1234
```

The `user_id` should match the user_id selected in the [above section](#inform-peers).

Calling this endpoint will return a body with the bytes of the oldest message in the relay message queue. Internally, the service will pop this message from the queue, such that the next call to the endpoint will return the next oldest message.

If no message could be found, an empty body and a `NO CONTENT` status is returned.

The bytes returned should be exactly as they were sent by the peer. As such, they should be able to be decrypted/unpacked and parsed as a DIDComm message, where they can then be passed into aries_vcx protocol handlers.