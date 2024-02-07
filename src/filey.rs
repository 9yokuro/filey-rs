use crate::{
    file_types::FileTypes,
    Error::{FileyError, GetFileNameError},
    Permissions, Result,
};
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use std::{
    convert::AsRef,
    env::var,
    fmt,
    fs::{copy, create_dir_all, hard_link, metadata, remove_dir_all, remove_file, rename, File},
    io::{Read, Write},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Filey {
    path: PathBuf,
}

impl fmt::Display for Filey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
    }
}

impl AsRef<Path> for Filey {
    fn as_ref(&self) -> &Path {
        let path: &Path = self.path.as_ref();
        path
    }
}

impl Read for Filey {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut f = File::open(self)?;
        f.read(buf)
    }
}

impl Write for Filey {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut f = File::create(self)?;
        f.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut f = File::create(self)?;
        f.flush()
    }
}

impl Filey {
    /// Constructs a new Filey.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Filey {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Returns path to the file.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the type of the file.
    /// If the path doesn't exist, return None.
    pub fn file_type(&self) -> Option<FileTypes> {
        FileTypes::which(&self.path)
    }

    /// Returns size of the file.
    ///
    /// # Errors
    /// * The user lacks permissions.
    /// * The file doesn't exist.
    ///
    /// # Examples
    /// ```
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn get_size() -> Result<(), Box<Error>> {
    /// let size = Filey::new("install.sh").size()?;
    /// println!("{}", size); // 1079
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # get_size().unwrap();
    /// # }
    /// ```
    pub fn size(&self) -> Result<u64> {
        let metadata = metadata(&self.path)
            .map_err(|e| e.into())
            .map_err(FileyError)?;
        let size = metadata.len();
        Ok(size)
    }

    pub fn permissions(&self) -> Result<Permissions> {
        Permissions::from_path(self)
    }

    /// Returns the file name or the directory name.
    /// Returns None if the path terminates in ...
    ///
    /// # Examples
    /// ```
    /// # use filey::Filey;
    /// #
    /// # fn get_file_name() -> Option<String> {
    /// let file = Filey::new("src/lib.rs");
    /// assert_eq!(file.file_name()?.as_str(), "lib.rs");
    ///
    /// let directory = Filey::new("src/lib.rs");
    /// assert_eq!(directory.file_name()?.as_str(), "src");
    /// # Some(directory.to_string())
    /// # }
    /// # fn main() {
    /// # get_file_name().unwrap();
    /// # }
    /// ```
    pub fn file_name(&self) -> Option<String> {
        Some(self.path.file_name()?.to_string_lossy().to_string())
    }

    /// Returns the stem portion of the file name.
    /// Returns None if there is no file name.
    ///
    /// # Examples
    /// ```
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn get_file_stem() -> Option<String> {
    /// let file = Filey::new("src/lib.rs");
    /// assert_eq!(file.file_stem()?.as_str(), "lib");
    /// # Some(file.to_string())
    /// # }
    /// # fn main() {
    /// # get_file_stem().unwrap();
    /// # }
    /// ```
    pub fn file_stem(&self) -> Option<String> {
        Some(self.path.file_stem()?.to_string_lossy().to_string())
    }

    /// Returns the parent directory.
    /// Returns None if the path terminates in a root or prefix, or if it's the empty string.
    ///
    /// # Examples
    /// ```
    /// # use filey::Filey;
    /// #
    /// # fn get_parent_dir() -> Option<PathBuf> {
    /// let file = Filey::new("src/lib.rs");
    /// assert_eq!(file.parent_dir()?.as_str(), "src");
    /// # Some(file.path())
    /// # }
    /// # fn main() {
    /// # get_parent_dir().unwrap();
    /// # }
    /// ```
    pub fn parent_dir(&self) -> Option<String> {
        Some(self.path.parent()?.to_string_lossy().to_string())
    }

    /// Returns the absolutized path of the file or the directory.
    ///
    /// # Errors
    /// * The environment variable HOME isn't set.
    /// * The environment variable's name contains the equal sign character (=) or the NUL
    /// character.
    /// * The environment variable's value is not valid Unicode.
    ///
    /// # Examples
    /// ```
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn get_absoluzed() -> Result<(), Box<Error>> {
    /// let mut file = Filey::new("src/lib.rs");
    /// assert_eq!(file.absolutized()?
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Tom/src/lib.rs");
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # get_absoluzed().unwrap();
    /// # }
    /// ```
    pub fn absolutize(&mut self) -> Result<&mut Self> {
        let absolutized = self
            .path
            .absolutize()
            .map_err(|e| e.into())
            .map_err(FileyError)?;
        self.path = absolutized.to_path_buf();
        Ok(self)
    }

    /// Return the canonicalized(absolutized and symbolic links solved) path.
    ///
    /// # Errors
    /// * The path doesn't exist.
    /// * A non-final component in path is not a directory.
    ///
    /// # Examples
    /// ```
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn get_canonicalized() -> Result<(), Box<Error>> {
    /// // nvim/init.lua -> /home/Lisa/dotfiles/nvim/init.lua
    /// let mut file = Filey::new("nvim/init.lua");
    /// assert_eq!(file.canonicalize()?
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Lisa/dotfiles/nvim/init.lua");
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # get_canonicalized().unwrap();
    /// # }
    /// ```
    pub fn canonicalize(&mut self) -> Result<&mut Self> {
        let canonicalized = self
            .path
            .canonicalize()
            .map_err(|e| e.into())
            .map_err(FileyError)?;
        self.path = canonicalized;
        Ok(self)
    }

    /// Replaces an initial tilde of the path by the environment variable HOME.
    ///
    /// # Errors
    /// * The environment variable HOME isn't set.
    /// * The environment variable's name contains the equal sign character (=) or the NUL
    /// character.
    /// * The environment variable's value is not valid Unicode.
    ///
    /// # Examples
    /// ```
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn get_expanded() -> Result<(), Box<Error>> {
    /// let mut directory = Filey::new("~/audio");
    /// assert_eq!(directory.expand_user()?
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Mike/audio");
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # get_expanded().unwrap();
    /// # }
    /// ```
    pub fn expand_user(&mut self) -> Result<&mut Self> {
        let s = &self.to_string();
        if s.starts_with('~') {
            let expanded = s.replacen('~', &home_dir()?, 1);
            self.path = Path::new(&expanded).to_path_buf();
        }
        Ok(self)
    }

    /// Replaces path_to_home by tilde.
    ///
    /// # Errors
    /// * The environment variable HOME isn't set.
    /// * The environment variable's name contains the equal sign character (=) or the NUL
    /// character.
    /// * The environment variable's value is not valid Unicode.
    ///
    /// # Examples
    /// ```
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn get_closed() -> Result<(), Box<Error>> {
    /// let mut file = Filey::new("/home/Meg/cats.png");
    /// assert_eq!(file.close_user()?.as_str(), "~/cats.png")
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # get_closed().unwrap();
    /// # }
    /// ```
    pub fn contract_user(&mut self) -> Result<&mut Self> {
        let home_dir = &home_dir()?;
        let s = self.to_string();
        if s.starts_with(home_dir) {
            let contracted = s.replacen(home_dir, "~", 1);
            self.path = Path::new(&contracted).to_path_buf();
        }
        Ok(self)
    }

    /// Move a file or a directory to the given path.
    ///
    /// # Errors
    /// * The user lacks permissions.
    /// * from(Filey) and to(path: P) are on separate filesystems.
    ///
    /// # Panics
    /// * Both from and to don't exist.
    ///
    /// # Examples
    /// ```
    /// # use std::path::Path;
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn moves() -> Result<(), Box<Error>> {
    /// let mut file = Filey::new("cats.png");
    /// file.move_to("photos/animals/")?;
    /// assert_eq!(Path::new("photos/animals/cats.png").exists(), true);
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # moves().unwrap();
    /// # }
    /// ```
    pub fn move_to<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Self> {
        let path = path.as_ref();

        if path.is_dir() {
            let file_name = self.file_name().ok_or_else(|| GetFileNameError {
                path: self.to_string(),
            })?;
            let to = path.to_path_buf().join(file_name);

            rename(&self, &to)
                .map_err(|e| e.into())
                .map_err(FileyError)?;
            self.path = to;
            Ok(self)
        } else {
            rename(&self, path)
                .map_err(|e| e.into())
                .map_err(FileyError)?;
            self.path = path.to_path_buf();
            Ok(self)
        }
    }

    /// Detects the type of a file and remove the file.
    ///
    /// # Errors
    /// * The file doesn't exist.
    /// * The user lacks permissions.
    ///  
    /// # Examples
    /// ```
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn rm() -> Result<(), Box<Error>> {
    /// let file = Filey::new("coredump");
    /// file.remove()?;
    /// assert_eq!(file.exists(), false);
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # rm().unwrap();
    /// # }
    /// ```
    pub fn remove(&self) -> Result<()> {
        if self.path.is_dir() {
            remove_dir_all(&self)
                .map_err(|e| e.into())
                .map_err(FileyError)?
        } else {
            remove_file(&self)
                .map_err(|e| e.into())
                .map_err(FileyError)?;
        }
        Ok(())
    }

    pub fn create_file(&self) -> Result<Self> {
        File::create(self)
            .map_err(|e| e.into())
            .map_err(FileyError)?;
        Ok(self.clone())
    }

    pub fn create_dir(&self) -> Result<Self> {
        create_dir_all(self)
            .map_err(|e| e.into())
            .map_err(FileyError)?;
        Ok(self.clone())
    }

    /// Copy the contents of file to another.
    pub fn copy<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        if path.is_dir() {
            let file_name = self.file_name().ok_or_else(|| GetFileNameError {
                path: self.to_string(),
            })?;
            let to = path.to_path_buf().join(file_name);

            copy(self, to).map_err(|e| e.into()).map_err(FileyError)?;
            Ok(())
        } else {
            copy(self, path).map_err(|e| e.into()).map_err(FileyError)?;
            Ok(())
        }
    }

    /// (Unix only) Create a new symbolic link on the filesystem.
    ///
    /// # Examples
    /// ```
    /// # use filey::{Filey, FileTypes};
    /// # use std::path::Path;
    /// # use std::error::Error;
    /// #
    /// # fn create_symlink() -> Result<(), Box<Error> {
    /// let mut vimrc_dotfiles = Filey::new("~/dotfiles/vimrc");
    /// vimrc_dotfiles.create(FileTypes::File).symlink("~/.vimrc")?;
    /// assert!(Path::new("~/.vimrc").exists());
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # create_symlink().unwrap();
    /// # }
    /// ```
    #[cfg(target_family = "unix")]
    pub fn symlink<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        if path.is_dir() {
            let file_name = self.file_name().ok_or_else(|| GetFileNameError {
                path: self.to_string(),
            })?;
            let link = path.to_path_buf().join(file_name);
            symlink(self, link)
                .map_err(|e| e.into())
                .map_err(FileyError)?;
            Ok(())
        } else {
            symlink(self, path)
                .map_err(|e| e.into())
                .map_err(FileyError)?;
            Ok(())
        }
    }

    /// Create a new hard link on the filesystem.
    ///
    /// # Errors
    /// The original path is not a file or doesn't exist.
    ///
    /// # Examples
    /// ```
    /// # use filey::{Filey, FileTypes};
    /// # use std::path::Path;
    /// # use std::error::Error;
    /// #
    /// # fn create_hard_link() -> Result<(), Box<Error> {
    /// let mut file = Filey::new("foo.txt");
    /// file.create(FileTypes::File).hard_link("bar.txt")?;
    /// assert_eq!(Path::new("bar.txt").exists(), true);
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # create_hard_link().unwrap();
    /// # }
    /// ```
    pub fn hard_link<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        if path.is_dir() {
            let file_name = self.file_name().ok_or_else(|| GetFileNameError {
                path: self.to_string(),
            })?;
            let link = path.to_path_buf().join(file_name);
            hard_link(self, link)
                .map_err(|e| e.into())
                .map_err(FileyError)?;
            Ok(())
        } else {
            hard_link(self, path)
                .map_err(|e| e.into())
                .map_err(FileyError)?;
            Ok(())
        }
    }

    pub fn exists(&self) -> bool {
        self.path.is_symlink() || self.path.exists()
    }

    pub fn is_file(&self) -> bool {
        self.path.is_file()
    }

    pub fn is_dir(&self) -> bool {
        self.path.is_dir()
    }

    pub fn is_symlink(&self) -> bool {
        self.path.is_symlink()
    }
}

fn home_dir() -> Result<String> {
    var("HOME").map_err(|e| e.into()).map_err(FileyError)
}
