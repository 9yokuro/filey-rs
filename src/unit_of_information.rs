use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
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
    /// use fpop_rs::UnitOfInfo;
    ///
    /// let n: u64 = UnitOfInfo::GiB.into();
    /// // convert 1GiB to 1,024MiB.
    /// assert_eq!(UnitOfInfo::convert(n, UnitOfInfo::MiB) as u64, 1_024);
    pub fn convert(n: u64, u: Self) -> f64 {
        let m: u64 = u.into();
        (n / m) as f64
    }
}
