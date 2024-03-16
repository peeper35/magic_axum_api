FROM rust:1.76.0 as build

# create a new empty shell project
RUN USER=root cargo new --bin magic_axum_api
WORKDIR /magic_axum_api

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/magic_axum_api*
RUN cargo build --release

FROM debian:bullseye-stable
COPY --from=build /magic_axum_api/target/release/magic_axum_api .
CMD ["./magic_axum_api"]