use crate::uci::Command;
use crate::uci::Command::{Go, Position};
use crate::uci::GoOptions::Infinite;
use crate::uci::PositionOptions::{Fen, StartPos};
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};

pub fn executor(uci_tx: Sender<String>, uci_rx: Receiver<Command>) {
    while let Ok(cmd) = uci_rx.recv() {
        match cmd {
            Go(opts) => match opts {
                Infinite => uci_tx
                    .send(String::from_str("Simulated engine start").unwrap())
                    .expect("failed to send message from engine to uci"),
            },
            Position(opts) => match opts {
                Fen(fen) => todo!("parse fen into board"),
                StartPos => todo!("init board from start"),
            },
            cmd => {
                panic!(
                    "engine received command from uci that shouldn't be passed to engine: {:?}",
                    cmd
                )
            }
        }
    }
}
