//! # fpop
//! 'fpop' is a collection of utilities to make file operaions more convenient.
//!
//! This library is made up of three main components:
//!
//! - [`FileOperations`]: the main struct.
//! - [`UnitOfInfo`]: make unit convertion easier.
//! - [`FileTypes`]: make treating file types easier.

mod file_operations;
mod file_types;
mod unit_of_information;
mod test;

pub use crate::{
    unit_of_information::UnitOfInfo,
    file_types::FileTypes,
    file_operations::Filey,
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    FileyError(anyhow::Error),
    #[error("{} No such file or directory", path)]
    NotFound { path: String },
    #[error("{} is not a directory", path)]
    NotADirectory { path: String },
}
