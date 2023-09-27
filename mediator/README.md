# Aries VCX Mediator

A mediator service for Aries Agents.

**Status**: Dev

## Build

This project depends on `aries-vcx` and is tightly integrated to work with it.
As such we expect this `mediator` module to be in a subdirectory of aries-vcx repo.
You may transplant it, but expect to change the `Cargo.toml` and adjust the dependencies on aries modules manually.

When ready it's simple to build.

```bash
# Dev environment build
cargo build
```

## Usage

You can run and test the produced binary using cargo.

```bash
cargo run
```

### Configurable Options

Currently the mediator reads the following environment variables.

```yaml
`ENDPOINT_ROOT`: 
- **Description**: This is the address at which the mediator will listen for connections.
- **Default**: "127.0.0.1:8005"
- **Usage**: `ENDPOINT_ROOT=127.0.0.1:3000 cargo run`
```

## API

Currently exposed endpoints.

```yaml
`/register`:
- **Description** : Shows an Aries Out Of Band (OOB) invitation which can be used to connect to the mediator using a conformant Aries Agent.
```
