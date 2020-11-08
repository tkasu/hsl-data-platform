from confluent_kafka import Producer


class MqttWrapper:
    debug: bool

    def __init__(self, producer):
        self.producer = producer
        self.debug = False

    def on_read(self, client, userdata, msg):
        if self.debug:
            print(msg.topic + " " + str(msg.payload))
        self.producer.deliver_messages([str(msg.payload)])

    @classmethod
    def from_producer(cls, producer):
        return cls(producer)


class StringProducer:
    producer: Producer
    topic: str

    def __init__(self, servers: str, topic: str):
        self.producer = Producer({"bootstrap.servers": servers})
        self.topic = topic

    @staticmethod
    def delivery_report(err, msg):
        """Called once for each message produced to indicate delivery result.
        Triggered by poll() or flush().
        Copied from: https://github.com/confluentinc/confluent-kafka-python
        """
        if err is not None:
            print("Message delivery failed: {}".format(err))
        else:
            print("Message delivered to {} [{}]".format(msg.topic(), msg.partition()))

    def deliver_messages(self, messages):
        self.producer.poll(0)

        for msg in messages:
            self.producer.produce(
                self.topic, msg.encode("UTF-8"), callback=self.delivery_report
            )

        self.producer.flush()
