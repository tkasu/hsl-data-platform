#!/bin/bash
docker-compose up -d

sleep 10 # Wait for 10 seconds to the broker to be available
docker exec -it kafka_kafka_1 kafka-topics.sh --create --topic hsl --partitions 1 --replication-factor 1 --zookeeper zookeeper:2181
docker exec -it kafka_kafka_1 kafka-topics.sh --create --topic hsl-speed-stats --partitions 1 --replication-factor 1 --zookeeper zookeeper:2181