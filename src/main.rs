use tokio::net::{TcpListener};
use tokio::io;
use tokio::stream::StreamExt;
use tokio::sync::{Mutex, mpsc};
use std::sync::Arc;

use futures::sink::SinkExt;
use tokio_util::codec::{FramedRead, FramedWrite};

use mqtt3::Packet;

pub mod codec;
use codec::MqttCodec;

pub mod client;

pub mod broker;
use broker::Broker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("0.0.0.0:1883").await?;
    let broker = Arc::new(Mutex::new(Broker::new()));

    while let Some(Ok(tcpstream)) = listener.next().await {
        let broker = broker.clone();

        println!("new connection: {}", tcpstream.peer_addr().unwrap());
        let (stream, sink) = io::split(tcpstream);
        let (sender, mut receiver) = mpsc::channel::<Packet>(10);

        // packet sending future
        tokio::spawn(async move {
            let mut sink = FramedWrite::new(sink, MqttCodec::new());
            while let Some(i) = receiver.recv().await {
                println!("sending new packet: {:?}", i);
                sink.send(i).await.unwrap();
            }
        });

        // packet receiving future
        tokio::spawn(async move {
            let mut stream = FramedRead::new(stream, MqttCodec::new());

            // first control packet must be connect packet according to spec
            if let Some(Ok(Packet::Connect(c))) = stream.next().await {
                broker.lock().await.handle_connect(c, sender).await.unwrap();
            } else {
                println!("no proper CONNECT package, shutting down connection");
                return;
            }

            println!("now waiting for packets back");

            while let Some(packet) = stream.next().await {
                match packet {
                    Err(e) => { println!("got a err: {:?}", e); return; },
                    _ => println!("here comes the new packet: {:?}", packet),
                }
            }
        });
    }
    Ok(())
}
