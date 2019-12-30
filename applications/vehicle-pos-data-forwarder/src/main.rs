use vehicle_pos_data_forwarder::{Config, run};

fn main() {
    let mqtt_host = String::from("mqtt.hsl.fi");
    let mqtt_port = 1883;
    let mqtt_topic = String::from("/hfp/v2/journey/ongoing/vp/+/+/+/2543/1/#");
    let kafka_host = String::from("localhost:9092");
    let kafka_topic = String::from("test");
    
    let config = Config{mqtt_host, mqtt_port, mqtt_topic, kafka_host, kafka_topic};
    run(config)
}
