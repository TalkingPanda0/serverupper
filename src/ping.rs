use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::{TcpStream, UdpSocket},
};

use anyhow::{Result};

use crate::{packet::Packet, reader::Reader, writer::Writer};

pub fn ping_server(address: &str, port: u16) -> Result<Vec<u8>> {
    let stream = TcpStream::connect(format!("{address}:{port}"))?;

    let mut writer = BufWriter::new(&stream);
    let mut reader = BufReader::new(&stream);

    writer.write_packet(&Packet::Handshake {
        version: 776,
        address: address.into(),
        port,
        state: 1,
    })?;
    writer.write_packet(&Packet::StatusRequest)?;
    writer.flush()?;

    let length = reader.read_varint()?;

    let mut buf = vec![0u8; length as usize];
    reader.read_exact(&mut buf)?;

    Ok(buf)
}

pub fn is_server_on(address: &str, port: u16) -> bool {
    TcpStream::connect(format!("{address}:{port}")).is_ok()
}

pub fn send_wol(mac_address: &[u8; 6]) -> Result<()> {
    println!("Sending wol...");
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    let mut buffer: Vec<u8> = Vec::with_capacity(102);
    buffer.extend_from_slice(&[0xFF; 6]);
    buffer.extend(mac_address.repeat(16));

    socket.send_to(&buffer, "255.255.255.255:9")?;
    Ok(())
}
