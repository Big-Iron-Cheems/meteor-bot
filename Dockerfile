# Build stage
FROM rust:alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig

# Set working directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Copy actual source and build
COPY src/ ./src/

# Build release binary with musl target
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create a non-root user
RUN adduser -D meteor

# Copy the built binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/meteor-bot /usr/local/bin/meteor-bot

# Set user and entrypoint
USER meteor
CMD ["meteor-bot"]