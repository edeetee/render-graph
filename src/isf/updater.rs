use std::{time::SystemTime};
use glium::backend::Facade;
use thiserror::Error;

use crate::isf::{meta::{IsfInfo, IsfInfoReadError}, shader::{IsfShader, IsfShaderLoadError}};

pub struct IsfUpdater {
    pub modified: SystemTime
}

#[derive(Error, Debug)]
pub enum IsfReloadError{
    #[error("Could not read {0}")]
    Read(#[from] IsfInfoReadError),
    #[error("Could not load {0}")]
    Load(#[from] IsfShaderLoadError)
}

pub fn reload_ifs_shader(
    facade: &impl Facade,
    old_info: &IsfInfo,
) -> Result<(IsfInfo, IsfShader), IsfReloadError> {
    let new_info = IsfInfo::new_from_path(&old_info.path)?;
    let shader = IsfShader::new(facade, &new_info)?;

    Ok((new_info, shader))
}

impl IsfUpdater {
    pub fn reload_if_updated(&mut self, facade: &impl Facade, isf_info: &mut IsfInfo, shader: &mut IsfShader) -> Result<(), IsfReloadError> {
        let new_version = isf_info.path.metadata().unwrap().modified().unwrap();
        let diff = new_version.duration_since(self.modified);

        if let Ok(diff) = diff {
            //after a small time for fs jank
            if 10 < diff.as_millis() {
                //iterate version even on error (wait for change to retry update)
                self.modified = new_version;

                let (new_info, new_shader) = reload_ifs_shader(facade, &isf_info)?;
                println!("Reloaded shader: {}", isf_info.name);
                *shader = new_shader;
                *isf_info = new_info;
            }
        }

        Ok(())
    }
}