mod test;

use anyhow::{Context, Result};
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use std::env::var;
use std::fmt;
use std::fs::{metadata, remove_dir_all, remove_file, rename, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

pub trait FileIO {
    type Item;
    fn read<P: AsRef<Path>>(&self, path: P) -> Result<Self::Item>;
    fn write<P: AsRef<Path>, S: AsRef<str>>(&self, path: P, s: S) -> Result<()>;
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum FileTypes {
    File,
    Directory,
    Symlink,
    None,
}

impl fmt::Display for FileTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Directory => write!(f, "directory"),
            Self::Symlink => write!(f, "symlink"),
            Self::None => write!(f, ""),
        }
    }
}

impl FileTypes {
    pub fn which<P: AsRef<Path>>(path: P) -> Self {
        let p: &Path = path.as_ref();
        if p.exists() {
            if p.is_dir() {
                Self::Directory
            } else if p.is_symlink() {
                Self::Symlink
            } else {
                Self::File
            }
        } else {
            Self::None
        }
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct FileInfo {
    path: String,
    filetype: FileTypes,
    size: u64,
}

impl fmt::Display for FileInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!(
            "path: {}\ntype: {}\n size:{}",
            &self.path, &self.filetype, &self.size
        );
        write!(f, "{}", s)
    }
}

impl FileIO for FileInfo {
    type Item = Self;

    fn read<P: AsRef<Path>>(&self, path: P) -> Result<Self::Item> {
        let f = File::open(&path)?;
        let reader = BufReader::new(f);
        let fileinfo = serde_json::from_reader(reader)?;
        Ok(fileinfo)
    }

    fn write<P: AsRef<Path>, S: AsRef<str>>(&self, path: P, _s: S) -> Result<()> {
        let f = OpenOptions::new().write(true).truncate(true).open(&path)?;
        let writer = BufWriter::new(f);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}

impl FileInfo {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let filetype = FileTypes::which(&path);
        let size = metadata(&path)?.len();
        let path = path.as_ref().display().to_string();
        let fileinfo = Self {
            path,
            filetype,
            size,
        };
        Ok(fileinfo)
    }
}

pub struct FileOperations {
    path: PathBuf,
}

impl fmt::Display for FileOperations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path.to_string_lossy())
    }
}

impl FileOperations {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileOperations {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn pathbuf(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    pub fn file_name(&self) -> Result<String> {
        let name = self
            .path
            .file_name()
            .context("file_operations_rs::FileOperations::file_name: Failed to get file name")?
            .to_string_lossy()
            .to_string();
        Ok(name)
    }

    pub fn file_stem(&self) -> Result<String> {
        let stem = self
            .path
            .file_stem()
            .context("file_operations_rs::FileOperations::file_stem: Failed to get file stem")?
            .to_string_lossy()
            .to_string();
        Ok(stem)
    }

    pub fn parent_dir(&self) -> Result<PathBuf> {
        let parent_dir = self
            .path
            .parent()
            .context(
                "file_operations_rs::FileOperations::parent_dir: Failed to get parent directory",
            )?
            .to_path_buf();
        Ok(parent_dir)
    }

    pub fn absolutized(&self) -> Result<PathBuf> {
        let path = self.path.absolutize()?.to_path_buf();
        Ok(path)
    }

    pub fn canonicalized(&self) -> Result<PathBuf> {
        let path = self.path.canonicalize()?;
        Ok(path)
    }

    pub fn expand_user(&self) -> Result<PathBuf> {
        let home_dir = var("HOME")?;
        let s = &self.path.to_string_lossy().to_string();
        if s.starts_with("~") {
            let p = s.replacen("~", &home_dir, 1);
            Ok(Path::new(&p).to_path_buf())
        } else {
            Ok(self.pathbuf())
        }
    }

    pub fn close_user(&self) -> Result<String> {
        let home_dir = var("HOME")?;
        let s = self.pathbuf().to_string_lossy().to_string();
        if s.starts_with(&home_dir) {
            let p = s.replacen(&home_dir, "~", 1);
            Ok(p)
        } else {
            Ok(s)
        }
    }

    pub fn move_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        match FileTypes::which(&path) {
            FileTypes::Directory => {
                let p = path.as_ref().display().to_string();
                let to = format!("{}/{}", p, self.file_name()?);
                rename(&self.path, to)?;
            }
            _ => rename(&self.path, path)?,
        }
        Ok(())
    }

    pub fn remove(&self) -> Result<()> {
        match FileTypes::which(&self.path) {
            FileTypes::Directory => remove_dir_all(&self.path)?,
            _ => remove_file(&self.path)?,
        }
        Ok(())
    }

    pub fn exists(&self) -> bool {
        self.pathbuf().exists()
    }
}

impl FileIO for FileOperations {
    type Item = String;

    fn read<P: AsRef<Path>>(&self, path: P) -> Result<Self::Item> {
        let f = File::open(&path)?;
        let mut reader = BufReader::new(f);
        let mut s = String::new();
        reader.read_to_string(&mut s)?;
        Ok(s)
    }

    fn write<P: AsRef<Path>, S: AsRef<str>>(&self, path: P, s: S) -> Result<()> {
        let f = OpenOptions::new().write(true).truncate(true).open(&path)?;
        let mut writer = BufWriter::new(f);
        writer.write_all(s.as_ref().as_bytes())?;
        Ok(())
    }
}
