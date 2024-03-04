# Build stage
FROM rust:1.76-buster as builder

WORKDIR /app

# Accept build arguments
ARG DATABASE_URL

ENV DATABASE_URL=$DATABASE_URL

COPY . .

RUN cargo build --release

# Production stage
FROM debian:buster-slim

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/auth-endpoint .

CMD ["./auth-endpoint"]