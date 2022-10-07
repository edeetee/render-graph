use std::{fmt::{Display, Formatter}, fs::{read_dir, read_to_string}, path::{Path, PathBuf}};

use egui::Rgba;
use isf::{Input, InputType, Isf};
use strum::Display;

pub fn default_isf_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders")
}

pub fn parse_isf_shaders(path: impl AsRef<Path>) -> impl Iterator<Item = (IsfPathInfo, Isf)> {    
    read_dir(path)
        .unwrap()
        .into_iter()
        .filter_map(|file| {
            let path  = file.unwrap().path();

            match try_read_isf(path.clone()) {
                Ok(isf) => {
                    Some(isf)
                },
                Err(err) => {
                    if matches!(err, IsfReadError::ParseError(_)) {
                        eprintln!("Error parsing isf_meta file ({path:?}): {err}");
                    }
                    None
                },
            }
        })
}

pub fn try_read_isf(path: PathBuf) -> Result<(IsfPathInfo, Isf), IsfReadError> {
    let ext = path.extension()
        .map(|ext| ext.to_str())
        .flatten()
        .ok_or(IsfReadError::InvalidExt)?;

    if ext == "fs" {
        let content = read_to_string(&path)?;
        let isf = isf::parse(&content)?;

        Ok((path.into(), isf))
    } else {
        Err(IsfReadError::InvalidExt)
    }
}

#[derive(Display)]
pub enum IsfReadError {
    IoError(std::io::Error),
    InvalidExt,
    ParseError(isf::ParseError),
}

impl From<std::io::Error> for IsfReadError {
    fn from(err: std::io::Error) -> Self {
        IsfReadError::IoError(err)
    }
}

impl From<isf::ParseError> for IsfReadError {
    fn from(err: isf::ParseError) -> Self {
        IsfReadError::ParseError(err)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsfPathInfo{
    pub name: String,
    pub path: PathBuf
}

impl AsRef<Path> for IsfPathInfo {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl From<PathBuf> for IsfPathInfo {
    fn from(path: PathBuf) -> Self {
        Self {
            name: path.file_stem().unwrap().to_str().unwrap().to_string(),
            // version: path.metadata().unwrap().modified().unwrap(),
            path,
        }
    }
}

impl Display for IsfPathInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
    }
}