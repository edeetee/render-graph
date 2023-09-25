use std::{borrow::BorrowMut, fmt::Formatter, rc::Rc, sync::Once, time::SystemTime};

use ffgl::{logln, validate};
// use egui_node_graph::graph;
// mod ffgl;
use ::ffgl::{ffgl_handler, FFGLHandler};
use gl::{types, FRAMEBUFFER};
use glium::{
    backend::{Context, Facade},
    framebuffer::{ColorAttachment, EmptyFrameBuffer, RenderBuffer, SimpleFrameBuffer},
    texture::{TextureAny, TextureAnyImage},
    BlitMask, BlitTarget, CapabilitiesSource, Display, Frame, GlObject, Rect, Surface, Texture2d,
};
use naga::back;

use crate::{
    common::persistent_state::PersistentState,
    graph::{
        def::{GraphEditorState, GraphState},
        ShaderGraphProcessor,
    },
    textures,
};

const GL_INIT_ONCE: Once = std::sync::Once::new();

#[derive(Debug)]
struct RawGlBackend {
    size: (u32, u32),
}

impl RawGlBackend {
    ///Only run once!!!
    fn new(size: (u32, u32)) -> Self {
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

pub struct RenderGraphHandler {
    graph: crate::graph::def::Graph,
    graph_state: GraphState,
    processor: crate::graph::ShaderGraphProcessor,
    texture_manager: crate::textures::TextureManager,
    backend: Rc<RawGlBackend>,
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
        let backend = Rc::new(RawGlBackend::new(inst_data.get_dimensions()));

        logln!("BACKEND: {backend:?}");

        let state = PersistentState::load_from_default_path();

        let mut graph = state.editor.graph;

        for (node_id, node) in &graph.nodes {
            logln!("{node_id:?}: {node:#?}");
        }

        let ctx = glium::backend::Context::new(
            backend.clone(),
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
            backend,
            processor: ShaderGraphProcessor::new_from_graph(&mut graph, &ctx),
            graph_state: state.state,
            graph,
            texture_manager,
            ctx,
        }
    }

    unsafe fn draw(&mut self, inst_data: &ffgl::FFGLData, frame_data: &ffgl::ProcessOpenGLStruct) {
        //reset to what glium expects the state to be in
        gl::BindFramebuffer(FRAMEBUFFER, 0);
        self.ctx.rebuild(self.backend.clone()).unwrap();

        let res = inst_data.get_dimensions();
        // let rb = RenderBuffer::new(
        //     &self.ctx,
        //     glium::texture::UncompressedFloatFormat::F32F32F32F32,
        //     res.0,
        //     res.1,
        // )
        // .unwrap();

        //Higher resolution than this seems to cut off the image
        let mut frame = Frame::new(self.ctx.clone(), (res.0 / 2, res.1 / 2));
        // let frame_dimen = frame.get_dimensions();

        validate::validate_context_state();

        // let tex = self.texture_manager.get_color(&self.ctx);
        let fb = &mut frame;
        fb.clear_color(0.0, 0.0, 1.0, 1.0);

        self.render_frame(inst_data, fb);

        // gl::BindFramebuffer(FRAMEBUFFER, 0);
        // gl::ClearColor(0.0, 0.0, 1.0, 1.0);

        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, frame_data.HostFBO);
        blit_fb(frame.get_dimensions(), res);

        frame.finish().unwrap();
        self.ctx.finish();

        //reset to what host expects
        gl_reset(frame_data);
    }
}

impl RenderGraphHandler {
    /**
    ### WORKING:
     - Texture/Frame Clear
     - Texture -> Frame
     - Frame -> Output

     ### Not working:
     - Shader render seems to be wrong size??
    */
    fn render_frame(&mut self, inst_data: &ffgl::FFGLData, target: &mut impl Surface) {
        let ramp = 1.0
            - inst_data
                .host_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f32()
                % 1.0;

        target.clear_color(ramp, 0.0, 0.0, 1.0);

        self.processor
            .update(&mut self.graph, &self.graph_state, &self.ctx);

        self.processor.render_shaders(
            &mut self.graph,
            &self.ctx,
            &mut self.texture_manager,
            |node_id, tex| {
                // frame.clear_color(ramp, 0.0, ramp, 1.0);
                // tex.as_surface().clear_color(0.0, ramp, 0.0, 1.0);

                // logln!(
                //     "rendering {tex:?} {:?} {:?}",
                //     tex.get_internal_format().unwrap(),
                //     tex.get_texture_type()
                // );
                tex.as_surface()
                    .fill(target, glium::uniforms::MagnifySamplerFilter::Nearest);
            },
        );

        for (node_id, node) in self.graph.nodes.iter() {
            for err in vec![
                &node.user_data.render_error,
                &node.user_data.create_error,
                &node.user_data.update_error,
            ] {
                if let Some(err) = err {
                    logln!("ERROR on {node_id:?}: {err:?}");
                }
            }
        }
    }
}

struct TextureType {
    target: u32,
    binding: u32,
}
const TEXTURE_TYPES: [TextureType; 2] = [
    TextureType {
        target: gl::TEXTURE_1D,
        binding: gl::TEXTURE_BINDING_1D,
    },
    TextureType {
        target: gl::TEXTURE_2D,
        binding: gl::TEXTURE_BINDING_2D,
    },
    // Add other texture types here...
];

unsafe fn gl_reset(frame_data: &ffgl::ProcessOpenGLStructTag) {
    let mut gl_int = 0;
    gl::UseProgram(0);

    let mut num_samplers = 0;
    gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut num_samplers);

    for texture_type in TEXTURE_TYPES.iter() {
        for sampler in 0..num_samplers {
            gl::ActiveTexture(gl::TEXTURE0 + sampler as u32);
            // Check if textures are unbound for the current texture unit.
            gl::GetIntegerv(texture_type.binding, &mut gl_int);
            gl::BindTexture(texture_type.target, 0);
        }
    }

    gl::ActiveTexture(gl::TEXTURE0);

    // gl::VertexArrayElementBuffer(vaobj, buffer)
    // gl::BindTextureUnit(0, 0);
    gl::BindFramebuffer(gl::FRAMEBUFFER, frame_data.HostFBO);
}

// fn copy_hostfbo(src: &impl Surface, target: &impl Surface) {
//     let src_dim = src.get_dimensions();
//     let src_rect = Rect {
//         left: 0,
//         bottom: 0,
//         width: src_dim.0 / 2 as u32,
//         height: src_dim.1 as u32,
//     };
//     let target_dim = target.get_dimensions();
//     let target_rect = glium::BlitTarget {
//         left: 0,
//         bottom: 0,
//         width: (target_dim.0 / 2) as i32,
//         height: target_dim.1 as i32,
//     };
//     src.blit_color(
//         &src_rect,
//         target,
//         &target_rect,
//         glium::uniforms::MagnifySamplerFilter::Nearest,
//     )
// }

unsafe fn blit_fb((read_w, read_h): (u32, u32), (write_w, write_h): (u32, u32)) {
    let src_rect = BlitTarget {
        left: 0,
        bottom: 0,
        width: (read_w as i32),
        height: (read_h as i32),
    };

    let target_rect = BlitTarget {
        left: (write_w as f32 * 0.05) as u32,
        bottom: (write_h as f32 * 0.05) as u32,
        width: (write_w as f32 * 0.9) as i32,
        height: (write_h as f32 * 0.9) as i32,
    };

    gl::BlitFramebuffer(
        src_rect.left as gl::types::GLint,
        src_rect.bottom as gl::types::GLint,
        (src_rect.left as i32 + src_rect.width) as gl::types::GLint,
        (src_rect.bottom as i32 + src_rect.height) as gl::types::GLint,
        (target_rect.left) as gl::types::GLint,
        (target_rect.bottom) as gl::types::GLint,
        (target_rect.left as i32 + target_rect.width) as gl::types::GLint,
        (target_rect.bottom as i32 + target_rect.height) as gl::types::GLint,
        gl::COLOR_BUFFER_BIT,
        gl::NEAREST,
    );
}

ffgl_handler!(RenderGraphHandler);
