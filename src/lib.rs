//! # filey
//! 'filey' is a collection of utilities to make file operations more convenient.
//!
//! This library is made up of three main components:
//!
//! - [`Filey`]: the main struct.
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
//! let mut file = Filey::new(".great_app.conf").create(FileTypes::File)?;
//! let file_size = file.size()?;
//! println!("{}", file_size); // 0
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

mod file_types;
mod filey;
mod macros;
mod test;

pub use crate::{file_types::FileTypes, filey::Filey, macros::*};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    FileyError(anyhow::Error),
    #[error("'{}' No such file or directory", path)]
    NotFoundError {
        path: String,
    },
    #[error("'{}' is not a directory", path)]
    NotADirectoryError {
        path: String,
    },
    #[error("'{}' already exists", path)]
    AlreadyExistsError {
        path: String,
    },
    #[error("Could not get the name of '{}'", path)]
    GetFileNameError {
        path: String,
    },
}

pub type Result<T> = std::result::Result<T, crate::Error>;
