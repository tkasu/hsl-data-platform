package main

import (
	"encoding/json"
	"fmt"
	mqtt "github.com/eclipse/paho.mqtt.golang"
	"net/url"
	"time"
)

func getMqttConnection(url *url.URL) (mqtt.Client, error) {
	opts := mqtt.NewClientOptions()
	opts.AddBroker(url.String())

	client := mqtt.NewClient(opts)
	token := client.Connect()
	for !token.WaitTimeout(3 * time.Second) {
	}
	if err := token.Error(); err != nil {
		err = fmt.Errorf("mqtt connection error: %s", err)
		return nil, err
	}
	return client, nil
}

func mqttSub(client mqtt.Client, topic string, handler mqtt.MessageHandler) {
	client.Subscribe(topic, 0, handler)
}


func getConsoleEndpoint() func([]byte) {
	f := func(bs []byte) {
		fmt.Printf("Received message: %s\n", bs)
	}
	return f
}

func chanForwarder(msgChan chan []byte) mqtt.MessageHandler {
	f := func(client mqtt.Client, m mqtt.Message) {
		msgChan <- m.Payload()
	}
	return f
}

func jsonEncoder(inChan chan []byte, outChan chan []byte, errChan chan error) {
	for {
		msgRaw := <- inChan
		msg, err := json.Marshal(string(msgRaw))
		if err != nil {
			errChan <- err
		} else {
			outChan <- msg
		}
	}
}

func jsonDecoder(inChan chan []byte, outChan chan []byte, errChan chan error) {
	var resType string
	for {
		msg := <- inChan
		err := json.Unmarshal(msg, &resType)
		if err != nil {
			errChan <- err
		} else {
			outChan <- msg
		}
	}
}

func main() {
	mqttUrlRaw := "tcp://mqtt.hsl.fi:1883"
	topic := "/hfp/v2/journey/ongoing/vp/+/+/+/2543/1/#"

	mqttUrl, err := url.Parse(mqttUrlRaw)
	if err != nil {
		fmt.Println("Error parsing mqtt url: ", mqttUrl)
		fmt.Println(err)
		return
	}
	fmt.Println("Using mqtt server: ", mqttUrl.Host)

	client, err := getMqttConnection(mqttUrl)
	if err != nil {
		fmt.Println("Error connecting to mqtt broker: ", err)
	}

	rawMsgChan := make(chan []byte)
	jsonMsgChan := make(chan []byte)
	outChan := make(chan []byte)
	errChan :=  make(chan error)
	forwarder := chanForwarder(rawMsgChan)
	go mqttSub(client, topic, forwarder)
	go jsonEncoder(rawMsgChan, jsonMsgChan, errChan)
	go jsonDecoder(jsonMsgChan, outChan, errChan)

	msgEndpoint := getConsoleEndpoint()
	errHandler := func(e error) {
		fmt.Println(e)
	}

	for {
		select {
			case msg := <-outChan:
				msgEndpoint(msg)
			case err := <- errChan:
				errHandler(err)
		}
	}
}
