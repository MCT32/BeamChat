use std::{io::{self, Read, Write}, net::TcpListener};
use env_logger::Env;
use log::{info, debug, error};
use thiserror::Error;

const HOST: &str = "127.0.0.1:5555";

#[derive(Error, Debug)]
enum BeamChatError {
    #[error("Error opening TCP server")]
    TcpError(#[from] io::Error),
}

enum ConnectionState {
    Listening,
    Connected,
}

fn main() -> Result<(), BeamChatError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let listener = match TcpListener::bind(HOST) {
        Ok(listener) => listener,
        Err(err) => return Err(BeamChatError::TcpError(err)),
    };
    info!("Listening on {HOST}");

    let mut connection_state = ConnectionState::Listening;

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
        debug!("New connection: {peer}");

        let mut buf = [0u8; 1024];
        loop {
            // Connection code here
        }
    }

    return Ok(());
}
