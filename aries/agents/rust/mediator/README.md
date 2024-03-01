# Aries VCX Mediator

A mediator service for Aries Agents.

**Status**: Dev

## Build

This project depends on `aries-vcx` and is tightly integrated to work with it.

When ready, it's simple to build.

```bash
# Dev environment build
cargo build
```

## Usage

### Cargo

You can run and test the produced binaries using cargo.

```bash
cargo run --bin mediator
```

```bash
# For manual testing / demo purposes
cargo run --package client-tui

# To also see panic and debug info produced by tui
cargo run --package client-tui 2> err || cat  err
```

```bash
# To run automated tests, you should have a mediator already running
# # Terminal 1
cargo run --bin mediator
# # Terminal 2
cargo test 
```

### Docker

#### 1. You can build the docker image yourself or pull it from github

`Dockerfile` is provided, to build the mediator image.
The image produced includes the mediator binary and required ssl libraries.

```bash
# Note: Build context needs to include aries-vcx repository root. 
docker build --tag mediator --file ./Dockerfile  ../../../../
```

Alternatively you can pull the latest prebuilt mediator image directly.

```bash
docker pull ghcr.io/hyperledger/aries-vcx/mediator:main
```

#### 2. Use docker-compose with provided `compose.yaml` to quickly bring up mediator service along with mysql database

This can also be used for local development purposes, when you build your own mediator image as described above.

```bash
# Note: Configuration can be customized using .env file, 
# or by manually passing expected environment variables.
docker compose up -d
```

To wind down services

```bash
docker compose down
```

When you run compose, the mediator and database containers are started on the same private network.  
The configuration in .env file is used for database<->mediator connection parameters.  
The mediator (but not the database) is additionally exposed on localhost:8005 by default for interaction.

### Configurable Options

Currently the mediator reads the following environment variables.

```yaml
`ENDPOINT_ROOT`: 
- **Description**: This is the address at which the mediator will listen for connections.
- **Default**: "127.0.0.1:8005"
- **Usage**: `ENDPOINT_ROOT=0.0.0.0:3000`

`MYSQL_URL`: 
- **Description**: MySQL url for the MYSQL database used for mediator persistence. 
- **Default**: - (This is required!)
- **Usage**: `MYSQL_URL=mysql://admin:password1235@localhost:3306/mediator-persistence.db`
```

### Configurable Features

Mediator's Agent contains some client code, which is used for testing, and dependent crates.  
In production, you may want to remove this. To do so, pass `--no-default-features` to cargo when building.  
This will ensure you are not pulling in the client code.  

```bash
cargo build --package mediator --no-default-features --release
```

## API

Currently exposed endpoints.

```yaml
`/register`:
- **Description** : | 
    Shows an Aries Out Of Band (OOB) invitation which can be used to connect to the mediator using a conformant Aries Agent.
    Use `Accept` header with value "application/json" to receive same in json format.

`/register.json`:
- **Description** : Returns OOB invitation in json format.
```

```yaml
`/didcomm`:
- **Description** : | 
    Endpoint for Aries DIDCOMM communication. 
    Encrypted Aries messages (envelops) can be passed and received from this endpoint in json serialized format.
```
