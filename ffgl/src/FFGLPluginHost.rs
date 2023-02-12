use crate::ffgl::*;

pub struct FFGLPluginHost{
    inst: FFGLPluginInstance
}

impl FFGLPluginHost {
    fn new() -> Self {
        let inst = unsafe {
            FFGLPluginInstance::new()
        };
        

        Self{
            inst
        }
    }
}