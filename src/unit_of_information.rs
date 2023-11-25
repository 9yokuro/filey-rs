use serde::{Deserialize, Serialize};
use std::fmt;

/// Units derived from bit.
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize, Eq)]
pub enum UnitOfInfo {
    KiB,
    MiB,
    GiB,
    TiB,
    PiB,
    EiB,
}

impl fmt::Display for UnitOfInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::KiB => write!(f, "KiB"),
            Self::MiB => write!(f, "MiB"),
            Self::GiB => write!(f, "GiB"),
            Self::TiB => write!(f, "TiB"),
            Self::PiB => write!(f, "PiB"),
            Self::EiB => write!(f, "EiB"),
        }
    }
}

impl From<UnitOfInfo> for u64 {
    fn from(u: UnitOfInfo) -> u64 {
        match u {
            UnitOfInfo::KiB => 1_024,
            UnitOfInfo::MiB => 1_048_576,
            UnitOfInfo::GiB => 1_073_741_824,
            UnitOfInfo::TiB => 1_099_511_627_776,
            UnitOfInfo::PiB => 1_125_899_906_842_624,
            UnitOfInfo::EiB => 1_152_921_504_606_846_976,
        }
    }
}

impl UnitOfInfo {
    /// # Examples
    /// ```
    /// use filey::UnitOfInfo;
    ///
    /// let n: u64 = UnitOfInfo::GiB.into();
    /// // convert 1GiB to 1,024MiB.
    /// assert_eq!(UnitOfInfo::convert(n, UnitOfInfo::MiB) as u64, 1_024);
    pub fn convert(n: u64, u: Self) -> f64 {
        let m: u64 = u.into();
        (n / m) as f64
    }

    pub fn format(n: u64) -> String {
        let m = digit(n);
        if (4..7).contains(&m) {
            format!("{}{}", round(Self::convert(n, Self::KiB)), Self::KiB)
        } else if (7..10).contains(&m) {
            format!("{}{}", round(Self::convert(n, Self::MiB)), Self::MiB)
        } else if (10..13).contains(&m) {
            format!("{}{}", round(Self::convert(n, Self::GiB)), Self::GiB)
        } else if (13..16).contains(&m) {
            format!("{}{}", round(Self::convert(n, Self::TiB)), Self::TiB)
        } else if (16..19).contains(&m) {
            format!("{}{}", round(Self::convert(n, Self::PiB)), Self::PiB)
        } else if (19..22).contains(&m) {
            format!("{}{}", round(Self::convert(n, Self::EiB)), Self::EiB)
        } else {
            format!("{}B", n)
        }
    }
}

fn round(n: f64) -> u64 {
    n.round() as u64
}

fn digit(n: u64) -> u64 {
    n.to_string().chars().collect::<Vec<char>>().len() as u64
}
