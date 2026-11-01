use crate::board::Board;
use crate::error::FenError;
use crate::uci::Command::{Go, Position};
use crate::uci::GoOptions::Infinite;
use crate::uci::PositionOptions::{Fen, StartPos};
use crate::uci::{Command, GoOptions, PositionOptions};
use std::process::exit;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};

pub struct Engine {
    uci_tx: Sender<String>,
    uci_rx: Receiver<Command>,
    board: Option<Board>,
}

impl Engine {
    pub fn new(uci_tx: Sender<String>, uci_rx: Receiver<Command>) -> Engine {
        Engine {
            uci_tx,
            uci_rx,
            board: None,
        }
    }

    pub fn start(&mut self) {
        while let Ok(cmd) = self.uci_rx.recv() {
            match cmd {
                Go(opts) => self.handle_go(opts),
                Position(opts) => self.handle_position(opts),
                cmd => {
                    panic!(
                        "engine received command from uci that shouldn't be passed to engine: {:?}",
                        cmd
                    )
                }
            }
        }
    }

    fn handle_go(&mut self, opts: GoOptions) {
        match opts {
            Infinite => self
                .uci_tx
                .send(String::from_str("Simulated engine start").unwrap())
                .expect("failed to send message from engine to uci"),
        }
    }

    fn handle_position(&mut self, opts: PositionOptions) {
        match opts {
            Fen(fen) => {
                self.board = Some(
                    Board::from_fen(fen.as_str()).unwrap_or_else(|e| self.handle_error(fen, e)),
                )
            }
            StartPos => self.board = Some(Board::start_position()),
        }
    }

    fn handle_error(&mut self, fen: String, e: FenError) -> ! {
        self.uci_tx
            .send(format!("invalid fen {fen}: {:?}", e))
            .expect("failed send fen error message from engine to uci");
        exit(1);
    }
}
