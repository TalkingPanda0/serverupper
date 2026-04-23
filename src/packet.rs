use std::{fmt::{Debug, Formatter}, io::Write};

use anyhow::Result;
use crate::{status::Status, text::Text, writer::Writer};
use derive_more::{Deref,From};


#[derive(Deref,From)]
pub struct UUID(pub u128);

impl Debug for UUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:x}", self.0))
    }
}



#[derive(Debug)]
pub enum Packet {
    Handshake {
        version: i64,
        address: String,
        port: u16,
        state: i64,
    },
    StatusResponse(Box<Status>),
    Ping(i64),
    Login(String, UUID),
    LoginSuccess(UUID, String),
    LoginAcknowledged,
    StatusRequest,
    Transfer(String, u64),
    Disconnect(Text),
    Unknown,
}

impl Packet {
    pub fn bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        match self {
            Self::StatusResponse(status) => {
                buffer.write_string(&serde_json::to_string(status)?)?;
            }
            Self::Ping(payload) => {
                buffer.write_all(&payload.to_be_bytes())?;
            }
            Self::LoginSuccess(uuid, name) => {
                buffer.write_uuid(uuid)?;
                buffer.write_string(name)?;
                buffer.write_varint(0)?;
            }
            Self::Transfer(host, port) => {
                buffer.write_string(host)?;
                buffer.write_varint(*port)?;
            }

            Self::Handshake {
                version,
                address,
                port,
                state,
            } => {
                buffer.write_varint(*version as u64)?;
                buffer.write_string(address)?;
                buffer.write_u16(*port)?;
                buffer.write_varint(*state as u64)?;
            }

            Self::Disconnect(text) => {
                buffer.write_string(&serde_json::to_string(text)?)?;
            }

            Self::Unknown | Self::LoginAcknowledged | Self::Login(..) | Self::StatusRequest => (),
        }

        Ok(buffer)
    }
    pub fn packet_id(&self) -> i64 {
        match self {
            Self::Unknown => 0x00,
            Self::Handshake { .. } => 0x00,
            Self::StatusRequest => 0x00,
            Self::StatusResponse(_) => 0x00,
            Self::Ping(_) => 0x01,
            Self::Login(..) => 0x00,
            Self::LoginSuccess(..) => 0x02,
            Self::LoginAcknowledged => 0x03,
            Self::Transfer(..) => 0x0B,
            Self::Disconnect(_) => 0x00,
        }
    }
}
