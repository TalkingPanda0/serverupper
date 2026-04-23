use std::io::Write;

use anyhow::Result;

use crate::packet::{Packet};

const SEGMENT_BITS: u64 = 0x7F;
const CONTINUE_BIT: u64 = 0x80;

pub trait Writer {
    fn write_u8(&mut self, value: u8) -> Result<()>;
    fn write_u16(&mut self, value: u16) -> Result<()>;

    fn write_varint(&mut self, value: u64) -> Result<()>;
    fn write_uuid(&mut self, value: &u128) -> Result<()>;
    fn write_string(&mut self, value: &str) -> Result<()>;

    fn write_packet(&mut self, packet: &Packet) -> Result<()>;
}

impl<T: Write> Writer for T {
    fn write_u8(&mut self, value: u8) -> Result<()> {
        self.write_all(std::slice::from_ref(&value))?;
        Ok(())
    }
    fn write_u16(&mut self, value: u16) -> Result<()> {
        self.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    fn write_varint(&mut self, mut value: u64) -> Result<()> {
        loop {
            if (value & !SEGMENT_BITS) == 0 {
                self.write_u8(value as u8)?;
                break;
            }
            self.write_u8(((value & SEGMENT_BITS) | CONTINUE_BIT) as u8)?;
            value >>= 7;
        }
        Ok(())
    }

    fn write_string(&mut self, value: &str) -> Result<()> {
        self.write_varint(value.len() as u64)?;
        self.write_all(value.as_bytes())?;
        Ok(())
    }

    fn write_uuid(&mut self, value: &u128) -> Result<()> {
        self.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    fn write_packet(&mut self, packet: &Packet) -> Result<()> {
        let packet_data = packet.bytes()?;
        let mut packet_id: Vec<u8> = Vec::new();
        packet_id.write_varint(packet.packet_id() as u64)?;
        let length = packet_data.iter().len() + packet_id.len();

        self.write_varint(length as u64)?;
        self.write_all(&packet_id)?;
        self.write_all(&packet_data)?;

        Ok(())
    }
}
