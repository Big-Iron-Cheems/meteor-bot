# Build stage
FROM --platform=$BUILDPLATFORM rust:alpine AS builder

ARG TARGETARCH

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig

# Resolve the correct musl triple from BuildKit's TARGETARCH
RUN case "$TARGETARCH" in \
      amd64) echo "x86_64-unknown-linux-musl" ;; \
      arm64) echo "aarch64-unknown-linux-musl" ;; \
      *) echo "Unsupported arch: $TARGETARCH" >&2 && exit 1 ;; \
    esac > /rust-target.txt && \
    rustup target add "$(cat /rust-target.txt)"

# Set working directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Copy actual source and build
COPY src/ ./src/
RUN cargo build --release --locked --target "$(cat /rust-target.txt)"
RUN cp "target/$(cat /rust-target.txt)/release/meteor-bot" /meteor-bot

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create a non-root user
RUN adduser -D meteor

# Copy the built binary from the builder stage
COPY --from=builder /meteor-bot /usr/local/bin/meteor-bot

# Set user and entrypoint
USER meteor
CMD ["meteor-bot"]