# Changelog

All notable changes to this project will be documented in this file.

## [0.4.4] - 2026-05-19

### 🐛 Bug Fixes

- Harden release workflow and avoid slow body reads

## [0.4.3] - 2025-11-26

### ⚙️ Miscellaneous Tasks

- *(ci)* Simplify artifact name extraction with grep
- Update changelog for v0.4.3
- Release ok version 0.4.3

## [0.4.2] - 2025-11-26

### 🐛 Bug Fixes

- *(ci)* Use Rust 2021 edition for release builds

### ⚙️ Miscellaneous Tasks

- *(ci)* Let cargo release handle version bump and lockfile updates
- Update changelog for v0.4.2
- Release ok version 0.4.2

## [0.4.1] - 2025-11-26

### 🚀 Features

- Add health-check
- Add body read deadline and validation (#5)
- Retry across workers before dropping connection (#6)

### ⚙️ Miscellaneous Tasks

- Skip release when tag already exists
- Speed up release builds with matrix and caches
- Include rustc in cache key and cache windows build
- Align crate version, add sccache for release builds
- Fix release workflow outputs and cache keys
- Fix docker job dependencies after matrix change
- Ensure cargo-edit is installed for set-version
- Fix workflow ordering for cargo-edit install
- Update changelog for v0.4.1

## [0.3.2] - 2025-04-28

### ⚙️ Miscellaneous Tasks

- Update dockerhub-description to version 4
- Update changelog for v0.3.2
- Release ok version 0.3.2

## [0.3.1] - 2025-04-28

### ⚙️ Miscellaneous Tasks

- Add badges to readme
- Add links to badges
- Remove backslash from block marker
- Fix wording in curl example
- Change environment to map in compose example
- Add configuration table
- Add quick start guide
- *(build)* Add automatic README update for docker hub
- Change description wording to be more precise
- Use webserver everywhere
- Update changelog for v0.3.1
- Release ok version 0.3.1

## [0.3.0] - 2025-04-27

### 🚀 Features

- Respond with 404 to unknown paths
- Add `SHOW_FAVICON` environment variable with default value of `true`
- Use `sync_channel` for logs with try_send
- THREAD_POOL_SIZE is now configurable via env
- Respond correctly to HEAD requests

### 🐛 Bug Fixes

- Do not attempt to consume body on GET for / or /favicon.ico
- Avoid per-line heap allocations when parsing Content-Length
- Gracefully handle a disconnected worker queue
- Validate PORT env var before binding
- Handle try_send errors explicitly
- Return 501 for unsupported HTTP methods

### 🚜 Refactor

- Simplify get_client_address
- Simplify sanitize method
- Use format macro instead of string.into
- Use string.into for port default value
- Use match ok/err for getting content-length header

### ⚡ Performance

- Skip allocating on heap for method and path
- Add 24h caching header for favicon

### ⚙️ Miscellaneous Tasks

- Formatting fixes
- Update changelog for v0.3.0
- Release ok version 0.3.0

## [0.2.0] - 2025-04-25

### 🚀 Features

- Add request logging
- Add favicon
- Create a thread pool with 4 threads
- Better error responses
- Add `X-Content-Type-Options` and `X-Frame-Options` security headers
- Sanitize request_line before printing to logs
- Get client ip-address from X-Forwarded-For with fallback to stream.peer_addr
- Switch to per-worker bounded channels and round-robin dispatch
- Connection handling panic isolation
- Reject chunked encoding
- Logger thread to prevent log message intermixing

### 🐛 Bug Fixes

- Potential Slowloris attacks
- Improve performance by having a fixed size buffer for reading
- Improve Slowloris mitigation
- Mitigate TCP receive buffer stalling attacks
- Use sync_channel with queue capacity of 100
- Log dropped connections
- Return error on when remaining > 0 and n == 0
- Parse headers in lower-case
- Treat premature EOF while reading headers as an error, not success

### 🚜 Refactor

- Add generalized write_response method

### ⚡ Performance

- Pre-build byte slices

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.2.0
- Release ok version 0.2.0

## [0.1.10] - 2025-04-23

### 🐛 Bug Fixes

- Use distroless/cc instead of distroless/static

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.1.10
- Release ok version 0.1.10

## [0.1.9] - 2025-04-23

### 🐛 Bug Fixes

- Set ok_server executable

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.1.9
- Release ok version 0.1.9

## [0.1.8] - 2025-04-23

### 🐛 Bug Fixes

- User hard-coded binary name

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.1.8
- Release ok version 0.1.8

## [0.1.7] - 2025-04-23

### ⚙️ Miscellaneous Tasks

- Remove actions-rs/toolchain@v1
- Update changelog for v0.1.7
- Release ok version 0.1.7

## [0.1.6] - 2025-04-23

### 🐛 Bug Fixes

- Name extraction for Windows build

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.1.6
- Release ok version 0.1.6

## [0.1.5] - 2025-04-23

### 🐛 Bug Fixes

- Only move the file

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.1.5
- Release ok version 0.1.5

## [0.1.4] - 2025-04-23

### 🐛 Bug Fixes

- Remove TARGETARCH override and correct arm-v7 binary name

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.1.4
- Release ok version 0.1.4

## [0.1.3] - 2025-04-23

### 🐛 Bug Fixes

- Update Dockerfile to handle TARGETVARIANT

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.1.3
- Release ok version 0.1.3

## [0.1.2] - 2025-04-22

### 🐛 Bug Fixes

- *(ci)* Release needed for docker_image step

### ⚙️ Miscellaneous Tasks

- Update changelog for v0.1.2
- Release ok version 0.1.2

## [0.1.1] - 2025-04-22

### ⚙️ Miscellaneous Tasks

- Init
- Update changelog for v0.1.1
- Release ok version 0.1.1

<!-- generated by git-cliff -->
