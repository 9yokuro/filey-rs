//! # filey
//! 'filey' is a collection of utilities to make file operaions more convenient.
//!
//! This library is made up of three main components:
//!
//! - [`Filey`]: the main struct.
//! - [`UnitOfInfo`]: make unit convertion easier.
//! - [`FileTypes`]: make treating file types easier.

mod file_operations;
mod file_types;
mod test;
mod unit_of_information;

pub use crate::{file_operations::Filey, file_types::FileTypes, unit_of_information::UnitOfInfo};

/// Filey's error type.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    FileyError(anyhow::Error),
    #[error("{} No such file or directory", path)]
    NotFound {
        path: String,
    },
    #[error("{} is not a directory", path)]
    NotADirectory {
        path: String,
    },
}
