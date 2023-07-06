# Simple Message Relay
The simple message relay is a basic implementation of a mediator/relay service which can be used for testing agent-to-agent comms.

*This relay should never be used in production/public environments, as it intentionally lacks user authorization and data persistance for testing simplicitly purposes.*

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
To use the service as an aries agent, the agent should pick a unique ID (representing their user account) and encode it within their endpoint for the service:

```
http://{base_url}/receive_user_message/{user_id}

// e.g.

http://localhost:8420/receive_user_message/1234-1234-1234-1234
```

this "user endpoint" can then be used as the agent's `service_endpoint` in connection protocols.

TODO - aries_vcx example and example of fetching