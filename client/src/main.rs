use std::{io::{self, BufRead, BufReader}, net::{TcpListener, UdpSocket}, thread};
use env_logger::Env;
use log::{error, info, trace};
use thiserror::Error;

use crate::{audio::audio_thread, packet::Packet};

mod packet;
mod audio;

const HOST: &str = "127.0.0.1:5555";

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let (game_data_sender, game_data_receiver) = std::sync::mpsc::channel::<Packet>();
    let audio_thread = thread::spawn(move || audio_thread(game_data_receiver));

    let socket = match UdpSocket::bind(HOST) {
        Ok(socket) => socket,
        Err(err) => {
            error!("Error binding UDP socket: {err}");
            return;
        },
    };
    info!("Listening on {HOST}");

    let mut buf = [0u8;1024];
    loop {
        match socket.recv(&mut buf) {
            Ok(size) => {
                let packet: Packet = match serde_json::from_slice(&buf[..size]) {
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
