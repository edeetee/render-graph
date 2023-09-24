use std::{fmt::Formatter, rc::Rc, time::SystemTime};

use ffgl::logln;
// use egui_node_graph::graph;
// mod ffgl;
use ::ffgl::{ffgl_handler, FFGLHandler};
use gl::types;
use glium::{
    backend::{Context, Facade},
    framebuffer::{ColorAttachment, EmptyFrameBuffer, SimpleFrameBuffer},
    texture::{TextureAny, TextureAnyImage},
    BlitMask, CapabilitiesSource, Frame, GlObject, Rect, Surface, Texture2d,
};
use naga::back;

use crate::{
    common::persistent_state::PersistentState,
    graph::{def::GraphEditorState, ShaderGraphProcessor},
    textures,
};

struct RawGlBackend {
    size: (u32, u32),
}

impl RawGlBackend {
    ///Only run once!!!
    fn new(size: (u32, u32)) -> Self {
        gl_loader::init_gl();
        gl::load_with(|s| gl_loader::get_proc_address(s).cast());

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

pub struct RenderGraphHandler {
    graph: crate::graph::def::Graph,
    processor: crate::graph::ShaderGraphProcessor,
    texture_manager: crate::textures::TextureManager,
    ctx: Rc<Context>,
}

impl std::fmt::Debug for RenderGraphHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderGraphHandler")
            .field("graph", &self.graph)
            .field("processor", &self.processor)
            .finish()
    }
}

impl FFGLHandler for RenderGraphHandler {
    unsafe fn new(inst_data: &ffgl::FFGLData) -> Self {
        let backend = RawGlBackend::new(inst_data.get_dimensions());

        logln!("path: {:?}", PersistentState::default_path());

        let state = PersistentState::load_from_default_path();

        let mut graph = state.editor.graph;

        logln!("graph {:#?}", graph);

        let ctx = glium::backend::Context::new(
            backend,
            false,
            glium::debug::DebugCallbackBehavior::Custom {
                callback: Box::new(|src, typ, sev, a, b, c| {
                    logln!("src{src:?},typ{typ:?},sev{sev:?},a{a:?},b{b:?},c{c:?}");
                }),
                synchronous: false,
            },
        )
        .unwrap();

        let texture_manager = textures::TextureManager {
            res: inst_data.get_dimensions(),
            ..Default::default()
        };

        logln!("OPENGL_VERSION {}", ctx.get_opengl_version_string());

        Self {
            processor: ShaderGraphProcessor::new_from_graph(&mut graph, &ctx),
            graph,
            texture_manager,
            ctx,
        }
    }

    unsafe fn draw(&mut self, inst_data: &ffgl::FFGLData, frame_data: &ffgl::ProcessOpenGLStruct) {
        let (width, height) = inst_data.get_dimensions();
        let mod_sec = inst_data
            .host_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32()
            % 1.0;

        let mut frame = Frame::new(self.ctx.clone(), inst_data.get_dimensions());
        frame.clear_color(0.0, 1.0 - mod_sec, 0.0, 1.0);

        self.processor.render_shaders(
            &mut self.graph,
            &self.ctx,
            &mut self.texture_manager,
            |node_id, tex| {
                // frame.blit_buffers_from_simple_framebuffer(tex.as_su, source_rect, target_rect, filter)
                tex.as_surface()
                    .fill(&frame, glium::uniforms::MagnifySamplerFilter::Nearest);
                // gl::Copy
                // gl::GetBind
                // self.ctx.swap_buffers().unwrap();
                // gl::BindFramebuffer(gl::FRAMEBUFFER, frame_data.HostFBO);
                copy(tex, frame_data.HostFBO, width / 2, height);
            },
        );

        frame.finish().unwrap();
        gl::BindFramebuffer(gl::FRAMEBUFFER, frame_data.HostFBO);
    }
}

unsafe fn copy(tex: &Texture2d, frame_data: gl::types::GLuint, width: u32, height: u32) {
    gl::BindFramebuffer(gl::READ_FRAMEBUFFER, tex.get_id());
    gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, frame_data);

    let (tex_w, tex_h) = tex.dimensions();

    let src_rect = Rect {
        left: 0,
        bottom: 0,
        width: tex_w,
        height: tex_h,
    };

    let target_rect = Rect {
        left: 0,
        bottom: 0,
        width,
        height,
    };

    gl::BlitFramebuffer(
        src_rect.left as gl::types::GLint,
        src_rect.bottom as gl::types::GLint,
        (src_rect.left + src_rect.width) as gl::types::GLint,
        (src_rect.bottom + src_rect.height) as gl::types::GLint,
        target_rect.left as gl::types::GLint,
        target_rect.bottom as gl::types::GLint,
        (target_rect.left as i32 + target_rect.width as i32) as gl::types::GLint,
        (target_rect.bottom as i32 + target_rect.height as i32) as gl::types::GLint,
        gl::COLOR_BUFFER_BIT,
        gl::LINEAR,
    );
}

ffgl_handler!(RenderGraphHandler);
