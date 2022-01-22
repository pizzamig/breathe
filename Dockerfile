FROM lukemathwalker/cargo-chef:latest as planner
WORKDIR app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef:latest as cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:buster as builder
WORKDIR app
# copy the source and build
COPY . .
COPY --from=cacher /app/target /target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo install -vf --path .

FROM debian:buster-slim as runtime
RUN apt-get update \
 && apt-get install --auto-remove --no-install-recommends --no-install-suggests --show-upgraded --yes tini \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/breathe /usr/local/bin/breathe
COPY resources/tests/config.toml /root/.config/breathe.toml
ENTRYPOINT [ "/usr/bin/tini", "--", "breathe"]
