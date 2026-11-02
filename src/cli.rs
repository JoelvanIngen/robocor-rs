use clap::{Arg, Command};
use std::process::exit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Behaviour {
    Default,
    Perft(u8),
}

fn cli() -> Command {
    Command::new("robocor").about("A chess engine").subcommand(
        Command::new("perft").about("Run perft test").arg(
            Arg::new("depth")
                .help("Depth to calculate to")
                .required(true),
        ),
    )
}

pub fn parse() -> Behaviour {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("perft", submatches)) => {
            let depth = submatches
                .get_one::<String>("depth")
                .unwrap()
                .parse::<u8>()
                .unwrap_or_else(|_| {
                    println!("Please provide a valid number (integer)");
                    exit(1);
                });
            Behaviour::Perft(depth)
        }
        None => Behaviour::Default,
        _ => unreachable!(),
    }
}
