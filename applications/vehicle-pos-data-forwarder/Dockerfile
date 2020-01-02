FROM rust:1.40-buster as builder
WORKDIR /usr/src/vehicle-pos-data-forwarder
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/src/vehicle-pos-data-forwarder /usr/local/bin/vehicle-pos-data-forwarder
ENTRYPOINT ["/usr/local/bin/vehicle-pos-data-forwarder/target/release/vehicle-pos-data-forwarder"]
CMD ["dev", "debug"]
