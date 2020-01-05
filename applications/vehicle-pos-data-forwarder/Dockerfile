FROM rust:1.40-buster as builder
WORKDIR /usr/src/vehicle-pos-data-forwarder
COPY . .
RUN apt-get update && apt-get install -y libssl-dev libsasl2-dev # for rdkafka sls slas
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/src/vehicle-pos-data-forwarder /usr/local/bin/vehicle-pos-data-forwarder
RUN apt-get update && apt-get install -y libssl-dev libsasl2-dev ca-certificates # for rdkafka sls slas
ENV HSL_CA_CERT_PATH /etc/ssl/certs/ca-certificates.crt
ENTRYPOINT ["/usr/local/bin/vehicle-pos-data-forwarder/target/release/vehicle-pos-data-forwarder"]
CMD ["dev", "debug"]
