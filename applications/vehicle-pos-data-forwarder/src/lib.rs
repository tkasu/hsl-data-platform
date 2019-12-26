use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::thread;
use mosquitto_client;
use serde_json;
use std::fmt::Debug;

fn read_mqtt_feed(sender: &SyncSender<String>) -> Result<(), mosquitto_client::Error> {
    let m = mosquitto_client::Mosquitto::new("hsl");

    //tls port 8883 not working, how to set cert-file correctly?
    //m.tls_set("/etc/ssl/certs/", "/etc/ssl/certs/", "/etc/ssl/certs/", None);
    m.connect("mqtt.hsl.fi", 1883)?;

    let vehicle_postions = m.subscribe("/hfp/v2/journey/ongoing/vp/+/+/+/2543/1/#", 0)?;

    let mut mc = m.callbacks(());
    mc.on_message(|_,msg| {
        if vehicle_postions.matches(&msg) {
            let data = msg.text().to_string();
            sender.send(data).unwrap();
        }
    });

    m.loop_forever(200)
}

fn convert_to_json(receiver: &Receiver<String>, sender: &SyncSender<serde_json::Value>) {
    loop {
        let next = receiver.recv().unwrap();
        let value: serde_json::Value = serde_json::from_str(next.as_str()).unwrap();
        sender.send(value).unwrap();
    }
}

fn apply_fn_to_chan<T1, T2>(f: impl Fn(T1) -> T2, receiver: &Receiver<T1>, sender: &SyncSender<T2>) {
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

pub fn run() {
    let (raw_data_sender, raw_data_receiver) = sync_channel(100);
    let (json_data_sender, json_data_receiver) = sync_channel(100);
    let (transformer_sender, transformer_receiver) = sync_channel(100);

    thread::spawn(move|| {
        read_mqtt_feed(&raw_data_sender).unwrap();
    });

    thread::spawn(move|| {
        convert_to_json(&raw_data_receiver, &json_data_sender);
    });

    thread::spawn(move|| {
        apply_fn_to_chan(|x| x["VP"].clone(), &json_data_receiver, &transformer_sender);
    });

    print_items(&transformer_receiver);
}
