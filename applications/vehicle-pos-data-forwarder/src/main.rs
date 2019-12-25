use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::thread;
use mosquitto_client;

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

fn print_channel_items(receiver: &Receiver<String>) {
    loop {
        let next = receiver.recv().unwrap();
        println!("{}", next)
    }
}

fn main() {
    let (sender, receiver) = sync_channel(100);

    thread::spawn(move|| {
        read_mqtt_feed(&sender).unwrap();
    });
    print_channel_items(&receiver);
}
