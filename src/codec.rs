use tokio_util::codec::{Decoder, Encoder};
use std::io::{self, Cursor, BufReader, Error, ErrorKind};
use bytes::BytesMut;

use mqtt3::{Packet, MqttRead, MqttWrite};

pub struct MqttCodec;

impl MqttCodec {
    pub fn new() -> MqttCodec {
        MqttCodec {}
    }
}

impl Decoder for MqttCodec {
    type Item = Packet;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Packet>, io::Error> {
        let len = src.len();
        if len < 2 {
            return Ok(None);
        }

        let mut buf = BufReader::new(Cursor::new(&src));

        match buf.read_packet() {
            Ok(v) => {
                src.split_to(len);
                Ok(Some(v))
            }
            Err(mqtt3::Error::Io(e)) => Err(e),
            Err(e) => Err(Error::new(ErrorKind::Other, format!("mqtt3 err: {:?}", e))),
        }
    }
}

impl Encoder<Packet> for MqttCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), io::Error> {
        let mut buf = Cursor::new(Vec::new());

        match buf.write_packet(&item) {
            Ok(()) => Ok(dst.extend_from_slice(buf.get_ref())),
            Err(e) => Err(Error::new(ErrorKind::Other, format!("mqtt3 encode err: {:?}", e))),
        }
    }
}
