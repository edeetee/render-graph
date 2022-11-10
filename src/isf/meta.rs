use std::{fmt::{Display, Formatter}, fs::{read_dir, read_to_string}, path::{Path, PathBuf}, ffi::{OsStr, OsString}};


use isf::{Isf};

use thiserror::Error;

#[cfg(target_os="windows")]
pub fn default_isf_path() -> PathBuf {
    Path::new("C:\\ProgramData\\ISF")
}

#[cfg(target_os="macos")]
pub fn default_isf_path() -> PathBuf {
    Path::new("/Library/Graphics/ISF").to_path_buf()
}

pub fn parse_isf_shaders(path: impl AsRef<Path>) -> impl Iterator<Item = IsfInfo> {    
    read_dir(path)
        .unwrap()
        .into_iter()
        .filter_map(|file| {
            let path  = file.unwrap().path();

            match IsfInfo::new_from_path(&path) {
                Ok(isf) => {
                    Some(isf)
                },
                Err(err) => {
                    if matches!(err, IsfInfoReadError::ParseError(_)) {
                        eprintln!("Error parsing isf_meta file ({path:?}): {err}");
                    }
                    None
                },
            }
        })
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

#[derive(Debug, Clone, PartialEq)]
pub struct IsfInfo{
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
    pub fn new_from_path(path: &Path) -> Result<Self, IsfInfoReadError> {
        let ext = path.extension();
    
        if Some("fs") == ext.map(OsStr::to_str).flatten() {
            let content = read_to_string(&path)?;
            let isf = isf::parse(&content)?;

            let name = path.file_stem();

            if let Some(name) = name.map(OsStr::to_str).flatten() {
                Ok(
                    Self {
                        name: name.to_string(),
                        path: path.to_owned(),
                        def: isf,
                    }
                )
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