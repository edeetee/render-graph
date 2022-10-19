use std::{fmt::{Display, Formatter}, fs::{read_dir, read_to_string}, path::{Path, PathBuf}, ffi::{OsStr, OsString}};


use isf::{Isf};

use thiserror::Error;

pub fn default_isf_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders")
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

        let ext_str = ext.map(|ext| ext.to_str())
            .flatten();
    
        if ext_str == Some("fs") {
            let content = read_to_string(&path)?;
            let isf = isf::parse(&content)?;

            Ok(
                Self {
                    name: path.file_stem().unwrap().to_str().unwrap().to_string(),
                    path: path.to_owned(),
                    def: isf,
                }
            )
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