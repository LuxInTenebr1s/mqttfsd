use tokio::sync::mpsc::Sender;
use mqtt3::{LastWill, Packet};

pub struct Client {
    id: String,
    keep_alive: u16,
    last_will: Option<LastWill>,
    sender: Option<Sender<Packet>>,
}

const KEEP_ALIVE_DEF: u16 = 30;

impl Client {
    pub fn new(id: String, sender: Sender<Packet>) -> Client {
        Client {
            id,
            sender: Some(sender),
            keep_alive: KEEP_ALIVE_DEF,
            last_will: None,
        }
    }

    pub fn renew(&mut self, sender: Sender<Packet>) -> &mut Self {
        self.sender = Some(sender);
        self
    }

    pub fn reconnect(&mut self, sender: Sender<Packet>) -> &mut Self {
        self.sender = Some(sender);
        self
    }

    pub fn set_lastwill(&mut self, last_will: Option<LastWill>) {
        self.last_will = last_will;
    }

    pub fn set_keepalive(&mut self, keep_alive: u16) {
        if keep_alive == 0 {
            self.keep_alive = KEEP_ALIVE_DEF;
        } else {
            self.keep_alive = keep_alive;
        }
    }

    pub async fn send(&mut self, packet: Packet) {
        if let Some(sender) = &mut self.sender {
            sender.send(packet).await.unwrap();
        }
    }
}
