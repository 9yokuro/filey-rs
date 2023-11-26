use crate::Error::NotFound;
use serde::{Deserialize, Serialize};
use std::{fmt, path::Path};

/// Kinds of files.
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum FileTypes {
    File,
    Directory,
    Symlink,
}

impl fmt::Display for FileTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Directory => write!(f, "directory"),
            Self::Symlink => write!(f, "symlink"),
        }
    }
}

impl FileTypes {
    /// Judge the type of a file.
    ///
    /// # Errors
    /// * The file doesn't exist.
    ///
    /// # Examples
    /// ```
    /// # use filey::FileTypes;
    /// # use std::error::Error;
    /// #
    /// # fn kind() -> Result<(), Box<Error>> {
    /// let file = "src/lib.rs";
    /// println!(FileTypes::which(file)?); // file
    ///
    /// let directory = "src";
    /// println!(FileTypes::which(directory)?); // directory
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # kind().unwrap();
    /// # }
    /// ```
    pub fn which<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let p: &Path = path.as_ref();
        if p.exists() {
            if p.is_dir() {
                Ok(Self::Directory)
            } else if p.is_symlink() {
                Ok(Self::Symlink)
            } else {
                Ok(Self::File)
            }
        } else {
            Err(NotFound {
                path: path.as_ref().display().to_string(),
            })?
        }
    }
}
