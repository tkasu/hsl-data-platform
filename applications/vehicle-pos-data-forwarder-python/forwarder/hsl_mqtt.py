import paho.mqtt.client as mqtt


class Connector:
    topic: str

    def __init__(self, topic: str):
        self.topic = topic

    def on_connect(self, client, userdata, flags, rc):
        print("Connected with result code " + str(rc))
        client.subscribe(self.topic)


class ConsoleReader:
    @staticmethod
    def on_read(client, userdata, msg):
        print(msg.topic + " " + str(msg.payload))


def start_mqtt(connector, reader):
    client = mqtt.Client()
    client.on_connect = connector.on_connect
    client.on_message = reader.on_read

    client.connect("mqtt.hsl.fi", 1883, 60)

    client.loop_forever()
