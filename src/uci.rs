use crate::engine::EngineTask;
use std::io::{BufRead, Write, stdin, stdout};
use std::process::exit;
use std::str::SplitWhitespace;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::spawn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Uci,
    IsReady,
    Go(GoOptions),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoOptions {
    Infinite,
}

impl Command {
    pub fn parse(line: &str) -> Self {
        let mut tokens = line.split_whitespace();
        let cmd = match tokens.next() {
            Some(s) => s,
            None => return Command::Unknown,
        };

        match cmd {
            "uci" => Command::Uci,
            "isready" => Command::IsReady,
            "go" => Command::Go(GoOptions::parse(&mut tokens)),
            _ => Command::Unknown,
        }
    }
}

impl GoOptions {
    pub fn parse(opts: &mut SplitWhitespace) -> Self {
        let subcommand = opts.next();
        match subcommand {
            Some("infinite") => GoOptions::Infinite,
            Some(s) => {
                println!("unknown 'go' subcommand: {}", s);
                exit(1);
            }
            None => {
                println!("expected subcommand after 'go'");
                exit(1);
            }
        }
    }
}

pub fn executor(engine_tx: Sender<EngineTask>, engine_rx: Receiver<String>) {
    let (uci_tx, uci_rx) = channel::<String>();
    spawn(move || {
        input_loop(uci_tx);
    });

    spawn(move || output(engine_rx));

    for line in uci_rx {
        match Command::parse(&line) {
            Command::Uci => {
                println!("id name RoboCor 0.1");
                println!("id author JoelvanIngen");
                println!("uciok");
                stdout().flush().unwrap();
            }
            Command::IsReady => {
                println!("readyok");
                stdout().flush().unwrap();
            }
            Command::Go(options) => {
                engine_tx
                    .send(EngineTask { kind: options })
                    .expect("failed to send engine task");
            }
            Command::Unknown => {
                println!("unknown command: {}", line);
                exit(1);
            }
        }
    }
}

fn output(rx: Receiver<String>) {
    let mut stdout = stdout();

    while let Ok(line) = rx.recv() {
        println!("{}", line);
        stdout.flush().unwrap();
    }
}

fn input_loop(tx: Sender<String>) {
    let stdin = stdin();
    let mut handle = stdin.lock();
    let mut buff = String::new();

    while let Ok(bytes) = handle.read_line(&mut buff) {
        if bytes == 0 {
            break;
        }

        let line = buff.trim();
        tx.send(line.to_string()).expect("failed to send UCI input");

        buff.clear();
    }
}
