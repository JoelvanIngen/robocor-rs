use std::io::{BufRead, stdin};
use std::process::exit;
use std::sync::mpsc;
use std::thread::spawn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Uci,
    IsReady,
    Unknown,
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
            _ => Command::Unknown,
        }
    }
}

pub fn executor() {
    let (tx, rx) = mpsc::channel::<String>();
    spawn(move || {
        input_loop(tx);
    });

    for line in rx {
        match Command::parse(&line) {
            Command::Uci => {
                println!("id name RoboCor 0.1");
                println!("id author JoelvanIngen");
                println!("uciok");
            }
            Command::IsReady => {
                println!("readyok");
            }
            Command::Unknown => {
                println!("unknown command: {}", line);
                exit(1);
            }
        }
    }
}

fn input_loop(tx: mpsc::Sender<String>) {
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
