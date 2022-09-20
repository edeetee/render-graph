use std::{env::current_dir, path::{Path, PathBuf}, fs::{read_dir, read_to_string}, ffi::OsStr, convert::{TryFrom, TryInto}, fmt::{Display, Formatter}};

use isf::{Isf, Input, InputType};

use super::{connection_types::NodeInputDef, def::{NodeConnectionTypes, NodeValueTypes}};

pub fn parse_isf_shaders() -> impl Iterator<Item = (IsfFile, Isf)> {
    // let files = current_dir()?;
    let shaders_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
    
    read_dir(shaders_dir)
        .unwrap()
        .into_iter()
        .filter_map(|file| {
            let path  = file.unwrap().path();
            let ext = path.extension()?.to_str()?;

            if ext == "fs" {
                let content = read_to_string(path.clone()).unwrap();
                let isf = isf::parse(&content);
                return isf.ok().map(|isf| (path.into(), isf))
            }

            None
        })
}

#[derive(Clone, PartialEq)]
pub struct IsfFile{
    pub name: String,
    pub path: PathBuf
}

impl AsRef<Path> for IsfFile {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl From<PathBuf> for IsfFile {
    fn from(path: PathBuf) -> Self {
        Self {
            path,
            name: path.file_stem().unwrap().to_str().unwrap().to_string(),
        }
    }
}

impl Display for IsfFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
    }
}

impl TryFrom<&InputType> for NodeConnectionTypes {
    type Error = ();

    fn try_from(ty: &InputType) -> Result<Self, Self::Error> {
        match ty {
            InputType::Image => Ok(NodeConnectionTypes::Texture2D),
            InputType::Float(_) => Ok(NodeConnectionTypes::Float),
            InputType::Point2d(_) => Ok(NodeConnectionTypes::Texture2D),
            _ => Err(())
        }
    }
}

impl TryFrom<&Input> for NodeInputDef {
    type Error = ();

    fn try_from(input: &Input) -> Result<Self, Self::Error> {
        let ty: NodeConnectionTypes = (&input.ty).try_into()?;
        // let value: NodeValueTypes = 
        
        Ok(Self {
            name: input.name,
            ty,
            value: input.ty.into(),
        })
    }
}