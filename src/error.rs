use std::{num::ParseIntError};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Fmt error: {0}")]
    Fmt(#[from] std::fmt::Error),

    #[error("ParseIntError {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}
