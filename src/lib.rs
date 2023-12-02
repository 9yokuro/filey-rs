//! # filey
//! 'filey' is a collection of utilities to make file operations more convenient.
//!
//! This library is made up of three main components:
//!
//! - [`Filey`]: the main struct.
//! - [`UnitOfInfo`]: make unit convertion easier.
//! - [`FileTypes`]: make treating file types easier.
//!
//! # A Basic example
//! ```
//! use filey::Filey;
//! # use std::error::Error;
//! #
//! # fn examples() -> Result<(), Box<Error>> {
//! use filey::{Filey, FileTypes};
//!
//! // Create a new file.
//! let file = Filey::new(".great_app.conf").create(FileTypes::File)?;
//!
//! // Two months later...
//! let file_size = file.size_styled()?;
//! println!("{}", file_size); // 17MiB
//!
//! let dotfile = file.move_to("dotfiles/")?;
//!
//! dotfile.symlink(".great_app.conf")?;
//! # Ok(())
//! # }
//! # fn main() {
//! # examples().unwrap();
//! # }
//! ```

mod file_operations;
mod file_types;
mod macros;
mod test;
mod unit_of_information;

pub use crate::{
    file_operations::Filey, file_types::FileTypes, macros::*, unit_of_information::UnitOfInfo,
};

/// Filey's error type.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    FileyError(anyhow::Error),
    #[error("'{}' No such file or directory", path)]
    NotFound {
        path: String,
    },
    #[error("'{}' is not a directory", path)]
    NotADirectory {
        path: String,
    },
    #[error("'{}' already exists", path)]
    AlreadyExists {
        path: String,
    },
}

pub type Result<T> = std::result::Result<T, crate::Error>;
