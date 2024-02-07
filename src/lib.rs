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
#[cfg(target_family = "unix")]
mod permissions;
mod test;
pub mod units;

pub use crate::{file_types::FileTypes, filey::Filey, permissions::Permissions};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    FileyError(anyhow::Error),
    #[error("'{}' already exists", path)]
    AlreadyExists {
        path: String,
    },
    #[error("Could not get the name of '{}'", path)]
    GetFileNameError {
        path: String,
    },
}

pub type Result<T> = std::result::Result<T, crate::Error>;
