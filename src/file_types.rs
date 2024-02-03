use serde::{Deserialize, Serialize};
use std::{fmt, path::Path};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Serialize, Deserialize, Hash)]
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
    /// Detects the type of a file.
    /// If the file doesn't exist, returns None.
    ///
    /// # Examples
    /// ```
    /// # use filey::FileTypes;
    /// #
    /// # fn kind() -> Option<()> {
    /// let file = "src/lib.rs";
    /// println!(FileTypes::which(file)?); // file
    ///
    /// let directory = "src";
    /// println!(FileTypes::which(directory)?); // directory
    /// # Some(())
    /// # }
    /// # fn main() {
    /// # kind().unwrap();
    /// # }
    /// ```
    pub fn which<P: AsRef<Path>>(path: P) -> Option<Self> {
        let p: &Path = path.as_ref();

        if p.is_symlink() {
            Some(Self::Symlink)
        } else if p.exists() {
            if p.is_dir() {
                Some(Self::Directory)
            } else {
                Some(Self::File)
            }
        } else {
            None
        }
    }
}
