use std::rc::Rc;

use ffgl::logln;
// use egui_node_graph::graph;
// mod ffgl;
use ::ffgl::{ffgl_handler, FFGLHandler};
use glium::{Frame, backend::{Facade, Context}, Surface, framebuffer::{SimpleFrameBuffer, EmptyFrameBuffer, ColorAttachment}, texture::{TextureAnyImage, TextureAny}, Texture2d, GlObject, Rect, BlitMask};
use naga::back;

use crate::{graph::{def::GraphEditorState, ShaderGraphProcessor}, textures};

//todo: store graph in binary

// const path: &'static str = 

const graph_file: &[u8] = include_bytes!("../target/debug/render-graph-auto-save.json");

struct RawGlBackend{
    size: (u32, u32)
}

impl RawGlBackend {
    ///Only run once!!!
    fn new(size: (u32,u32)) -> Self {
        gl_loader::init_gl();
        gl::load_with(|s| gl_loader::get_proc_address(s).cast());

        Self{
            size
        }
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

    unsafe fn make_current(&self) {
        
    }
}

struct RenderGraphHandler {
    graph: crate::graph::def::Graph,
    processor: crate::graph::ShaderGraphProcessor,
    ctx: Rc<Context>,
}

impl FFGLHandler for RenderGraphHandler {
    unsafe fn new(inst_data: &ffgl::FFGLData) -> Self {
        let backend = RawGlBackend::new(inst_data.get_dimensions());

        let editor: GraphEditorState = serde_json::from_slice(graph_file).unwrap();
        let mut graph = editor.graph;
        
        let ctx = glium::backend::Context::new(backend, true, 
            glium::debug::DebugCallbackBehavior::Custom { callback: Box::new(
            |src, typ, sev, a, b, c| {
                logln!("{:?},{:?},{:?},{:?},{:?},{:?}", src, typ, sev, a, b, c);
            }
        ), synchronous: false }).unwrap();

        // glium::HeadlessRenderer
        // gl::st
        
        Self {
            processor: ShaderGraphProcessor::new_from_graph(&mut graph, &ctx),
            graph,
            ctx
        }
    }

    unsafe fn draw(&mut self, inst_data: &ffgl::FFGLData, frame_data: &ffgl::ProcessOpenGLStruct) {
        // let mut fb = SimpleFrameBuffer::new()
        // gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer)
        let (width, height) = inst_data.get_dimensions();
        // let fbo = Texture2d::from_id(
        //     &self.ctx, 
        //     glium::texture::UncompressedFloatFormat::U8U8U8U8, 
        //     frame_data.HostFBO, 
        //     false, 
        //     glium::texture::MipmapsOption::EmptyMipmaps, 
        //     glium::texture::Dimensions::Texture2d { width, height }
        // );

        // fbo.as_surface().cl
        // gl::ClearCo

        // let mut fb = SimpleFrameBuffer::new()
        gl::BindFramebuffer(gl::FRAMEBUFFER, frame_data.HostFBO);
        // gl::ClearColor(1.0, 0.0, 0.0, 1.0);
        // gl::Clear(gl::COLOR_BUFFER_BIT);

        let mut frame = Frame::new(self.ctx.clone(), self.ctx.get_framebuffer_dimensions());
        frame.clear_color(1.0, 0.0, 1.0, 1.0);
        frame.finish().unwrap();

        // self.processor.render_shaders(&mut self.graph, &self.ctx, |_,tex|{
        //     // frame.blit_buffers_from_simple_framebuffer(tex.as_su, source_rect, target_rect, filter)
        //     // tex.as_surface().fill(&fbo.as_surface(), glium::uniforms::MagnifySamplerFilter::Nearest);
        //     // gl::Copy
        //     // gl::GetBind
        //     self.ctx.swap_buffers().unwrap();
        //     // gl::BindFramebuffer(gl::FRAMEBUFFER, frame_data.HostFBO);
        //     copy(tex, frame_data.HostFBO, width, height);
        // });
        

        // fbo.as_surface().clear_color(0.0, 1.0, 0.0, 1.0);

        // for node in self.graph.nodes.values() {
        //     logln!("{:#?}", node);
        // }

        //return the framebuffer
        gl::BindFramebuffer(gl::FRAMEBUFFER, frame_data.HostFBO);

        // self.ctx.finish()
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
        height: tex_h
    };

    let target_rect = Rect {
        left: 0,
        bottom: 0,
        width,
        height
    };

    gl::BlitFramebuffer(src_rect.left as gl::types::GLint,
        src_rect.bottom as gl::types::GLint,
        (src_rect.left + src_rect.width) as gl::types::GLint,
        (src_rect.bottom + src_rect.height) as gl::types::GLint,
        target_rect.left as gl::types::GLint, target_rect.bottom as gl::types::GLint,
        (target_rect.left as i32 + target_rect.width as i32) as gl::types::GLint,
        (target_rect.bottom as i32 + target_rect.height as i32) as gl::types::GLint, 
        gl::COLOR_BUFFER_BIT,
        gl::NEAREST
    );
}

// ffgl::
ffgl_handler!(RenderGraphHandler);