use std::{fmt::{Display, Formatter}, fs::{read_dir, read_to_string}, path::{Path, PathBuf}};

use egui::Rgba;
use isf::{Input, InputType, Isf};

pub fn parse_isf_shaders() -> impl Iterator<Item = (IsfPathInfo, Isf)> {
    // let files = current_dir()?;
    let shaders_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
    
    read_dir(shaders_dir)
        .unwrap()
        .into_iter()
        // .flat_map(|f| {
        //     let dir_entry = f.unwrap();

        //     if dir_entry.file_type().unwrap().is_dir() {
        //         read_dir(dir_entry.path()).unwrap().into_iter()
        //     } else {
        //         std::iter::once(f)
        //     }
        // })
        .filter_map(|file| {
            let path  = file.unwrap().path();
            let ext = path.extension()?.to_str()?;

            if ext == "fs" {
                let content = read_to_string(&path).unwrap();
                return match isf::parse(&content) {
                    Ok(isf) => {
                        Some((path.into(), isf))
                    },
                    Err(err) => {
                        eprintln!("Error parsing isf_meta file ({path:?}): {err}");
                        None
                    },
                }
            }

            None
        })
}

//TODO: single with result
// pub fn parse(){

// }

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