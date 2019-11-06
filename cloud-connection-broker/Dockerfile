FROM rust:1.35 as build

WORKDIR /rumqttd

COPY ./Cargo.* ./
COPY ./src ./src

RUN cargo build --release

FROM rust:1.35

ADD conf/rumqttd.conf /etc/rumqttd.conf
COPY --from=build /rumqttd/target/release/rumqttd /rumqttd

ENTRYPOINT ["/rumqttd"]