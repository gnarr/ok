# syntax=docker/dockerfile:1

# Runtime-only Dockerfile: uses prebuilt binaries from host context
# Build with buildx by passing TARGETARCH and BINARY_NAME

FROM gcr.io/distroless/cc:nonroot

# Name of the binary (without extension)
ARG BINARY_NAME=ok_server
# Architecture (amd64, arm64, armv7)
ARG TARGETARCH
# Variant for architectures like arm (v7)
ARG TARGETVARIANT

# Default port
ENV PORT=8080

# Copy the appropriate prebuilt binary and set executable
COPY --chmod=0755 \
  binaries/linux-${TARGETARCH}${TARGETVARIANT:+-${TARGETVARIANT}}/ok_server \
  /usr/local/bin/ok_server

EXPOSE 8080
USER nonroot

ENTRYPOINT ["/usr/local/bin/ok_server"]

HEALTHCHECK \
  --interval=30s \
  --timeout=5s \
  --start-period=10s \
  --retries=3 \
  CMD ["/usr/local/bin/ok_server","--health-check"]
