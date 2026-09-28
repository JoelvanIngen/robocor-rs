use std::num::ParseIntError;

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

#[derive(Debug)]
pub enum NotationError {
    InvalidSquare(String),
    InvalidNotation(String),
}
