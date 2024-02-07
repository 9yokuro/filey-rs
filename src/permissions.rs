use crate::{Error::FileyError, Result};
use serde::{Deserialize, Serialize};
use std::{fs::metadata, os::unix::fs::PermissionsExt, path::Path};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permission {
    execute: bool,
    write: bool,
    read: bool,
}

impl Permission {
    pub fn new(execute: bool, write: bool, read: bool) -> Self {
        Self {
            execute,
            write,
            read,
        }
    }

    fn from_mode(mode: u32) -> Self {
        if mode == 1 {
            Self::new(true, false, false)
        } else if mode == 2 {
            Self::new(false, true, false)
        } else if mode == 3 {
            Self::new(true, true, false)
        } else if mode == 4 {
            Self::new(false, false, true)
        } else if mode == 5 {
            Self::new(true, false, true)
        } else if mode == 6 {
            Self::new(false, true, true)
        } else if mode == 7 {
            Self::new(true, true, true)
        } else {
            Self::new(false, false, false)
        }
    }

    pub fn has_execute(&self) -> bool {
        self.execute
    }

    pub fn has_write(&self) -> bool {
        self.write
    }

    pub fn has_read(&self) -> bool {
        self.read
    }

    pub fn set_execute(&mut self, value: bool) -> &mut Self {
        self.execute = value;
        self
    }

    pub fn set_write(&mut self, value: bool) -> &mut Self {
        self.write = value;
        self
    }

    pub fn set_read(&mut self, value: bool) -> &mut Self {
        self.read = value;
        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permissions {
    user: Permission,
    group: Permission,
    others: Permission,
}

impl Permissions {
    pub fn new(user: Permission, group: Permission, others: Permission) -> Self {
        Self {
            user,
            group,
            others,
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let metadata = metadata(path).map_err(|e| e.into()).map_err(FileyError)?;
        let permissions = format!("{:o}", metadata.permissions().mode());
        let permissions = permissions.chars().skip(2).collect::<Vec<char>>();
        let length = permissions.len();

        let user = Permission::from_mode(char_to_u32(permissions[length - 3])?);
        let group = Permission::from_mode(char_to_u32(permissions[length - 2])?);
        let others = Permission::from_mode(char_to_u32(permissions[length - 1])?);

        Ok(Self::new(user, group, others))
    }

    pub fn user(&self) -> &Permission {
        &self.user
    }

    pub fn group(&self) -> &Permission {
        &self.group
    }

    pub fn others(&self) -> &Permission {
        &self.others
    }
}

fn char_to_u32(c: char) -> Result<u32> {
    c.to_string()
        .parse::<u32>()
        .map_err(|e| e.into())
        .map_err(FileyError)
}
