from forwarder.hsl_mqtt import start_mqtt, Connector, ConsoleReader
from forwarder.kafka_producers import StringProducer, MqttWrapper


def main():
    connector = Connector("/hfp/v2/journey/#")

    # processor = ConsoleReader()

    producer = StringProducer("localhost:9092", "hsl-dev")
    processor = MqttWrapper.from_producer(producer)
    # processor.debug = True

    start_mqtt(connector, processor)


if __name__ == "__main__":
    main()
