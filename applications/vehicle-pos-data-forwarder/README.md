# HSL vehicle position data forwarder

Small Rust application to get data vehicle position data from [HSL's High-frequency position API](https://digitransit.fi/en/developers/apis/4-realtime-api/vehicle-positions/) and forward it to different endpoints. 

## Installation

### Cargo

```bash
cargo install --path .
```

### Docker

```
docker build -t hsl .
```

## Usage

### Cargo

In the examples below, if you have built your application with `cargo install`, you can replace `cargo run`with the built binary.

#### Console output

Debug default development vehicle postion feed to console:

```bash 
cargo run <topic> debug
```

where <topic> can be either "dev" for default development feed or [HFP topic string](https://digitransit.fi/en/developers/apis/4-realtime-api/vehicle-positions/).  

#### Kafka

```bash 
cargo run <topic> kafka <kafka_host> <kafka_topic>
```

As kafka output is currently used only as local test env for Azure Event Hub's kafka protocol, authentication is not suppported.

#### Azure event hub

```bash 
cargo run <topic> eventhub <eventhub_name>
```

Additionally, following environment variable is required:

HSL_EVENTHUB_CONN='<your_connection_string>'

Also, if you ca certification file is not at /usr/lib/ssl/certs/ca-certificates.crt, you can use environment variable HSL_CA_CERT_PATH to set it correctly.

### Docker

Works similar to cargo, expect the application binary is the docker image entrypoint, ´docker run hsl <&args>' e.g.

```bash
docker run hsl dev debug
```

## License
[MIT](https://choosealicense.com/licenses/mit/)
