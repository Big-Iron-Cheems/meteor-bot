# Create a build image
FROM golang:1.23-alpine AS build

WORKDIR /app

# Download dependencies
COPY go.mod go.sum .
RUN go mod download

# Build bot
COPY . .
RUN go build -ldflags="-s -w" -o build/meteor-bot cmd/meteor-bot/main.go

# Create a runtime image
FROM alpine:latest

WORKDIR /app

COPY --from=build /app/build/meteor-bot .

CMD ["./meteor-bot"]
