use anyhow::Result;
use std::{
    env,
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
};

use crate::{
    packet::Packet,
    ping::{is_server_on, ping_server, send_wol},
    reader::Reader,
    status::get_offline_status,
    text::Text,
    writer::Writer,
};

pub mod packet;
pub mod ping;
pub mod reader;
pub mod status;
pub mod text;
pub mod writer;

fn main() -> Result<()> {
    let host_address = env::var("HOST_ADDRESS").unwrap_or("0.0.0.0".into());
    let host_port = env::var("HOST_PORT").unwrap_or("25565".into());

    let server_address = Arc::from(env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS is empty."));
    let server_port = env::var("SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(25565);

    let socket_address: SocketAddr = env::var("SERVER_IP")
        .unwrap_or(format!("{server_address}:{server_port}"))
        .parse()
        .expect("Server address or port is invalid.");

    let mac_address = mac_address_from_str(
        &env::var("SERVER_MAC_ADDRESS").expect("SERVER_MAC_ADDRESS is empty."),
    );

    let address = format!("{host_address}:{host_port}");
    let listener = TcpListener::bind(&address)?;
    println!("Listening on {address}");
    println!("Target server: {server_address}:{server_port}");

    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue;
        };

        let _ = stream
            .peer_addr()
            .inspect(|ip| println!("Request from {ip}"));

        let server_address = Arc::clone(&server_address);

        std::thread::spawn(move || {
            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);

            let mut v = 774;
            let mut next_state: Option<i64> = None;

            loop {
                let Ok(packet) = reader
                    .read_packet(next_state)
                    .inspect_err(|err| eprintln!("read packet err: {err}"))
                else {
                    break;
                };
                println!("Got Packet: {:?}", packet);

                let result = match packet {
                    Packet::Handshake { version, state, .. } => {
                        v = version;
                        next_state = Some(state);
                        Ok(false)
                    }
                    Packet::StatusRequest => send_status(&mut writer, &socket_address, v),
                    Packet::Ping(payload) => send_pong(&mut writer, payload,v),
                    Packet::Login(name, uuid) => {
                        if is_server_on(&socket_address) {
                            next_state = Some(4);
                            send_login_success(&mut writer, name, *uuid,v)
                        } else {
                            next_state = Some(5);
                            send_kick(&mut writer,v)
                        }
                    }

                    Packet::LoginAcknowledged => {
                        println!("Login succesfull!");
                        Ok(false)
                    }
                    Packet::Unknown if next_state == Some(4) => {
                        println!("Transfering client");
                        send_transfer(&mut writer, &server_address, server_port as u16,v)
                    }
                    _ => Ok(false),
                };
                let _ = writer.flush().inspect_err(|err| eprintln!("{err}"));

                match result.inspect_err(|e| eprintln!("{e}")) {
                    Err(_) | Ok(true) => {
                        break;
                    }
                    _ => (),
                }
            }

            if next_state == Some(5) {
                let _ = send_wol(&mac_address).inspect_err(|err| eprintln!("{err}"));
            }
        });
    }

    Ok(())
}

fn mac_address_from_str(str: &str) -> [u8; 6] {
    let a: Vec<u8> = str
        .split(":")
        .filter_map(|i| u8::from_str_radix(i, 16).ok())
        .collect();

    a.try_into().expect("Failed parsing mac_address.")
}

fn send_kick(stream: &mut BufWriter<&TcpStream>, version: i64) -> Result<bool> {
    stream.write_packet(
        &Packet::Disconnect(Text::literal(
            "Server is currently offline. Please wait while the server starts up.",
        )),
        version,
    )?;
    Ok(true)
}

fn send_transfer(
    stream: &mut BufWriter<&TcpStream>,
    address: &str,
    port: u16,
    version: i64,
) -> Result<bool> {
    stream.write_packet(&Packet::Transfer(address.into(), port.into()), version)?;
    Ok(true)
}

fn send_login_success(
    stream: &mut BufWriter<&TcpStream>,
    name: String,
    uuid: u128,
    version: i64,
) -> Result<bool> {
    stream.write_packet(&Packet::LoginSuccess(uuid.into(), name), version)?;
    Ok(false)
}

fn send_pong(stream: &mut BufWriter<&TcpStream>, payload: i64, version: i64) -> Result<bool> {
    stream.write_packet(&Packet::Ping(payload), version)?;
    Ok(true)
}

fn send_status(
    writer: &mut BufWriter<&TcpStream>,
    address: &SocketAddr,
    version: i64,
) -> Result<bool> {
    if let Ok(response) = ping_server(address).inspect_err(|e| eprintln!("{e}")) {
        writer.write_varint(response.len() as u64)?;
        writer.write_all(&response)?;
    } else {
        writer.write_packet(
            &Packet::StatusResponse(Box::new(get_offline_status(version))),
            version,
        )?;
    };

    Ok(false)
}
