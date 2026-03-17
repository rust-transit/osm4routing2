//! Error types for osm4routing.
//!
//! This module defines the error types that can be returned by the library.

use osmpbfreader::NodeId;
use std::fmt;

/// Errors that can occur when reading OSM data.
#[derive(Debug)]
pub enum Error {
    /// An I/O error occurred while reading the file.
    Io(std::io::Error),
    /// An error occurred while writing CSV output.
    Csv(csv::Error),
    /// A node referenced in a way was not found in the data.
    MissingNode(NodeId),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Csv(e) => write!(f, "CSV error: {}", e),
            Error::MissingNode(id) => write!(f, "Missing node with id: {}", id.0),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Csv(e) => Some(e),
            Error::MissingNode(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Self {
        Error::Csv(e)
    }
}
