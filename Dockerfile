FROM rust:slim-buster as builder

# trick to cache dependencies
WORKDIR /usr/src
RUN USER=root cargo new breathe
WORKDIR /usr/src/breathe
COPY Cargo.* ./
RUN cargo build --release

# copy the source and build
COPY src ./src
COPY resources ./resources
RUN cargo install --path .

FROM debian:buster-slim as runtime
RUN apt-get update \
 && apt-get install --auto-remove --no-install-recommends --no-install-suggests --show-upgraded --yes tini \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/breathe /usr/local/bin/breathe
COPY --from=builder /usr/src/breathe/resources/tests/config.toml /root/.config/breathe.toml
ENTRYPOINT [ "/usr/bin/tini", "--", "breathe"]
