FROM rust:latest AS builder
WORKDIR /server

RUN apt update -y && apt install lld clang -y\
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY . .

#Set offline mod for sqlx 
ENV SQLX_OFFLINE true

#Build project
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /server

# Copy the compiled binary from the builder environment

COPY --from=builder /server/target/release/zero2prod zero2prod
# Copy configuration and migrations folders
COPY configuration configuration
COPY migrations migrations

ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]