# Build stage
FROM rust:alpine AS builder

# Install necessary build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

# Set working directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -rf src/

# Copy actual source and build
COPY src/ ./src/
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest
RUN apk add --no-cache ca-certificates
RUN adduser -D meteor

# Copy the built binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/meteor-bot /usr/local/bin/meteor-bot

# Set user and entrypoint
USER meteor
CMD ["meteor-bot"]