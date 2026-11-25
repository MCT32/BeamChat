use std::{io::{self, BufRead, BufReader}, net::TcpListener, thread};
use env_logger::Env;
use log::{error, info, trace};
use thiserror::Error;

use crate::{audio::audio_thread, packet::Packet};

mod packet;
mod audio;

const HOST: &str = "127.0.0.1:5555";

#[derive(Error, Debug)]
enum BeamChatError {
    #[error("Error opening TCP server")]
    TcpError(#[from] io::Error),
}

fn main() -> Result<(), BeamChatError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let (game_data_sender, game_data_receiver) = std::sync::mpsc::channel::<Packet>();
    let audio_thread = thread::spawn(move || audio_thread(game_data_receiver));

    let listener = match TcpListener::bind(HOST) {
        Ok(listener) => listener,
        Err(err) => return Err(BeamChatError::TcpError(err)),
    };
    info!("Listening on {HOST}");

    for stream in listener.incoming() {
        let stream = match stream {
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

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        loop {
            line.clear();

            match reader.read_line(&mut line) {
                // Zero bytes means stream has closed
                Ok(0) => {
                    info!("TCP stream closed, waiting for new connection...");
                    break;
                },
                Ok(_) => {
                    let packet: Packet = match serde_json::from_str(line.trim_end()) {
                        Ok(packet) => packet,
                        Err(err) => {
                            error!("Invalid JSON line: {err}");
                            continue;
                        }
                    };

                    trace!("Got packet: {packet:?}");
                    game_data_sender.send(packet).expect("Cannot send data to audio thread, the threat probably paniced");
                },
                Err(err) => {
                    error!("Error getting data from stream: {err}");
                    break;
                },
            };
        }
    }

    return Ok(());
}
