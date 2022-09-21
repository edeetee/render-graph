use std::{path::{Path, PathBuf}, fs::{read_dir, read_to_string}, fmt::{Display, Formatter}};

use egui::Rgba;
use isf::{Isf, Input, InputType};

use super::{connection_types::NodeInputDef, def::{NodeConnectionTypes, NodeValueTypes}};

pub fn parse_isf_shaders() -> impl Iterator<Item = (IsfPathInfo, Isf)> {
    // let files = current_dir()?;
    let shaders_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
    
    read_dir(shaders_dir)
        .unwrap()
        .into_iter()
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
                        eprintln!("Error parsing isf file ({path:?}): {err}");
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

#[derive(Clone, PartialEq)]
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

impl From<&InputType> for NodeConnectionTypes {
    fn from(ty: &InputType) -> Self {
        match ty {
            InputType::Image => NodeConnectionTypes::Texture2D,
            // InputType::Float(_) => Ok(NodeConnectionTypes::None),
            InputType::Point2d(_) => NodeConnectionTypes::Texture2D,
            _ => NodeConnectionTypes::None
        }
    }
}

impl From<&InputType> for NodeValueTypes {
    fn from(ty: &InputType) -> Self {
        match ty {
            InputType::Float(v) => NodeValueTypes::Float(v.default.unwrap_or_default()),
            InputType::Color(v) => {
                let mut slice: [f32; 4] = Default::default();
                if let Some(default) = &v.default{
                    for (from, to) in default.iter().zip(&mut slice){
                        *to = *from;
                    }
                }
                let rgba = Rgba::from_rgba_premultiplied(slice[0], slice[1], slice[2], slice[3]);
                NodeValueTypes::Color(rgba)
            },
            InputType::Point2d(v) => NodeValueTypes::Vec2(v.default.unwrap_or_default()),
            InputType::Bool(v) => NodeValueTypes::Bool(v.default.unwrap_or_default()),
            _ => NodeValueTypes::None
        }
    }
}

impl From<&Input> for NodeInputDef {
    fn from(input: &Input) -> Self {
        let ty = (&input.ty).into();
        let value = (&input.ty).into();
        
        Self {
            name: input.name.clone(),
            ty,
            value,
        }
    }
}