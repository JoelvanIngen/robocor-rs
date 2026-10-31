use crate::uci::GoOptions;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineTask {
    pub kind: GoOptions,
}

pub fn executor(uci_tx: Sender<String>, uci_rx: Receiver<EngineTask>) {
    while let Ok(task) = uci_rx.recv() {
        let EngineTask { kind } = task;
        match kind {
            GoOptions::Infinite => {
                uci_tx
                    .send(String::from_str("Simulated engine start").unwrap())
                    .expect("failed to send message from engine to uci");
            }
        }
    }
}
