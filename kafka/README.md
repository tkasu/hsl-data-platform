# Setup local Kafka broker for testing

* Prequisites
    * Docker
    * docker-compose

# Start kafka and create a topic called hsl

```
./create_kafka.sh
```

# Start HSL data reader

See:

https://github.com/tkasu/hsl-data-platform/tree/master/applications/vehicle-pos-data-forwarder

Example (assuming that the working directory is <your-clone-folder>/applications/vehicle-pos-data-forwarder)

```
cargo run dev kafka localhost:9092 hsl
```

# See that the events are are working

```
docker exec -it kafka_kafka_1  kafka-console-consumer.sh --topic hsl --bootstrap-server localhost:9093
```

# Launch Stream runner

Run e.g. VehicleSpeedAnalysisStream in applications/vehicle-pos-kafka-streams

# Read average speed for each vehicle

docker exec -it kafka_kafka_1  kafka-console-consumer.sh --topic hsl-speed-stats --bootstrap-server localhost:9093 --property print.key=true --property value.deserializer=org.apache.kafka.common.serialization.DoubleDeserializer

# Destroy kafka

```
./destroy_kafka.sh
```