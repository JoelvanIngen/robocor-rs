use std::num::ParseIntError;

#[allow(dead_code)]
#[derive(Debug)]
pub enum FenError {
    MissingField(String),
    InvalidBoardSetup(String),
    InvalidTurn(String),
    InvalidCastlingRights(char),
    InvalidHalfmoveClock(ParseIntError),
    InvalidFullmoveNumber(ParseIntError),
    InvalidFenPiece(char),
    NotationError(NotationError),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum NotationError {
    InvalidSquare(String),
    InvalidNotation(String),
}
