# Start from the official Go image
FROM golang:1.23-alpine AS builder

# Set the Current Working Directory inside the container
WORKDIR /app

# Copy go.mod and go.sum files to the workspace
COPY go.mod go.sum ./

# Download all dependencies. Dependencies will be cached if the go.mod and go.sum files are not changed
RUN go mod download

# Copy the source code from the current directory to the Working Directory inside the container
COPY . .

# Build the Go app
RUN go build -ldflags="-s -w" -o build/meteor-bot cmd/meteor-bot/main.go

FROM alpine:latest

# Run the binary program produced by `go build`
CMD ["/app/meteor-bot"]