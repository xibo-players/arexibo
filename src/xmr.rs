// Xibo player Rust implementation, (c) 2022-2024 Georg Brandl.
// Licensed under the GNU AGPL, version 3 or later.

//! Receive, decrypt and handle incoming XMR messages from CMS.

use anyhow::{bail, Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use byteorder::{BE, ReadBytesExt};
use crossbeam_channel::{Receiver, Sender, unbounded};
use rsa::RsaPrivateKey;
use serde::{Deserialize, Deserializer, de::Error};
use serde_json::from_slice;
use time::{OffsetDateTime, Duration};
use std::net::TcpStream;
use std::io::{Read, Write};
use crate::config::CmsSettings;

/// Possible messages to forward to the collect thread.
#[derive(Debug)]
pub enum Message {
    CollectNow,
    Screenshot,
    Purge,
    WebHook(String),
    Command(String),
}

pub struct Manager {
    private_key: RsaPrivateKey,
    sender: Sender<Message>,
    socket: ZmqSubSocket,
}

const HEARTBEAT: &[u8] = b"H";

impl Manager {
    pub fn new(settings: &CmsSettings, connect: &str,
               private_key: RsaPrivateKey) -> Result<(Self, Receiver<Message>)> {
        let channel = settings.xmr_channel();
        let mut socket = ZmqSubSocket::connect(connect).context("connecting XMR socket")?;
        socket.subscribe(channel.as_bytes())?;
        socket.subscribe(HEARTBEAT)?;
        let (sender, receiver) = unbounded();

        Ok((Self {
            private_key,
            sender,
            socket,
        }, receiver))
    }

    pub fn run(mut self) {
        loop {
            if let Err(e) = self.process_msg() {
                log::error!("handling XMR message: {:#}", e);
            }
        }
    }

    fn process_msg(&mut self) -> Result<()> {
        let (channel, more) = self.socket.recv_frame()?;
        assert!(more);
        let (key, more) = self.socket.recv_frame()?;
        assert!(more);
        let (content, more) = self.socket.recv_frame()?;
        assert!(!more);
        if &*channel != HEARTBEAT {
            let json_msg = JsonMessage::new(&self.private_key, &key, &content)?;
            log::debug!("got XMR message: {:?}", json_msg);
            if let Some(msg) = json_msg.into_msg() {
                self.sender.send(msg).unwrap();
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct JsonMessage {
    action: String,
    #[serde(rename = "createdDt")]
    #[serde(deserialize_with = "deserialize_datetime")]
    created: OffsetDateTime,
    #[serde(default)]
    ttl: i64,
    #[serde(rename = "triggerCode")]
    #[serde(default)]
    trigger_code: Option<String>,  // for webhooks
    #[serde(rename = "commandCode")]
    #[serde(default)]
    command_code: Option<String>,  // for commands
}

impl JsonMessage {
    fn new(private_key: &RsaPrivateKey, key: &[u8], content: &[u8]) -> Result<Self> {
        let enc_key = BASE64.decode(key)?;
        let mut msg = BASE64.decode(content)?;
        let msg_key = decrypt_private_key(&enc_key, private_key)?;
        arc4::Arc4::with_key(&msg_key).encrypt(&mut msg);
        Ok(from_slice(&msg)?)
    }

    fn is_expired(&self) -> bool {
        self.created + Duration::seconds(self.ttl) < OffsetDateTime::now_utc()
    }

    fn into_msg(self) -> Option<Message> {
        if self.is_expired() {
            return None;
        }
        match &*self.action {
            "collectNow" => Some(Message::CollectNow),
            // we treat this the same as a collect, which will re-send the pubkey
            "rekeyAction" => Some(Message::CollectNow),
            "screenShot" => Some(Message::Screenshot),
            "purgeAll" => Some(Message::Purge),
            "triggerWebhook" => self.trigger_code.map(Message::WebHook),
            "commandAction" => self.command_code.map(Message::Command),
            _ => {
                log::info!("got unsupported XMR action {:?}", self.action);
                None
            }
        }
    }
}

fn deserialize_datetime<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<OffsetDateTime, D::Error> {
    let s = <String as Deserialize>::deserialize(d)?;
    OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
        .map_err(|_| D::Error::custom("invalid datetime string"))
}

fn decrypt_private_key(enc_key: &[u8], private_key: &RsaPrivateKey) -> Result<Vec<u8>> {
    let dec_data = private_key.decrypt(rsa::Pkcs1v15Encrypt, enc_key).context("failed to decrypt PK")?;
    Ok(dec_data)
}

struct ZmqSubSocket(TcpStream);

/// Implementation of ZMTP as far as we need it for XMR. We don't want to pull in the
/// `zmq` crate since it is almost unmaintained.
impl ZmqSubSocket {
    fn connect(uri: &str) -> Result<Self> {
        let rx = regex::Regex::new("tcp://([^:]*):([0-9]+)").context("invalid validation Regex")?;
        let caps = rx.captures(uri).context("invalid XMR connect URI")?;
        let host = caps.get(1).expect("present").as_str();
        let port = caps[2].parse().expect("digits");

        let mut stream = TcpStream::connect((host, port))?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;

        // greeting: signature, version (3.0), security (none) and server flag (no),
        // then pad to 64 bytes
        stream.write_all(b"\xff\x00\x00\x00\x00\x00\x00\x00\x01\x7f\
                           \x03\x00\
                           NULL\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                           \x00\
                           \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                           \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")?;
        // read greeting from peer
        let mut buf = [0; 64];
        stream.read_exact(&mut buf)?;
        if buf[0] != 0xff || buf[9] != 0x7f || buf[10] != 0x03 || &buf[12..16] != b"NULL" {
            bail!("ZMTP greeting not understood");
        }

        // send ready command
        stream.write_all(b"\x04\x19\x05READY\x0bSocket-Type\x00\x00\x00\x03SUB")?;
        // read ready command
        stream.read_exact(&mut buf[..2])?;
        if buf[0] != 0x04 {
            bail!("ZMTP command frame not understood");
        }
        let len = buf[1] as usize;
        if len >= 62 {
            bail!("ZMTP command frame too long");
        }
        stream.read_exact(&mut buf[2..2+len])?;
        if &buf[2..8] != b"\x05READY" {
            bail!("ZMTP READY command not understood");
        }

        // now we're ready to receive frames
        stream.set_read_timeout(None)?;
        Ok(Self(stream))
    }

    fn subscribe(&mut self, topic: &[u8]) -> Result<()> {
        let mut msg = Vec::with_capacity(3 + topic.len());
        msg.push(0);  // single-frame message, short length
        msg.push(1 + topic.len() as u8);  // length of msg
        msg.push(1);  // subscribe command
        msg.extend_from_slice(topic);
        self.0.write_all(&msg)?;
        Ok(())
    }

    fn recv_frame(&mut self) -> Result<(Vec<u8>, bool)> {
        let flags = self.0.read_u8()?;
        let more = flags & 1 != 0;
        let long_len = flags & 2 != 0;
        let len = if long_len {
            self.0.read_u64::<BE>()? as usize
        } else {
            self.0.read_u8()? as usize
        };
        let mut result = vec![0; len];
        self.0.read_exact(&mut result)?;
        Ok((result, more))
    }
}

// #[test]
// fn test_zmq() {
//     let mut socket = ZmqSubSocket::connect("tcp://localhost:5555").unwrap();
//     socket.subscribe(b"test").unwrap();
//     let first = socket.recv_frame().unwrap();
//     let second = socket.recv_frame().unwrap();
//     assert!(first.1);
//     assert!(!second.1);
//     assert_eq!(&*first.0, b"test");
//     assert_eq!(&*second.0, b"content");
// }

#[test]
fn test_decrypt() {
    let pem = "-----BEGIN RSA PRIVATE KEY-----
MIICXAIBAAKBgQDJg84myV3VE+v53gQKVbX+6pQrveSfZTcs/a3mikxhXO32peqh
OP2namgoixfBBwK6wzRjRzOHdsB4yQPTMRTZIsipTYHyIqYl5/6AxoRGAsjZtmaB
MNsxrBxMCGlWEKLPwSCecT8EbCrfl3GArf56SEglxDRyx7pDRRnAihPgMQIBEQKB
gAQ7xwUeC6blhxvWaX8kOIeBs4QlVXmrABVh1Wa5wzfTs0BXYoJPt+IsL11bH7E7
TpQO23QaPD4Ba03U5TCJotumgDf0zIfVx5p7GrpK4oqI4o+PX7gWCzurXaqmQiYq
CfZCCeHF+Z2KV2OmhXq3tvlx8Ne4gOiZ65K2vNhNiAEZAkEA1wAyT/hFPUoDnqYD
UfRJEQM1XyRxa0MTkUJh4UO+WCp+d2OtEuydMUdfSu9oGPUNPsMaXr3SzsE8rhp8
1iXB1QJBAO/xQqxO0YvYnDJgQFTXB34Lv66pCHkbBddvYnByfxqeIQJM9o61grUK
LCLjrZ9qPqa87xcYLPP4i8/iPuMKtu0CQQCXw+dHghLB2eRv/LcMrG/PxgeOdBPT
PmgqTPnML9GnpYZyZHoredhfBTQ05Tpr+EWVtuVwDYW/Hv2oErJ5C5fhAkEAm0HB
usmWpchlEYmTCbhQJGH0gBMFe4n0uJNd0EoWAioVW9dyXFdUk0LRQ8B/ZyahAnpA
WjzRywo8WVYosQbu1QJBAIK8lUC6fBRr2ElLltNV/cmR2To5rUYSQJJB9rDw9Inv
cwFD2YnuxuF9szIeWPTmHUl6aXRIByuKNexbHqTeNhY=
-----END RSA PRIVATE KEY-----
";
    use rsa::pkcs1::DecodeRsaPrivateKey;
    let privkey = rsa::RsaPrivateKey::from_pkcs1_pem(pem).unwrap();
    let msg = JsonMessage::new(&privkey,
                               b"uKgfpneak5Qx5vppLlJZEEcFQ5Y/xrk45ysmnsIVQGvndFR0R86pPRRDPxvqSBgCDb\
                                 4xInqC8fQLApEzEjULL4QwERycgfHWMY+KSAEDjaS2/3IvSUPa+XYZVZssC/jddIar\
                                 ZvqHdfylHqm1IiL6Tgaps05BYeyDYynRmngW8NM=",
                               b"TOwhZC5mz2N0GoQvUDXsXVDfC3A6Ov5I+raxOsBvvhOLgPFlpz2VxWTsvq5TX8JJ/b\
                                 gCSdfpe5DTA0bEvwXzDst1KtGjK1Nvdg==").unwrap();
    assert_eq!(msg.action, "screenShot");
}
