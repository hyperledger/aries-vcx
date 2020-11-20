# Sample app

Start the app
```
RUST_LOG=aries_agent_rs=trace cargo run
```

Create and initialize agent
```
 curl -XPOST localhost:8806/agent
```

Get its agent provision
```
 curl -XGET localhost:8806/agent
```

