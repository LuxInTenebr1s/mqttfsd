use std::io::{Error, ErrorKind};
use tokio::sync::mpsc::Sender;
use std::collections::HashMap;
use mqtt3::{Packet, Connect, Connack, ConnectReturnCode};

use super::client::Client;

pub struct Broker {
    pub session_list: HashMap<String, Client>,
}

impl Broker {
    pub fn new() -> Self {
        Broker {
            session_list: HashMap::new(),
        }
    }

    pub async fn handle_connect(&mut self, connect: Box<Connect>, sender: Sender<Packet>) -> Result<(), Error> {
        let id = &connect.client_id;
        let mut session_present = false;

        let session = match self.session_list.get_mut(id) {
            Some(s) => {
                if connect.clean_session {
                    s.renew(sender)
                } else {
                    session_present = true;
                    s.reconnect(sender)
                }
            },
            None => {
                self.session_list.insert(id.clone(), Client::new(id.clone(), sender));
                match self.session_list.get_mut(id) {
                    Some(s) => s,
                    None => return Err(Error::new(ErrorKind::Other, "it's impossible")),
                }
            },
        };

        session.set_lastwill(connect.last_will.clone());
        session.set_keepalive(connect.keep_alive);

        let connack = Connack {
            session_present,
            code: ConnectReturnCode::Accepted,
        };
        session.send(Packet::Connack(connack)).await;

        return Ok(())
    }
}
