use std::{
    fmt,
    env::var,
    fs::{metadata, read_dir, remove_dir_all, remove_file, rename},
    path::{Path, PathBuf},
};
use path_absolutize::Absolutize;
use crate::unit_of_information::UnitOfInfo;
use crate::file_types::FileTypes;
use crate::Error::{NotADirectory, FileyError};

#[derive(Clone)]
pub struct Filey {
    path: PathBuf,
}

impl fmt::Display for Filey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
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
    pub fn path(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    /// Returns type of the file.
    ///
    /// # Errors
    /// * The file doesn't exist.
    pub fn file_type(&self) -> Result<FileTypes, crate::Error> {
        let file_type = FileTypes::which(&self.path)?;
        Ok(file_type)
    }

    /// Returns size of the file.
    /// If path points to a directory, return the number of files in the directory.
    ///
    /// # Errors
    /// * The user lacks permissions.
    /// * The file doesn't exist.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::Filey;
    ///
    /// let size = Filey::new("install.sh").size().unwrap();
    /// println!("{}", size); // 1079
    /// ```
    pub fn size(&self) -> Result<u64, crate::Error> {
        if self.file_type()? == FileTypes::Directory {
            let number_of_files = self.list()?.len();
            Ok(number_of_files as u64)
        } else {
            let size = metadata(&self.path).map_err(|e| e.into()).map_err(FileyError)?.len();
            Ok(size)
        }
    }

    /// Return size of the file with a unit.
    /// If path points to a directory, return the number of files in the directory.
    ///
    /// # Errors
    /// * The user lacks permissions.
    /// * The file doesn't exist.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::Filey;
    ///
    /// let size_styled = Filey::new("great.rs").size_styled().unwrap();
    /// println!("{}", size_styled); // 20GiB
    /// ```
    pub fn size_styled(&self) -> Result<String, crate::Error> {
        if self.file_type()? == FileTypes::Directory {
            let number_of_files = self.list()?.len();
            Ok(number_of_files.to_string())
        } else {
            let n = self.size()?;
            let digit = n.to_string().chars().collect::<Vec<char>>().len();
            if (4..7).contains(&digit) {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::KiB));
                Ok(format!("{}{}", m, UnitOfInfo::KiB))
            } else if (7..11).contains(&digit) {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::MiB));
                Ok(format!("{}{}", m, UnitOfInfo::MiB))
            } else if (11..15).contains(&digit) {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::GiB));
                Ok(format!("{}{}", m, UnitOfInfo::GiB))
            } else if (15..19).contains(&digit) {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::TiB));
                Ok(format!("{}{}", m, UnitOfInfo::TiB))
            } else if (19..23).contains(&digit) {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::PiB));
                Ok(format!("{}{}", m, UnitOfInfo::PiB))
            } else if (23..27).contains(&digit) {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::EiB));
                Ok(format!("{}{}", m, UnitOfInfo::EiB))
            } else {
                Ok(format!("{}B", n))
            }
        }
    }

    /// Returns the file name or the directory name.
    /// Returns None if the path terminates in ...
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::*;
    ///
    /// let file = Filey::new("src/lib.rs");
    /// assert_eq!(file.file_name().unwrap().as_str(), "lib.rs");
    ///
    /// let directory = Filey::new("src/lib.rs");
    /// assert_eq!(directory.file_name().unwrap().as_str(), "src");
    /// ```
    pub fn file_name(&self) -> Option<String> {
        let name = self
            .path
            .file_name()?
            .to_string_lossy()
            .to_string();
        Some(name)
    }

    /// Returns the stem portion of the file name.
    /// Returns None if there is no file name.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::*;
    ///
    /// let file = Filey::new("src/lib.rs");
    /// assert_eq!(file.file_stem().unwrap().as_str(), "lib");
    /// ```
    pub fn file_stem(&self) -> Option<String> {
        let stem = self
            .path
            .file_stem()?
            .to_string_lossy()
            .to_string();
        Some(stem)
    }

    /// Returns the parent directory.
    /// Returns None if the path terminates in a root or prefix, or if it's the empty string.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::*;
    ///
    /// let file = Filey::new("src/lib.rs");
    /// assert_eq!(file.parent_dir()
    ///     .unwrap()
    ///     .to_string_lossy()
    ///     .to_string()
    ///     .as_str(),
    ///     "src");
    /// ```
    pub fn parent_dir(&self) -> Option<PathBuf> {
        let parent_dir = self
            .path
            .parent()?
            .to_path_buf();
        Some(parent_dir)
    }

    /// Returns the absolutized path of the file or the directory.
    ///
    /// # Errors
    /// * The environment variable isn't set.
    /// * The environment variable's name contains the equal sign character (=) or the NUL
    /// character.
    /// * The environment variable's value is not valid Unicode.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::*;
    ///
    /// let file = Filey::new("src/lib.rs");
    /// assert_eq!(file.absolutized()
    ///     .unwrap()
    ///     .to_string_lossy()
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Tom/src/lib.rs");
    /// ```
    pub fn absolutized(&self) -> Result<PathBuf, crate::Error> {
        let path = self.expand_user()?.absolutize().map_err(|e| e.into()).map_err(FileyError)?.to_path_buf();
        Ok(path)
    }

    /// Return the canonicalized(absolutized and symbolic links solved) path.
    ///
    /// # Errors
    /// * The path doesn't exist.
    /// * A non-final component in path is not a directory.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::*;
    ///
    /// // nvim/init.lua -> /home/Lisa/dotfiles/nvim/init.lua
    /// let file = Filey::new("nvim/init.lua");
    /// assert_eq!(file.canonicalized()
    ///     .unwrap()
    ///     .to_string_lossy()
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Lisa/dotfiles/nvim/init.lua");
    /// ```
    pub fn canonicalized(&self) -> Result<PathBuf, crate::Error> {
        let path = self.path.canonicalize().map_err(|e| e.into()).map_err(FileyError)?;
        Ok(path)
    }

    /// Replaces an initial tilde of the path by the environment variable HOME.
    ///
    /// # Errors
    /// * The environment variable isn't set.
    /// * The environment variable's name contains the equal sign character (=) or the NUL
    /// character.
    /// * The environment variable's value is not valid Unicode.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::*;
    ///
    /// let directory = Filey::new("~/audio");
    /// assert_eq!(directory.expand_user()
    ///     .unwrap()
    ///     .to_string_lossy()
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Mike/audio");
    /// ```
    pub fn expand_user(&self) -> Result<PathBuf, crate::Error> {
        let home_dir = var("HOME").map_err(|e| e.into()).map_err(FileyError)?;
        let s = &self.path.to_string_lossy().to_string();
        if s.starts_with('~') {
            let p = s.replacen('~', &home_dir, 1);
            Ok(Path::new(&p).to_path_buf())
        } else {
            Ok(self.path.to_path_buf())
        }
    }

    /// Replaces path_to_home by tilde.
    ///
    /// # Errors
    /// * The environment variable isn't set.
    /// * The environment variable's name contains the equal sign character (=) or the NUL
    /// character.
    /// * The environment variable's value is not valid Unicode.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::*;
    ///
    /// let file = Filey::new("/home/Meg/cats.png");
    /// assert_eq!(file.close_user().unwrap().as_str(), "~/cats.png")
    /// ```
    pub fn close_user(&self) -> Result<String, crate::Error> {
        let home_dir = var("HOME").map_err(|e| e.into()).map_err(FileyError)?;
        let s = self.path.to_string_lossy().to_string();
        if s.starts_with(&home_dir) {
            let p = s.replacen(&home_dir, "~", 1);
            Ok(p)
        } else {
            Ok(s)
        }
    }

    /// Move a file or a directory to the given path.
    ///
    /// # Errors
    /// * The user lacks permissions.
    /// * self.path and path: P are on separate filesystems.
    ///
    /// # Examples
    /// ```
    /// use std::path::Path;
    /// use fpop_rs::Filey;
    /// use fpop_rs::FileTypes;
    ///
    /// let file = Filey::new("cats.png").create(FileTypes::File).unwrap();
    /// file.move_to("photos/animals/").unwrap();
    /// assert_eq!(Path::new("photos/animals/cats.png").exists(), true);
    /// ```
    pub fn move_to<P: AsRef<Path>>(&self, path: P) -> Result<(), crate::Error> {
        match FileTypes::which(&path).unwrap_or_else(|_| self.file_type().unwrap()) {
            FileTypes::Directory => {
                let p = path.as_ref().display().to_string();
                let to = format!("{}/{}", p, self.file_name().unwrap_or_else(|| self.path.to_string_lossy().to_string()));
                rename(&self.path, to).map_err(|e| e.into()).map_err(FileyError)?;
            }
            _ => rename(&self.path, path).map_err(|e| e.into()).map_err(FileyError)?,
        }
        Ok(())
    }

    /// Judge the type of a file and remove the file.
    ///
    /// # Errors
    /// * The file doesn't exist.
    /// * The user lacks permissions.
    ///  
    /// # Examples
    /// ```
    /// use fpop_rs::Filey;
    /// use fpop_rs::FileTypes;
    ///
    /// let file = Filey::new("coredump");
    /// file.remove().unwrap();
    /// assert_eq!(file.exists(), false);
    /// ```
    pub fn remove(&self) -> Result<(), crate::Error> {
        match self.file_type()? {
            FileTypes::Directory => remove_dir_all(&self.path).map_err(|e| e.into()).map_err(FileyError)?,
            _ => remove_file(&self.path).map_err(|e| e.into()).map_err(FileyError)?,
        }
        Ok(())
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Returns a list of files in the directory.
    ///
    /// # Errors
    /// * path doesn't exists.
    /// * path is not a directory.
    /// * The user lacks permissions.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::Filey;
    ///
    /// let v = Filey::new("src/").list().unwarp();
    /// for i in v {
    ///     let s = i.to_string_lossy().to_string();
    ///     println!("{}", s)
    /// }
    ///
    /// // src/main.rs
    /// // src/ui.rs
    /// // src/draw.rs
    /// // src/errors.rs
    /// ```
    pub fn list(&self) -> Result<Vec<PathBuf>, crate::Error> {
        if self.file_type()? != FileTypes::Directory {
            Err(NotADirectory { path: self.path.to_string_lossy().to_string() })?
        } else {
            let mut v = vec![];
            for i in read_dir(&self.path).map_err(|e| e.into()).map_err(FileyError)? {
                let p = i.map_err(|e| e.into()).map_err(FileyError)?.path();
                v.push(p)
            }
            Ok(v)
        }
    }
}

fn round_size(n: f64) -> u64 {
    n.round() as u64
}
