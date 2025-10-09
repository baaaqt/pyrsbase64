use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug, Clone)]
pub enum DecodeError {
    InvalidInputLength(usize),
    InvalidSymbol(usize, u8),
    TooManyPaddings(usize),
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInputLength(length) => {
                write!(f, "Expected length to be divisible by 4, got {}", length)
            }
            Self::InvalidSymbol(length, c) => {
                write!(f, "Invalid symbol at position {}: {}", length, c)
            }
            Self::TooManyPaddings(count) => {
                write!(f, "Too many paddings: {}", count)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum BaseNError {
    Decode(DecodeError),
}

impl Display for BaseNError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseNError::Decode(e) => write!(f, "Decode error: {}", e),
        }
    }
}

impl Error for BaseNError {}
