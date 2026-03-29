# Stage 1: Build
FROM rust:alpine AS builder

# Install build dependencies for Rust and OpenSSL/C-libraries if needed
RUN apk add --no-cache musl-dev

WORKDIR /usr/src/commit-check
COPY . .

# Build the release binary
RUN cargo build --release

# Stage 2: Runtime
FROM alpine:latest

# Copy the binary from the builder stage
COPY --from=builder /usr/src/commit-check/target/release/commit-check /usr/local/bin/commit-check

# Set the working directory for the CI environment
WORKDIR /workspace

# Run the validator by default
ENTRYPOINT ["commit-check"]
CMD ["--help"]
