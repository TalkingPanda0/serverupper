use std::{
    io::{BufReader, Read},
    net::TcpStream,
};

use anyhow::{Result, bail};

use crate::{packet::Packet, status::Status};

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

pub trait Reader {
    fn read_u8(&mut self) -> Result<u8>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_i64(&mut self) -> Result<i64>;
    fn read_uuid(&mut self) -> Result<u128>;
    fn read_varint(&mut self) -> Result<i64>;
    fn read_varint_count(&mut self) -> Result<(i64, i64)>;
    fn read_string(&mut self) -> Result<String>;

    fn read_packet(&mut self, next_state: Option<i64>) -> Result<Packet>;
    fn read_clientbound_packet(&mut self) -> Result<Packet>;
    fn read_status_response(&mut self) -> Result<Packet>;
    fn read_handshake(&mut self) -> Result<Packet>;
    fn read_login(&mut self) -> Result<Packet>;
}

impl Reader for BufReader<&TcpStream> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf: u8 = 0;
        self.read_exact(std::slice::from_mut(&mut buf))?;
        Ok(buf)
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }
    fn read_uuid(&mut self) -> Result<u128> {
        let mut buf = [0u8; 16];
        self.read_exact(&mut buf)?;
        Ok(u128::from_be_bytes(buf))
    }

    fn read_varint(&mut self) -> Result<i64> {
        Ok(self.read_varint_count()?.0)
    }

    fn read_varint_count(&mut self) -> Result<(i64, i64)> {
        let mut value: i64 = 0;
        let mut position: i64 = 0;
        let mut count = 0;

        loop {
            let byte = self.read_u8()?;
            count += 1;

            value |= ((byte & SEGMENT_BITS) as i64) << position;

            if (byte & CONTINUE_BIT) == 0 {
                break;
            }
            position += 7;

            if position >= 32 {
                bail!("VarInt is too big.");
            }
        }

        Ok((value, count))
    }

    fn read_string(&mut self) -> Result<String> {
        let length = self.read_varint()? as usize;
        let mut buffer = vec![0; length];
        self.read_exact(&mut buffer)?;

        Ok(String::from_utf8(buffer)?)
    }

    fn read_packet(&mut self, next_state: Option<i64>) -> Result<Packet> {
        let length = self.read_varint()?;
        let (packet_id, id_length) = self.read_varint_count()?;
        match packet_id {
            0x00 if next_state.is_none() => self.read_handshake(),
            0x00 if next_state == Some(2) => self.read_login(),
            0x00 if next_state == Some(1) => Ok(Packet::StatusRequest),
            0x01 => self.read_i64().map(Packet::Ping),
            0x03 => Ok(Packet::LoginAcknowledged),
            n => {
                let left = length - id_length;
                /*let mut buf = vec![0u8; left as usize];
                self.read_exact(&mut buf)?;
                println!("{:#04X?}", buf);*/
                std::io::copy(&mut self.by_ref().take(left as u64), &mut std::io::sink())?;
                eprintln!("Unsupported packet id {n}");
                Ok(Packet::Unknown)
            }
        }
    }
    fn read_clientbound_packet(&mut self) -> Result<Packet> {
        let _length = self.read_varint()?;
        let packet_id = self.read_varint()?;
        if packet_id != 0 {
            bail!("Got a non status response client bound packet with id: {packet_id}");
        }
        self.read_status_response()
    }

    fn read_status_response(&mut self) -> Result<Packet> {
        let status: Status = serde_json::from_str(&self.read_string()?)?;

        Ok(Packet::StatusResponse(Box::new(status)))
    }

    fn read_handshake(&mut self) -> Result<Packet> {
        let version = self.read_varint()?;
        let address = self.read_string()?;
        let port = self.read_u16()?;
        let state = self.read_varint()?;

        Ok(Packet::Handshake {
            version,
            address,
            port,
            state,
        })
    }

    fn read_login(&mut self) -> Result<Packet> {
        let name = self.read_string()?;
        let uuid = self.read_uuid()?;

        Ok(Packet::Login(name, uuid))
    }
}
