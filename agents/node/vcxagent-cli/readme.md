# VCX Agent CLI
CLI Utility - simple Aries agent with CLI interface. 

# What you need
As this tool is based on [vcxagent-core](../vcxagent-core), the prerequisites are the same. You
need to:
- have compiled `aries-vcx` library,
- have mediator agent running.

# Run
```
node vcxclient-cli.js --help
```
Example:
```
npm run interactive -- --agencyUrl 'https://localhost:8080' --rustLog aries-vcx=trace,vcx=info --name dev
```
