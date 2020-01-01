use mqtt311;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use serde_json;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::time::Duration;
use std::{str, thread, time};

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
    use uuid::Uuid;

    let mqtt_host = String::from("test.mosquitto.org");
    let mqtt_port = 1883;
    let mqtt_topic = format!("/tomi/{}", Uuid::new_v4());
    let kafka_host = String::from("kafka_dummy_host");
    let kafka_topic = String::from("kafka_dummy_port");
    let config = Config {
        mqtt_host,
        mqtt_port,
        mqtt_topic,
        kafka_host,
        kafka_topic,
    };

    let (sender, receiver) = sync_channel(10);
    let read_config = config.clone();
    thread::spawn(move || {
        read_mqtt_feed(&sender, read_config);
    });

    thread::sleep(time::Duration::from_secs(5));

    let reconnection_options = ReconnectOptions::Always(10);
    let mqtt_options = MqttOptions::new("hsl", config.mqtt_host.as_str(), config.mqtt_port as u16)
        .set_keep_alive(10)
        .set_inflight(3)
        .set_request_channel_capacity(3)
        .set_reconnect_opts(reconnection_options)
        .set_clean_session(false);

    let (mut m, _) = MqttClient::start(mqtt_options).unwrap();

    thread::spawn(move || {
        let timeout = time::Duration::from_millis(500);
        thread::sleep(timeout);

        let msg = String::from("Hello from mosquitto!");
        m.publish(config.mqtt_topic.as_str(), QoS::AtLeastOnce, false, msg)
            .expect("First message send failed!");

        thread::sleep(timeout);
        let msg = String::from("Hello Again!");
        m.publish(config.mqtt_topic.as_str(), QoS::AtLeastOnce, false, msg)
            .expect("Second message send failed!");
    });

    let mut expected: HashSet<String> = HashSet::new();
    expected.insert("Hello from mosquitto!".to_string());
    expected.insert("Hello Again!".to_string());

    let mut messages: HashSet<String> = HashSet::new();
    messages.insert(receiver.recv().unwrap());
    messages.insert(receiver.recv().unwrap());

    assert_eq!(expected, messages);
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mqtt_host: String,
    pub mqtt_port: u32,
    pub mqtt_topic: String,
    pub kafka_host: String,
    pub kafka_topic: String,
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

fn read_mqtt_feed(sender: &SyncSender<String>, config: Config) -> () {
    let reconnection_options = ReconnectOptions::Always(10);
    let mqtt_options = MqttOptions::new("hsl", config.mqtt_host.as_str(), config.mqtt_port as u16)
        .set_keep_alive(10)
        .set_inflight(3)
        .set_request_channel_capacity(3)
        .set_reconnect_opts(reconnection_options)
        .set_clean_session(false);

    let (mut m, notifications) = MqttClient::start(mqtt_options).unwrap();

    let vehicle_postions = m
        .subscribe(config.mqtt_topic.as_str(), QoS::AtLeastOnce)
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

pub fn kafka_sender(receiver: &Receiver<serde_json::Value>, config: Config) {
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

        kafka_producer.send(
            FutureRecord::to(config.kafka_topic.as_str())
                .payload(next.as_bytes())
                .key(key.as_str()),
            -1,
        );
        // TODO How to correctly handle future results?
    }
}

pub fn run(config: Config) {
    let (raw_data_sender, raw_data_receiver) = sync_channel(100);
    let (json_data_sender, json_data_receiver) = sync_channel(100);
    let (transformer_sender, transformer_receiver) = sync_channel(100);

    let mqtt_config = config.clone();
    thread::spawn(move || {
        read_mqtt_feed(&raw_data_sender, mqtt_config);
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

    kafka_sender(&transformer_receiver, config)
    //print_items(&transformer_receiver);
}
