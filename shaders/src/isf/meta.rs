use std::{
    ffi::{OsStr, OsString},
    fmt::{Display, Formatter},
    fs::read_to_string,
    path::{Path, PathBuf},
};

use isf::Isf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_os = "windows")]
pub fn default_isf_path() -> PathBuf {
    Path::new("C:\\ProgramData\\ISF")
}

#[cfg(target_os = "macos")]
pub fn default_isf_path() -> PathBuf {
    Path::new("/Library/Graphics/ISF").to_path_buf()
}

#[derive(Error, Debug)]
pub enum IsfInfoReadError {
    #[error("file read failed")]
    IoError(#[from] std::io::Error),
    #[error("invalid file extension: .{0:?}")]
    InvalidExt(Option<OsString>),
    #[error("invalid file name (stem): {0:?}")]
    InvalidName(Option<OsString>),
    #[error("parse failed: {0}")]
    ParseError(#[from] isf::ParseError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsfInfo {
    pub name: String,
    pub path: PathBuf,
    pub def: Isf,
}

impl AsRef<Path> for IsfInfo {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl IsfInfo {
    pub fn try_from_path(path: &Path) -> Result<Self, IsfInfoReadError> {
        let ext = path.extension();

        if Some("fs") == ext.map(OsStr::to_str).flatten() {
            let content = read_to_string(&path)?;
            let isf = isf::parse(&content)?;

            let name = path.file_stem();

            if let Some(name) = name.map(OsStr::to_str).flatten() {
                Ok(Self {
                    name: name.to_string(),
                    path: path.to_owned(),
                    def: isf,
                })
            } else {
                Err(IsfInfoReadError::InvalidName(name.map(OsStr::to_owned)))
            }
        } else {
            Err(IsfInfoReadError::InvalidExt(ext.map(OsStr::to_owned)))
        }
    }
}

impl Display for IsfInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
    }
}
