# Ok — minimal zero-dependency Rust webserver

[![docker arch][8]][7]
[![docker size][9]][7]
[![docker pulls][10]][7]
[![docker version][11]][12]

[![release and build][3]][4]
![unsafe](https://img.shields.io/badge/unsafe-0%25-success)
[![license][5]][6]
![rustc](https://img.shields.io/badge/rustc-1.74%2B-orange)
![release](https://img.shields.io/github/v/release/gnarr/ok?include_prereleases)

[![contributions][1]][2]

[1]: https://img.shields.io/badge/contributions-welcome-brightgreen
[2]: https://github.com/gnarr/ok
[3]: https://img.shields.io/github/actions/workflow/status/gnarr/ok/release.yml?branch=main&label=release%20and%20build
[4]: https://github.com/gnarr/ok/actions/workflows/release.yml
[5]: https://img.shields.io/badge/license-MIT-blue
[6]: https://github.com/gnarr/ok/blob/main/LICENSE

[7]: https://hub.docker.com/r/gnarr/ok
[8]: https://img.shields.io/badge/platform-amd64%20%7C%20arm64%20%7C%20armv7-brightgreen
[9]: https://img.shields.io/docker/image-size/gnarr/ok/latest
[10]: https://img.shields.io/docker/pulls/gnarr/ok
[11]: https://img.shields.io/docker/v/gnarr/ok?sort=semver
[12]: https://hub.docker.com/r/gnarr/ok/tags


A minimal Rust webserver that responds with `OK` on HTTP request. It's designed to be a tiny, zero-dependency binary that listens on a configurable port.

## Quick Start

```sh
docker run -d -p 8080:8080 gnarr/ok
curl http://localhost:8080
# OK
```

## Features

- Single-source-file Rust server (no external crates)
- Multi‑architecture Docker images (amd64, arm64, arm/v7) via Docker Buildx

## Configuration

Variable | Description | Default
|---|---|---|
PORT | TCP port the server binds to | 8080 
SHOW_FAVICON | Whether to serve /favicon.ico (set to "false" to disable it) | true (any value other than "false" enables it) 
THREAD_POOL_SIZE | Number of worker threads to spawn (falls back to CPU count or 4) | CPU core count (via available_parallelism) or 4 if unknown 

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
# Should return 'OK' with HTTP 200
```

### Docker Compose

Create a `docker-compose.yml` in your project root:

```yaml
services:
  ok-server:
    image: gnarr/ok:latest
    ports:
      - "8080:8080"
    environment:
      PORT: 8080
```

Run:

```sh
docker-compose up -d
```

Then `curl http://localhost:8080` to see `OK`.
