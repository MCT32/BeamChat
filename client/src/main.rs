use std::{io::{self, Read, Write}, net::{Shutdown, TcpListener}};
use env_logger::Env;
use glam::{Quat, Vec3};
use log::{debug, error, info, trace, warn};
use serde_json::Value;
use serde::{Deserialize, de::Visitor};
use thiserror::Error;

const HOST: &str = "127.0.0.1:5555";

#[derive(Error, Debug)]
enum BeamChatError {
    #[error("Error opening TCP server")]
    TcpError(#[from] io::Error),
}

#[derive(Debug, Deserialize)]
struct Camera {
    position: Vec3,
    rotation: Quat,
}

#[derive(Debug, Deserialize)]
struct Packet {
    camera: Camera,
}

fn main() -> Result<(), BeamChatError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let listener = match TcpListener::bind(HOST) {
        Ok(listener) => listener,
        Err(err) => return Err(BeamChatError::TcpError(err)),
    };
    info!("Listening on {HOST}");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                error!("Incoming TCP connection failed: {err}");
                continue;
            },
        };

        // Might not matter if this fails
        let peer = match stream.peer_addr() {
            Ok(peer_addr) => peer_addr,
            Err(err) => {
                error!("Error getting address from connection: {err}");
                continue;
            },
        };

        info!("New connection: {peer}");

        let mut buf = [0u8; 1024];
        loop {
            let n = match stream.read(&mut buf) {
                Err(err) => {
                    error!("Error getting data from stream: {err}");
                    break;
                },
                Ok(bytes) => bytes,
            };

            if n == 0 { continue; };

            let packet: Packet = serde_json::from_slice(&buf[..n]).unwrap();

            trace!("Got packet: {packet:?}");
        }

        // Shouldn't matter if it fails?
        let _ = stream.shutdown(Shutdown::Both);
    }

    return Ok(());
}
