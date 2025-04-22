# Ok Server

A minimal Rust web server that responds with `OK` on any HTTP request. It's designed to be a tiny, zero-dependency binary that listens on a configurable port.

## Features

- Single-source-file Rust server (no external crates)
- Configurable port via `PORT` environment variable (default: `8080`)
- Multi‑architecture Docker images (amd64, arm64, arm/v7) via Docker Buildx

## Usage

### Docker

```sh
# pull from Docker Hub
docker pull gnarr/ok:latest

# Run container:
docker run -d -p 8080:8080 \
-e PORT=8080 \
gnarr/ok:latest
```

### Build and run locally

```sh
# Compile in release mode
cargo build --release

# Run the server (default port 8080)
./target/release/ok

# Or specify a port:
PORT=9000 ./target/release/ok
```

Verify:


```sh
curl http://localhost:8080
# Should print 'OK'
```

### Docker Compose

Create a `docker-compose.yml` in your project root:

```yaml\
services:
  ok-server:
    image: gnarr/ok:latest
    ports:
      - "8080:8080"
    environment:
      - PORT=8080
```

Run:

```sh
docker-compose up -d
```

Then `curl http://localhost:8080` to see `OK`.