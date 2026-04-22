use anyhow::Result;
use std::{
    env,
    io::{BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
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

    let server_address = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS is empty.");
    let server_port = env::var("SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(25565);

    let mac_address = mac_address_from_str(
        &env::var("SERVER_MAC_ADDRESS").expect("SERVER_MAC_ADDRESS is empty."),
    );

    let address = format!("{host_address}:{host_port}");
    let listener = TcpListener::bind(&address)?;
    println!("Listening on {address}.");

    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue;
        };

        let Ok(ip) = stream.peer_addr() else {
            continue;
        };
        println!("Request from {ip}");

        let server_address = server_address.clone();

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
                    Packet::StatusRequest => {
                        send_status(&mut writer, &server_address, server_port, v)
                    }
                    Packet::Ping(payload) => send_pong(&mut writer, payload),
                    Packet::Login(name, uuid) => {
                        if is_server_on(&server_address, server_port) {
                            next_state = Some(4);
                            send_login_success(&mut writer, name, uuid)
                        } else {
                            next_state = Some(5);
                            send_kick(&mut writer)
                        }
                    }

                    Packet::Unknown if next_state == Some(4) => {
                        send_transfer(&mut writer, &server_address, server_port)
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

fn send_kick(stream: &mut BufWriter<&TcpStream>) -> Result<bool> {
    stream.write_packet(&Packet::Disconnect(Text::literal(
        "Server is currently offline. Please wait while the server starts up.",
    )))?;
    Ok(true)
}

fn send_transfer(stream: &mut BufWriter<&TcpStream>, address: &str, port: u16) -> Result<bool> {
    stream.write_packet(&Packet::Transfer(address.into(), port.into()))?;
    Ok(true)
}

fn send_login_success(
    stream: &mut BufWriter<&TcpStream>,
    name: String,
    uuid: u128,
) -> Result<bool> {
    stream.write_packet(&Packet::LoginSuccess(uuid, name))?;
    Ok(false)
}

fn send_pong(stream: &mut BufWriter<&TcpStream>, payload: i64) -> Result<bool> {
    stream.write_packet(&Packet::Ping(payload))?;
    Ok(true)
}

fn send_status(
    writer: &mut BufWriter<&TcpStream>,
    address: &str,
    port: u16,
    version: i64,
) -> Result<bool> {
    let status = if let Ok(Packet::StatusResponse(response)) =
        ping_server(address, port).inspect_err(|e| eprintln!("{e}"))
    {
        response
    } else {
        Box::new(get_offline_status(version))
    };

    writer.write_packet(&Packet::StatusResponse(status))?;
    Ok(false)
}
