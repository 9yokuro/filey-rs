use crate::file_types::FileTypes;
use crate::unit_of_information::UnitOfInfo;
use crate::Error::{FileyError, NotADirectory, AlreadyExists};
use path_absolutize::Absolutize;
use std::{
    env::var,
    fmt,
    fs::{create_dir_all, metadata, read_dir, remove_dir_all, remove_file, rename, File, copy},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

/// The main struct.
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
    pub fn size(&self) -> Result<u64, crate::Error> {
        if self.file_type()? == FileTypes::Directory {
            let number_of_files = self.list()?.len();
            Ok(number_of_files as u64)
        } else {
            let size = metadata(&self.path)
                .map_err(|e| e.into())
                .map_err(FileyError)?
                .len();
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
    /// # use filey::Filey;
    /// # std::error::Error;
    /// #
    /// # fn get_size_styled() -> Result<(), Box<Error>> {
    /// let size_styled = Filey::new("great.rs").size_styled()?;
    /// println!("{}", size_styled); // 20GiB
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # get_size_styled().unwrap();
    /// # }
    /// ```
    pub fn size_styled(&self) -> Result<String, crate::Error> {
        if self.file_type()? == FileTypes::Directory {
            let number_of_files = self.list()?.len();
            Ok(number_of_files.to_string())
        } else {
            let n = self.size()?;
            Ok(UnitOfInfo::format(n))
        }
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
        let name = self.path.file_name()?.to_string_lossy().to_string();
        Some(name)
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
        let stem = self.path.file_stem()?.to_string_lossy().to_string();
        Some(stem)
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
    /// assert_eq!(file.parent_dir()?
    ///     .to_string_lossy()
    ///     .to_string()
    ///     .as_str(),
    ///     "src");
    /// # Some(file.path())
    /// # }
    /// # fn main() {
    /// # get_parent_dir().unwrap();
    /// # }
    /// ```
    pub fn parent_dir(&self) -> Option<PathBuf> {
        let parent_dir = self.path.parent()?.to_path_buf();
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
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn get_absoluzed() -> Result<(), Box<Error>> {
    /// let file = Filey::new("src/lib.rs");
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
    pub fn absolutized(&self) -> Result<Self, crate::Error> {
        let path = self
            .expand_user()?
            .path
            .absolutize()
            .map_err(|e| e.into())
            .map_err(FileyError)?
            .to_path_buf();
        let filey = Filey::new(path);
        Ok(filey)
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
    /// let file = Filey::new("nvim/init.lua");
    /// assert_eq!(file.canonicalized()?
    ///     .to_string()
    ///     .as_str(),
    ///     "/home/Lisa/dotfiles/nvim/init.lua");
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # get_canonicalized().unwrap();
    /// # }
    /// ```
    pub fn canonicalized(&self) -> Result<Self, crate::Error> {
        let path = self
            .path
            .canonicalize()
            .map_err(|e| e.into())
            .map_err(FileyError)?;
        let filey = Filey::new(path);
        Ok(filey)
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
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn get_expanded() -> Result<(), Box<Error>> {
    /// let directory = Filey::new("~/audio");
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
    pub fn expand_user(&self) -> Result<Self, crate::Error> {
        let home_dir = var("HOME").map_err(|e| e.into()).map_err(FileyError)?;
        let s = &self.path.to_string_lossy().to_string();
        if s.starts_with('~') {
            let p = s.replacen('~', &home_dir, 1);
            let filey = Filey::new(p);
            Ok(filey)
        } else {
            Ok(self.clone())
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
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// #
    /// # fn get_closed() -> Result<(), Box<Error>> {
    /// let file = Filey::new("/home/Meg/cats.png");
    /// assert_eq!(file.close_user()?.as_str(), "~/cats.png")
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # get_closed().unwrap();
    /// # }
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
    /// let file = Filey::new("cats.png");
    /// file.move_to("photos/animals/")?;
    /// assert_eq!(Path::new("photos/animals/cats.png").exists(), true);
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # moves().unwrap();
    /// # }
    /// ```
    pub fn move_to<P: AsRef<Path>>(&self, path: P) -> Result<Self, crate::Error> {
        if path.as_ref().exists() {
            if let FileTypes::Directory = FileTypes::which(&path)? {
                let p = path.as_ref().display().to_string();
                let to = format!(
                    "{}/{}",
                    p,
                    self.file_name().unwrap_or_else(|| self.to_string())
                );
                rename(&self.path, &to)
                    .map_err(|e| e.into())
                    .map_err(FileyError)?;
                let filey = Filey::new(&to);
                Ok(filey)
            } else {
                Err(AlreadyExists { path: path.as_ref().display().to_string() })
            }
        } else {
                rename(&self.path, &path)
                    .map_err(|e| e.into())
                    .map_err(FileyError)?;
                let filey = Filey::new(&path);
                Ok(filey)
        }
    }

    /// Judge the type of a file and remove the file.
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
    pub fn remove(&self) -> Result<(), crate::Error> {
        match self.file_type()? {
            FileTypes::Directory => remove_dir_all(&self.path)
                .map_err(|e| e.into())
                .map_err(FileyError)?,
            _ => remove_file(&self.path)
                .map_err(|e| e.into())
                .map_err(FileyError)?,
        }
        Ok(())
    }

    /// Create a new file or directory.
    ///
    /// # Examples
    /// ```
    /// # use filey::{Filey, FileTypes};
    /// # use std::error::Error;
    /// #
    /// # fn touch() -> Result<(), Box<Error> {
    /// let directory = File::new("photo/dogs").create(FileTypes::Directory)?;
    /// assert_eq!(directory.exists(), true);
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # touch().unwrap();
    /// # }
    /// ```
    pub fn create(&self, file_type: FileTypes) -> Result<Self, crate::Error> {
        match file_type {
            FileTypes::File => {
                File::create(&self.path)
                    .map_err(|e| e.into())
                    .map_err(FileyError)?;
            }
            FileTypes::Directory => create_dir_all(&self.path)
                .map_err(|e| e.into())
                .map_err(FileyError)?,
            FileTypes::Symlink => (),
        }
        Ok(self.clone())
    }

    /// Copy the contents of file to another.
    pub fn copy<P: AsRef<Path>>(&self, path: P) -> Result<Self, crate::Error> {
        copy(&self.path, &path).map_err(|e| e.into()).map_err(FileyError)?;
        let filey = Filey::new(path);
        Ok(filey)
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
    /// let vimrc_dotfiles = Filey::new("~/dotfiles/vimrc");
    /// vimrc_dotfiles.create(FileTypes::File).symlink("~/.vimrc")?;
    /// assert_eq!(Path::new("~/.vimrc").exists(), true);
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # create_symlink().unwrap();
    /// # }
    /// ```
    #[cfg(target_family = "unix")]
    pub fn symlink<P: AsRef<Path>>(&self, path: P) -> Result<(), crate::Error> {
        let original = &self.absolutized()?.path;
        let link = Filey::new(path).absolutized()?.path;
        symlink(original, link)
            .map_err(|e| e.into())
            .map_err(FileyError)?;
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
    /// # use filey::Filey;
    /// # use std::error::Error;
    /// # fn ls() -> Result<(), Box<Error>> {
    /// let v = Filey::new("src/").list()?;
    /// for i in v {
    ///     let s = i.to_string_lossy().to_string();
    ///     println!("{}", s)
    /// }
    ///
    /// // src/main.rs
    /// // src/ui.rs
    /// // src/draw.rs
    /// // src/errors.rs
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # ls().unwrap();
    /// # }
    /// ```
    pub fn list(&self) -> Result<Vec<PathBuf>, crate::Error> {
        if self.file_type()? != FileTypes::Directory {
            Err(NotADirectory {
                path: self.path.to_string_lossy().to_string(),
            })?
        } else {
            let mut v = vec![];
            for i in read_dir(&self.path)
                .map_err(|e| e.into())
                .map_err(FileyError)?
            {
                let p = i.map_err(|e| e.into()).map_err(FileyError)?.path();
                v.push(p)
            }
            Ok(v)
        }
    }
}

/// Concatenates file(s) to String.
///
/// # Examples
/// ```
/// # use filey::{Filey, catenate};
/// # use std::error::Error;
/// # use std::fs;
/// #
/// # fn cat() -> Result<(), Box<Error>> {
/// fs::write("h.rs", "fn main {")?;
/// fs::write("el.rs", r#"    println!("Hello, World!");"#)?;
/// fs::write("lo.rs", "}")?;
///
/// let s = catenate!("h.rs", "el.rs", "lo.rs");
/// println!("{}", s);
/// // fn main() {
/// //     println!("Hello, World!");
/// // }
/// # Ok(())
/// # }
/// # fn main() {
/// # cat().unwrap();
/// # }
/// ```
#[macro_export]
macro_rules! catenate {
    ( $( $path:expr ),* ) => {
        {
            use std::{io::Read, path::Path, fs::File};

            let mut buffer = String::new();
            $(
                if Path::new($path).is_file() {
                let mut s = String::new();
                let mut f = File::open($path).unwrap();
                f.read_to_string(&mut s).unwrap();
                buffer.push_str(&s);
                buffer.push('\n');
                }
            )*
            buffer
        }
    }
}

/// Creates file(s) or directory(s).
///
/// # Examples
/// ```
/// # use filey::{Filey, FileTypes, create};
/// #
/// create!(FileTypes::File, "src/draw_ui.rs", "src/app_state.rs", "run.rs");
/// ```
#[macro_export]
macro_rules! create {
    ( $file_type:expr $(, $path:expr )* $(,)?) => {
        {
            $(
                let f = Filey::new($path);
                if !f.exists() {
                    f.create($file_type).unwrap();
                }
            )*
        }
    }
}

/// Removes file(s) or directory(s).
///
/// # Examples
/// ```
/// # use filey::{Filey, remove};
/// remove!("old_dir", "unnecessary.jpg");
/// ```
#[macro_export]
macro_rules! remove {
    ( $( $path:expr ), *) => {
        {
            $(
                let f = Filey::new($path);
                if f.exists() {
                    f.remove().unwrap();
                }
            )*
        }
    }
}
