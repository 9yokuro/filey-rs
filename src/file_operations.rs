use std::{
    fmt,
    env::var,
    fs::{metadata, read_dir, remove_dir_all, remove_file, rename, File, create_dir_all},
    path::{Path, PathBuf},
};
use anyhow::{Result, bail};
use path_absolutize::Absolutize;
use crate::unit_of_information::UnitOfInfo;
use crate::file_types::FileTypes;

#[derive(Clone)]
pub struct FileOperations {
    path: PathBuf,
}

impl fmt::Display for FileOperations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
    }
}

impl FileOperations {
    /// Constructs a new FileOperations.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileOperations {
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
    pub fn file_type(&self) -> Result<FileTypes> {
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
    /// use fpop_rs::FileOperations;
    ///
    /// let size = FileOperations::new("install.sh").size().unwrap();
    /// println!("{}", size); // 1079
    /// ```
    pub fn size(&self) -> Result<u64> {
        if self.file_type()? == FileTypes::Directory {
            let number_of_files = self.list()?.len();
            Ok(number_of_files as u64)
        } else {
            let size = metadata(&self.path)?.len();
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
    /// use fpop_rs::FileOperations;
    ///
    /// let size_styled = FileOperations::new("great.rs").size_styled().unwrap();
    /// println!("{}", size_styled); // 20GiB
    /// ```
    pub fn size_styled(&self) -> Result<String> {
        if self.file_type()? == FileTypes::Directory {
            let number_of_files = self.list()?.len();
            Ok(number_of_files.to_string())
        } else {
            let n = self.size()?;
            let digit = n.to_string().chars().collect::<Vec<char>>().len();
            if 4 <= digit && digit < 7 {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::KiB));
                Ok(format!("{}{}", m, UnitOfInfo::KiB))
            } else if 7 <= digit && digit < 11 {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::MiB));
                Ok(format!("{}{}", m, UnitOfInfo::MiB))
            } else if 11 <= digit && digit < 15 {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::GiB));
                Ok(format!("{}{}", m, UnitOfInfo::GiB))
            } else if 15 <= digit && digit < 19 {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::TiB));
                Ok(format!("{}{}", m, UnitOfInfo::TiB))
            } else if 19 <= digit && digit < 23 {
                let m = round_size(UnitOfInfo::convert(n, UnitOfInfo::PiB));
                Ok(format!("{}{}", m, UnitOfInfo::PiB))
            } else if 23 <= digit && digit < 27 {
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
    /// let file = FileOperations::new("src/lib.rs");
    /// assert_eq!(file.file_name().unwrap().as_str(), "lib.rs");
    ///
    /// let directory = FileOperations::new("src/lib.rs");
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
    /// let file = FileOperations::new("src/lib.rs");
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
    /// let file = FileOperations::new("src/lib.rs");
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
    /// let file = FileOperations::new("src/lib.rs");
    /// assert_eq!(file.absolutized()
    ///     .unwrap()
    ///     .to_string_lossy()
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Tom/src/lib.rs");
    /// ```
    pub fn absolutized(&self) -> Result<PathBuf> {
        let path = self.expand_user()?.absolutize()?.to_path_buf();
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
    /// let file = FileOperations::new("nvim/init.lua");
    /// assert_eq!(file.canonicalized()
    ///     .unwrap()
    ///     .to_string_lossy()
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Lisa/dotfiles/nvim/init.lua");
    /// ```
    pub fn canonicalized(&self) -> Result<PathBuf> {
        let path = self.path.canonicalize()?;
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
    /// let directory = FileOperations::new("~/audio");
    /// assert_eq!(directory.expand_user()
    ///     .unwrap()
    ///     .to_string_lossy()
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Mike/audio");
    /// ```
    pub fn expand_user(&self) -> Result<PathBuf> {
        let home_dir = var("HOME")?;
        let s = &self.path.to_string_lossy().to_string();
        if s.starts_with("~") {
            let p = s.replacen("~", &home_dir, 1);
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
    /// let file = FileOperations::new("/home/Meg/cats.png");
    /// assert_eq!(file.close_user().unwrap().as_str(), "~/cats.png")
    /// ```
    pub fn close_user(&self) -> Result<String> {
        let home_dir = var("HOME")?;
        let s = self.path.to_string_lossy().to_string();
        if s.starts_with(&home_dir) {
            let p = s.replacen(&home_dir, "~", 1);
            Ok(p)
        } else {
            Ok(s)
        }
    }

    /// Creates a new file or directory.
    ///
    /// # Errors
    /// * The user lacks permissions.
    /// * path already exists.
    ///
    /// # Examples
    /// ```
    /// use fpop_rs::FileOperations;
    /// use fpop_rs::FileTypes;
    ///
    /// let file = FileOperations::new("src/errors.rs");
    /// file.create(FileTypes::File).unwrap();
    /// assert_eq!(file.exists(), true);
    /// ```
    pub fn create(&self, file_type: FileTypes) -> Result<Self> {
        match file_type {
            FileTypes::File => {File::create(&self.path)?;},
            FileTypes::Directory => create_dir_all(&self.path)?,
            _ => {}
        }
        Ok(self.clone())
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
    /// use fpop_rs::FileOperations;
    /// use fpop_rs::FileTypes;
    ///
    /// let file = FileOperations::new("cats.png").create(FileTypes::File).unwrap();
    /// file.move_to("photos/animals/").unwrap();
    /// assert_eq!(Path::new("photos/animals/cats.png").exists(), true);
    /// ```
    pub fn move_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        match FileTypes::which(&path).unwrap_or_else(|_| self.file_type().unwrap()) {
            FileTypes::Directory => {
                let p = path.as_ref().display().to_string();
                let to = format!("{}/{}", p, self.file_name().unwrap_or_else(|| self.path.to_string_lossy().to_string()));
                rename(&self.path, to)?;
            }
            _ => rename(&self.path, path)?,
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
    /// use fpop_rs::FileOperations;
    /// use fpop_rs::FileTypes;
    ///
    /// let file = FileOperations::new("coredump");
    /// file.remove().unwrap();
    /// assert_eq!(file.exists(), false);
    /// ```
    pub fn remove(&self) -> Result<()> {
        match self.file_type()? {
            FileTypes::Directory => remove_dir_all(&self.path)?,
            _ => remove_file(&self.path)?,
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
    /// use fpop_rs::FileOperations;
    ///
    /// let v = FileOperations::new("src/").list().unwarp();
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
    pub fn list(&self) -> Result<Vec<PathBuf>> {
        if self.file_type()? != FileTypes::Directory {
            bail!(
                "file-operations-rs::FileOperations::list: {} is not a directory",
                &self.path.to_string_lossy().to_string()
            )
        } else {
            let mut v = vec![];
            for i in read_dir(&self.path)? {
                let p = i?.path();
                v.push(p)
            }
            Ok(v)
        }
    }
}

fn round_size(n: f64) -> u64 {
    n.round() as u64
}
