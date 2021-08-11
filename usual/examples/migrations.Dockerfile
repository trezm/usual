###########
# Stage 0 #
###########
FROM rust:1.46 as cargo-build

WORKDIR /src/app

RUN apt-get update
RUN apt-get install -y openssl libpq-dev
RUN rustup update && \
    rustup default nightly && \
    rustup target add armv7-unknown-linux-gnueabihf

RUN cargo install diesel_cli --no-default-features --features postgres

###########
# Stage 1 #
###########
FROM debian:buster-slim

RUN apt-get update
RUN apt-get install libpq-dev -y

WORKDIR /app/migrations
COPY ./migrations ./migrations
ENV DATABASE_URL postgres://postgres:postgres@localhost/test

COPY --from=cargo-build /usr/local/cargo/bin/diesel /usr/local/bin/diesel

CMD ["diesel", "migration", "run"]
