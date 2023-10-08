use glium;
use std;
use std::sync::Once;

pub(crate) const GL_INIT_ONCE: Once = std::sync::Once::new();

#[derive(Debug)]
pub(crate) struct RawGlBackend {
    pub(crate) size: (u32, u32),
}

impl RawGlBackend {
    ///Only run once!!!
    pub(crate) fn new(size: (u32, u32)) -> Self {
        GL_INIT_ONCE.call_once(|| {
            gl_loader::init_gl();
            gl::load_with(|s| gl_loader::get_proc_address(s).cast());
        });

        Self { size }
    }
}

/// only use this inside the ffgl callback!!!
/// Failure to do so will cause UB (invalid context etc)
unsafe impl glium::backend::Backend for RawGlBackend {
    fn swap_buffers(&self) -> Result<(), glium::SwapBuffersError> {
        Ok(())
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const std::os::raw::c_void {
        gl_loader::get_proc_address(symbol).cast()
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        self.size
    }

    fn is_current(&self) -> bool {
        true
    }

    unsafe fn make_current(&self) {}
}
