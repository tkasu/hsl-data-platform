use serde_json;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::{env, str, thread};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::time;

    #[test]
    fn test_apply_fn_to_chan() {
        let (sender, receiver) = sync_channel(10);
        let (f_sender, f_receiver) = sync_channel(10);

        thread::spawn(move || {
            apply_fn_to_chan(|x| x + 2, &receiver, &f_sender);
        });

        let val = 3;
        sender.send(val).unwrap();
        assert_eq!(5, f_receiver.recv().unwrap())
    }

    #[test]
    fn test_covert_to_json() {
        let (sender, receiver) = sync_channel(10);
        let (f_sender, f_receiver) = sync_channel(10);

        thread::spawn(move || {
            convert_to_json(&receiver, &f_sender);
        });

        let json_str = String::from("{\"name\":\"Tomi\",\"age\":31,\"class\":\"rust basics\"}");
        sender.send(json_str).unwrap();
        let json_value = f_receiver.recv().unwrap();
        assert_eq!("Tomi", json_value["name"].as_str().unwrap());
        assert_eq!(31, json_value["age"].as_u64().unwrap());
    }

    #[test]
    fn test_mqtt_reader() {
        // TODO This test is unstable, messages to test.mosquitto.org are not going through reliably
        use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
        use uuid::Uuid;

        let mqtt_host = String::from("test.mosquitto.org");
        let mqtt_port = 1883;
        let mqtt_topic = format!("/tomi/{}", Uuid::new_v4());

        let config = hsl_mqtt::MqttConfig {
            mqtt_host,
            mqtt_port,
            mqtt_topic,
        };

        let (sender, receiver) = sync_channel(10);
        let read_config = config.clone();
        thread::spawn(move || {
            hsl_mqtt::read_mqtt_feed(&sender, read_config);
        });

        thread::sleep(time::Duration::from_secs(3));

        let reconnection_options = ReconnectOptions::Always(2);
        let mqtt_options =
            MqttOptions::new("hsl", config.mqtt_host.as_str(), config.mqtt_port as u16)
                .set_keep_alive(10)
                .set_inflight(3)
                .set_request_channel_capacity(3)
                .set_reconnect_opts(reconnection_options)
                .set_clean_session(false);

        let (mut m, _) = MqttClient::start(mqtt_options).unwrap();

        let pub_handler = thread::spawn(move || {
            let timeout = time::Duration::from_millis(500);

            let msg = String::from("Hello from mosquitto!");
            m.publish(config.mqtt_topic.as_str(), QoS::AtLeastOnce, false, msg)
                .expect("First message send failed!");

            thread::sleep(timeout);
            let msg = String::from("Hello Again!");
            m.publish(config.mqtt_topic.as_str(), QoS::AtLeastOnce, false, msg)
                .expect("Second message send failed!");

            thread::sleep(time::Duration::from_secs(10));
        });

        pub_handler.join().unwrap();

        let mut expected: HashSet<String> = HashSet::new();
        expected.insert("Hello from mosquitto!".to_string());
        expected.insert("Hello Again!".to_string());

        let mut messages: HashSet<String> = HashSet::new();
        messages.insert(receiver.recv().unwrap());
        messages.insert(receiver.recv().unwrap());

        assert_eq!(expected, messages);
    }
}

mod hsl_mqtt {
    use super::*;
    use mqtt311;
    use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};

    #[derive(Debug, Clone)]
    pub struct MqttConfig {
        pub mqtt_host: String,
        pub mqtt_port: u32,
        pub mqtt_topic: String,
    }

    #[derive(Debug)]
    struct MqttPayloadError(String);

    impl fmt::Display for MqttPayloadError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "There is an error: {}", self.0)
        }
    }

    impl Error for MqttPayloadError {}

    fn extract_mqtt_payload(p: mqtt311::Publish) -> Result<String, MqttPayloadError> {
        let m = match str::from_utf8(&p.payload) {
            Ok(m) => m,
            Err(e) => return Result::Err(MqttPayloadError(e.to_string())),
        };
        Result::Ok(String::from(m))
    }

    pub fn read_mqtt_feed(sender: &SyncSender<String>, config: MqttConfig) -> () {
        let reconnection_options = ReconnectOptions::Always(10);
        let mqtt_options =
            MqttOptions::new("hsl", config.mqtt_host.as_str(), config.mqtt_port as u16)
                .set_keep_alive(10)
                .set_inflight(3)
                .set_request_channel_capacity(3)
                .set_reconnect_opts(reconnection_options)
                .set_clean_session(false);

        let (mut m, notifications) = MqttClient::start(mqtt_options).unwrap();

        m.subscribe(config.mqtt_topic.as_str(), QoS::AtLeastOnce)
            .unwrap();

        for msg in notifications {
            match msg {
                rumqtt::client::Notification::Publish(p) => {
                    let data = extract_mqtt_payload(p).expect("Payload extract failed for message");
                    sender.send(data).unwrap();
                }
                _ => println!("Skipping non Publish-message: {:?}", msg),
            }
        }
    }
}

mod kafka_pub {
    use super::*;
    use futures::Future;
    use rdkafka::config::ClientConfig;
    use rdkafka::producer::{FutureProducer, FutureRecord};

    #[derive(Debug, Clone)]
    pub struct KafkaConfig {
        pub kafka_host: String,
        pub kafka_topic: String,
    }

    pub fn kafka_sender(receiver: &Receiver<serde_json::Value>, config: KafkaConfig) {
        let kafka_producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.kafka_host)
            .set("produce.offset.report", "true")
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        loop {
            let next = receiver.recv().unwrap();
            let key = format!("{}-{}", next["route"], next["tst"]);
            let next = next.to_string();

            let delivery = kafka_producer.send(
                FutureRecord::to(config.kafka_topic.as_str())
                    .payload(next.as_bytes())
                    .key(key.as_str()),
                -1,
            );
            print!("Sent key {} to kafka... ", key);
            match delivery.wait().unwrap() {
                Ok((num1, num2)) => println!("delivery ok {} {}.", num1, num2),
                Err(e) => panic!("Delivery failed to kafka: {:?}", e),
            };
        }
    }
}

mod eventhub_pub {
    use super::*;
    use futures::Future;
    use rdkafka::config::ClientConfig;
    use rdkafka::producer::{FutureProducer, FutureRecord};
    use regex::Regex;

    #[derive(Debug, Clone)]
    pub struct EventHubConfig {
        pub connection_str: String,
        pub eventhub_name: String,
        pub ca_cert_path: String,
    }

    pub fn eventhub_sender(receiver: &Receiver<serde_json::Value>, config: EventHubConfig) {
        // Credits to rtyler, see: https://brokenco.de/2019/04/04/azure-eventhubs-rust.html
        let connection_str = config.connection_str.as_str();
        let re = Regex::new(r"Endpoint=sb://(?P<host>.*)/;(.*)")
            .expect("Error parsing Azure EventHub conection string");
        let caps = re.captures(connection_str).unwrap();

        let mut brokers = String::from(&caps["host"]);
        brokers.push_str(":9093");

        println!("Sending messages to: {}", &brokers);
        println!("Sending messages to eventhub: {}", &config.eventhub_name);

        let kafka_producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("produce.offset.report", "true")
            .set("message.timeout.ms", "5000")
            .set("security.protocol", "SASL_SSL")
            /*
             * The following setting -must- be set for librdkafka, but doesn't seem to do anything
             * worthwhile,. Since we're not relying on kerberos, let's just give it some junk :)
             */
            .set("sasl.kerberos.kinit.cmd", "echo 'who wants kerberos?!'")
            /*
             * Another unused setting which is required to be present
             */
            .set("sasl.kerberos.keytab", "keytab")
            .set("sasl.mechanisms", "PLAIN")
            /*
             * The username that Azure Event Hubs uses for Kafka is really this
             */
            .set("sasl.username", "$ConnectionString")
            /*
             * NOTE: Depending on your system, you may need to change this to adifferent location
             */
            .set("ssl.ca.location", config.ca_cert_path.as_str())
            .set("sasl.password", &connection_str)
            .set("client.id", "rust_hsl_producer")
            .create()
            .expect("Producer creation error");

        loop {
            let next = receiver.recv().unwrap();
            let key = format!("{}-{}", next["route"], next["tst"]);
            let next = next.to_string();

            let delivery = kafka_producer.send(
                FutureRecord::to(config.eventhub_name.as_str())
                    .payload(next.as_bytes())
                    .key(key.as_str()),
                -1,
            );
            print!("Sent key {} to Azure Event Hub... ", key);
            match delivery.wait().unwrap() {
                Ok((num1, num2)) => println!("delivery ok {} {}.", num1, num2),
                Err(e) => panic!("Delivery failed to Azure Event Hub: {:?}", e),
            };
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mqtt_config: hsl_mqtt::MqttConfig,
    pub endpoint_config: EndpointConfig,
}

#[derive(Debug, Clone)]
pub enum EndpointConfig {
    KafkaConfig(kafka_pub::KafkaConfig),
    EventHubConfig(eventhub_pub::EventHubConfig),
    DebugConfig,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        let mqtt_host = String::from("mqtt.hsl.fi");
        let mqtt_port = 1883;

        let default_ca_cert_path = String::from("/usr/lib/ssl/certs/ca-certificates.crt");
        let ca_cert_path = match env::var("HSL_CA_CERT_PATH") {
            Ok(cert_path) => {
                println!("Using ca_cert path from environment variable HSL_CA_CERT_PATH={}", cert_path);
                cert_path
            }
            Err(_) => {
                println!("Using default ca_cert_path={}", default_ca_cert_path);
                default_ca_cert_path
            }
        };

        args.next(); // skip the name of the program

        let mqtt_topic = match args.next() {
            Some(arg) => {
                if arg == "dev" {
                    // set default query string for development
                    String::from("/hfp/v2/journey/ongoing/vp/+/+/+/2543/1/#")
                } else {
                    arg
                }
            }
            None => return Err("Did not get mqtt topic string, expected mqtt topic or 'dev'"),
        };

        let endpoint_config = match args.next() {
            Some(arg) => {
                if arg == "debug" {
                    EndpointConfig::DebugConfig
                } else if arg == "kafka" {
                    let kafka_host = match args.next() {
                        Some(arg) => arg,
                        None => return Err("Did not get kafka hostname"),
                    };

                    let kafka_topic = match args.next() {
                        Some(arg) => arg,
                        None => return Err("Did not get kafka topic"),
                    };

                    EndpointConfig::KafkaConfig(kafka_pub::KafkaConfig {
                        kafka_host,
                        kafka_topic,
                    })
                } else if arg == "eventhub" {

                    let connection_str = match env::var("HSL_EVENTHUB_CONN") {
                        Ok(connection_str) => {
                            connection_str
                        }
                        Err(_) => {
                            return Err("Did not find environment variable HSL_EVENTHUB_CONN which is required!")
                        }
                    };

                    let eventhub_name = match args.next() {
                        Some(arg) => arg,
                        None => return Err("Did not get eventhub name"),
                    };

                    EndpointConfig::EventHubConfig(eventhub_pub::EventHubConfig {
                        connection_str,
                        eventhub_name,
                        ca_cert_path,
                    })
                } else {
                    return Err("Incorrect endpoint type, expected kafka, eventhub or debug");
                }
            }
            None => return Err("Did not get endpoint type, expected kafka, eventhub or debug"),
        };

        let next_arg = args.next();
        if next_arg.is_some() {
            return Err("Unexpected argument after endpoint arguments.");
        }

        let mqtt_config = hsl_mqtt::MqttConfig {
            mqtt_host,
            mqtt_port,
            mqtt_topic,
        };

        Ok(Config {
            mqtt_config,
            endpoint_config,
        })
    }
}

fn convert_to_json(receiver: &Receiver<String>, sender: &SyncSender<serde_json::Value>) {
    loop {
        let next = receiver.recv().unwrap();
        let value: serde_json::Value = serde_json::from_str(next.as_str()).unwrap();
        sender.send(value).unwrap();
    }
}

fn apply_fn_to_chan<T1, T2>(
    f: impl Fn(T1) -> T2,
    receiver: &Receiver<T1>,
    sender: &SyncSender<T2>,
) {
    loop {
        let next = receiver.recv().unwrap();
        let new = f(next);
        sender.send(new).unwrap();
    }
}

fn print_items<T: Debug>(receiver: &Receiver<T>) {
    loop {
        let next = receiver.recv().unwrap();
        println!("{:?}", next)
    }
}

pub fn run(config: Config) {
    let (raw_data_sender, raw_data_receiver) = sync_channel(100);
    let (json_data_sender, json_data_receiver) = sync_channel(100);
    let (transformer_sender, transformer_receiver) = sync_channel(100);

    let mqtt_config = config.clone().mqtt_config;
    thread::spawn(move || {
        hsl_mqtt::read_mqtt_feed(&raw_data_sender, mqtt_config);
    });

    thread::spawn(move || {
        convert_to_json(&raw_data_receiver, &json_data_sender);
    });

    thread::spawn(move || {
        apply_fn_to_chan(
            |x| x["VP"].clone(),
            &json_data_receiver,
            &transformer_sender,
        );
    });

    match config.endpoint_config {
        EndpointConfig::DebugConfig => print_items(&transformer_receiver),
        EndpointConfig::KafkaConfig(kafka_config) => {
            kafka_pub::kafka_sender(&transformer_receiver, kafka_config)
        },
        EndpointConfig::EventHubConfig(eventhub_config) => {
            eventhub_pub::eventhub_sender(&transformer_receiver, eventhub_config)
        }
    }
}
