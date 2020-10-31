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

# Destroy kafka

```
./destroy_kafka.sh
```