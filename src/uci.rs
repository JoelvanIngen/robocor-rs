use std::io::{BufRead, Write, stdin, stdout};
use std::process::exit;
use std::str::SplitWhitespace;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::spawn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Uci,
    UciNewGame,
    IsReady,
    Position(PositionOptions),
    Go(GoOptions),
    Stop,
    Unknown,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoOptions {
    pub infinite: bool,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub depth: Option<u8>,
    pub movetime: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PositionInitType {
    StartPos,
    Fen(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionOptions {
    pub init_type: PositionInitType,
    pub moves: Option<Vec<String>>,
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
            "ucinewgame" => Command::UciNewGame,
            "isready" => Command::IsReady,
            "position" => Command::Position(PositionOptions::parse(&mut tokens)),
            "go" => Command::Go(GoOptions::parse(&mut tokens)),
            "stop" => Command::Stop,
            _ => Command::Unknown,
        }
    }
}

impl GoOptions {
    pub fn parse(opts: &mut SplitWhitespace) -> Self {
        let mut go_options = GoOptions::default();
        let mut subcommand = opts.next();
        if subcommand.is_none() {
            println!("expected subcommand after 'go'");
            exit(1);
        }

        while let Some(sub) = subcommand {
            // This can be shortened by a lot
            match sub {
                "infinite" => go_options.infinite = true,
                "wtime" => {
                    let time = match opts.next() {
                        Some(s) => s,
                        None => {
                            println!("expected value after 'wtime'");
                            exit(1);
                        }
                    };
                    go_options.wtime = Some(time.parse::<u64>().unwrap());
                }
                "btime" => {
                    let time = match opts.next() {
                        Some(s) => s,
                        None => {
                            println!("expected value after 'btime'");
                            exit(1);
                        }
                    };
                    go_options.btime = Some(time.parse::<u64>().unwrap());
                }
                "winc" => {
                    let time = match opts.next() {
                        Some(s) => s,
                        None => {
                            println!("expected value after 'winc'");
                            exit(1);
                        }
                    };
                    go_options.winc = Some(time.parse::<u64>().unwrap());
                }
                "binc" => {
                    let time = match opts.next() {
                        Some(s) => s,
                        None => {
                            println!("expected value after 'binc'");
                            exit(1);
                        }
                    };
                    go_options.binc = Some(time.parse::<u64>().unwrap());
                }
                "depth" => {
                    let depth = match opts.next() {
                        Some(s) => s,
                        None => {
                            println!("expected value after 'depth'");
                            exit(1);
                        }
                    };
                    go_options.depth = Some(depth.parse::<u8>().unwrap());
                }
                "movetime" => {
                    let time = match opts.next() {
                        Some(s) => s,
                        None => {
                            println!("expected value after 'movetime'");
                            exit(1);
                        }
                    };
                    go_options.movetime = Some(time.parse::<u64>().unwrap());
                }
                s => {
                    println!("unknown 'go' subcommand: {}", s);
                    exit(1);
                }
            }
            subcommand = opts.next();
        }
        go_options
    }
}

impl PositionOptions {
    fn parse(opts: &mut SplitWhitespace) -> Self {
        let subcommand = opts.next();
        let init_type = match subcommand {
            Some("startpos") => PositionInitType::StartPos,
            Some("fen") => {
                let fen = opts.take(6).collect::<Vec<_>>().join(" ");
                PositionInitType::Fen(fen)
            }
            Some(s) => {
                println!("unknown 'startpos' subcommand: {}", s);
                exit(1);
            }
            None => {
                println!("expected subcommand after 'startpos'");
                exit(1);
            }
        };

        let subcommand = opts.next();
        let moves = match subcommand {
            Some("moves") => Some(opts.map(|s| s.to_string()).collect::<Vec<String>>()),
            None => None,
            Some(s) => {
                println!("unknown 'position' subcommand: {}, expected 'moves'", s);
                exit(1);
            }
        };

        PositionOptions { init_type, moves }
    }
}

pub fn executor(engine_tx: Sender<Command>, engine_rx: Receiver<String>) {
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
            Command::Unknown => {
                println!("unknown command: {}", line);
                exit(1);
            }
            cmd => engine_tx.send(cmd).expect("failed to send engine task"),
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
