use std::{
    borrow::BorrowMut,
    cell::OnceCell,
    ffi::{CStr, CString},
    fmt::Formatter,
    rc::Rc,
    sync::{Once, OnceLock},
    time::SystemTime,
};

use color_eyre::owo_colors::OwoColorize;
use egui_node_graph::{InputId, NodeId};
use ffgl::{
    logln,
    parameters::{BasicParam, ParamValue},
    validate, Param,
};
// use egui_node_graph::graph;
// mod ffgl;
use ::ffgl::{ffgl_handler, FFGLHandler};
use gl::{
    types::{self, GLint},
    FRAMEBUFFER,
};
use glium::{
    backend::{Context, Facade},
    framebuffer::{ColorAttachment, EmptyFrameBuffer, RenderBuffer, SimpleFrameBuffer},
    texture::{TextureAny, TextureAnyImage},
    BlitMask, BlitTarget, CapabilitiesSource, Display, Frame, GlObject, Rect, Surface, Texture2d,
};
use itertools::Itertools;
use naga::back;

use crate::{
    common::{def::UiValue, persistent_state::PersistentState},
    graph::{
        def::{GraphEditorState, GraphState},
        ShaderGraphProcessor,
    },
    textures,
    util::SelfCall,
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

#[derive(Debug)]
pub struct NodeParam {
    node_id: NodeId,
    param_id: InputId,
    group_name: CString,
    name: CString,
    value: ParamValue,
}

type GraphInput = egui_node_graph::InputParam<crate::common::connections::ConnectionType, UiValue>;

impl NodeParam {
    pub fn new(input: &GraphInput, input_name: &str, node_name: &str) -> Option<Self> {
        if let Some(value) = (&input.value).into() {
            Some(NodeParam {
                node_id: input.node,
                param_id: input.id,
                group_name: CString::new(node_name.as_bytes()).unwrap(),
                name: CString::new(format!(
                    "{}.{input_name}",
                    node_name.chars().take(3).collect::<String>()
                ))
                .unwrap(),
                value,
            })
        } else {
            None
        }
    }
}

impl From<&UiValue> for Option<ParamValue> {
    fn from(value: &UiValue) -> Self {
        match value {
            UiValue::Float(vf) => Some(ParamValue::Float(vf.value)),
            UiValue::Mat4(m) => Some(ParamValue::Float(m.scale)),
            _ => None,
        }
    }
}

impl Param for NodeParam {
    fn name(&self) -> &CStr {
        &self.name
    }

    fn group(&self) -> &CStr {
        &self.group_name
    }

    fn get(&self) -> ffgl::parameters::ParamValue {
        self.value
    }

    fn set(&mut self, value: ffgl::parameters::ParamValue) {
        self.value = value
    }
}

pub struct StaticState {
    pub save_state: PersistentState,
    pub params: Vec<NodeParam>,
}

static mut INSTANCE: OnceLock<StaticState> = OnceLock::new();

impl StaticState {
    fn new() -> Self {
        let save_state = PersistentState::load_from_default_path();
        let graph = &save_state.editor.graph;

        let params = graph
            .nodes
            .iter()
            .map(|(node_id, node)| {
                let node_name = save_state
                    .state
                    .node_names
                    .get(&node_id)
                    .map(|n| format!("{n}"))
                    .unwrap_or(format!("{node_id:?}"));

                node.inputs
                    .iter()
                    //add reference to closure
                    .map(|(n, id)| (n, id, &save_state.editor.graph.inputs[*id]))
                    //move string to closure
                    .filter_map(move |(input_name, input_id, input)| {
                        NodeParam::new(input, &node_name, &input_name)
                    })
            })
            .flatten()
            .collect_vec();

        Self { save_state, params }
    }

    pub fn get_mut() -> &'static mut Self {
        Self::get();
        unsafe { INSTANCE.get_mut() }.unwrap()
    }

    pub fn get() -> &'static Self {
        unsafe { INSTANCE.get_or_init(Self::new) }
    }
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

static PARAMS: &[BasicParam] = &[
    BasicParam::standard("12345\0"),
    BasicParam::standard("test22\0"),
];

impl FFGLHandler for RenderGraphHandler {
    type Param = NodeParam;

    unsafe fn new(inst_data: &ffgl::FFGLData) -> Self {
        let backend = Rc::new(RawGlBackend::new(inst_data.get_dimensions()));

        logln!("BACKEND: {backend:?}");

        let mut graph = StaticState::get().save_state.editor.graph.clone();

        for (node_id, node) in &graph.nodes {
            logln!("{node_id:?}: {}", node.label);
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
            graph_state: StaticState::get().save_state.state.clone(),
            graph,
            texture_manager,
            ctx,
        }
    }

    fn params() -> &'static [Self::Param] {
        &StaticState::get().params
    }

    fn params_mut() -> &'static mut [Self::Param] {
        &mut StaticState::get_mut().params
    }

    // fn params(&self) -> &[ffgl::parameters::Param] {
    //     &self.params
    // }

    // fn params_mut(&mut self) -> &mut [ffgl::parameters::Param] {
    //     &mut self.params
    // }

    unsafe fn draw(
        &mut self,
        inst_data: &ffgl::FFGLData,
        frame_data: &ffgl::ffgl::ProcessOpenGLStruct,
    ) {
        let viewport = [
            inst_data.viewport.x as i32,
            inst_data.viewport.y as i32,
            inst_data.viewport.width as i32,
            inst_data.viewport.height as i32,
        ];

        // validate_viewport(&viewport);

        //glium expects default framebuffer
        gl::BindFramebuffer(FRAMEBUFFER, 0);

        // validate_viewport(&viewport);
        // validate::validate_context_state();

        // self.ctx.rebuild(self.backend.clone()).unwrap();

        let res = inst_data.get_dimensions();

        let frame = Frame::new(self.ctx.clone(), (res.0, res.1));
        let rb = RenderBuffer::new(
            &self.ctx,
            glium::texture::UncompressedFloatFormat::F32F32F32F32,
            res.0,
            res.1,
        )
        .unwrap();

        let fb = &mut SimpleFrameBuffer::new(&self.ctx, &rb).unwrap();
        // fb.clear_color(0.0, 0.0, 1.0, 1.0);

        self.render_frame(inst_data, fb);

        // validate_viewport(&viewport);

        //puts the texture into the framebuffer
        fb.fill(&frame, glium::uniforms::MagnifySamplerFilter::Nearest);

        // gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, frame_data.HostFBO);
        blit_fb(res, res);

        frame.finish().unwrap();

        //REQUIRED as host takes control of textures
        self.texture_manager.clear();

        //reset to what host expects
        // gl_reset(frame_data);
        // validate::validate_context_state();

        // validate_viewport(&viewport);
    }
}

unsafe fn validate_viewport(viewport: &[i32; 4]) {
    // let mut dims: [i32; 4] = [0; 4];
    // gl::GetIntegerv(gl::SCISSOR_BOX, &mut dims[0]);
    // assert_eq!(&dims, viewport, "SCISSOR_BOX wrong value: {dims:?}");

    let scissor_enabled = gl::IsEnabled(gl::SCISSOR_TEST);
    assert_eq!(scissor_enabled, gl::FALSE, "SCISSOR_TEST is enabled");

    let mut dims: [i32; 4] = [0; 4];
    gl::GetIntegerv(gl::VIEWPORT, &mut dims[0]);
    assert_eq!(&dims, viewport, "VIEWPORT wrong value: {dims:?}");
}

impl RenderGraphHandler {
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

        for param in Self::params() {
            // let node = self.graph.nodes.get(param.node_id).unwrap();
            let input = self.graph.inputs.get_mut(param.param_id).unwrap();

            let value = match param.value {
                ParamValue::Float(f) => f,
            };

            match &mut input.value {
                UiValue::Float(vf) => vf.value = value,
                UiValue::Mat4(m) => m.scale = value,
                _ => {}
            }
        }

        let ends = self.processor.render_shaders(
            &mut self.graph,
            &self.ctx,
            &mut self.texture_manager,
            |_, _| {},
        );

        for end in ends {
            if let Some(tex) = end {
                tex.as_surface()
                    .fill(target, glium::uniforms::MagnifySamplerFilter::Nearest);
            }
        }

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

unsafe fn gl_reset(frame_data: &ffgl::ffgl::ProcessOpenGLStructTag) {
    let mut gl_int = 0;
    gl::UseProgram(0);

    let mut num_samplers = 0;
    gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut num_samplers);

    for texture_type in TEXTURE_TYPES.iter() {
        for sampler in 0..num_samplers {
            gl::ActiveTexture(gl::TEXTURE0 + sampler as u32);
            // Check if textures are unbound for the current texture unit.
            gl::GetIntegerv(texture_type.binding, &mut gl_int);
            // gl::BindTexture(texture_type.target, 0);
        }
    }

    gl::ActiveTexture(gl::TEXTURE0);

    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    gl::BindBuffer(gl::VERTEX_BINDING_BUFFER, 0);
    gl::BindVertexArray(0);
    gl::Disable(gl::BLEND);

    gl::BlendFunc(gl::ONE, gl::ZERO);

    // gl::BindVertexBuffer(0, 0, 0, 0);

    // gl::VertexArrayElementBuffer(vaobj, buffer)
    // gl::BindTextureUnit(0, 0);
    gl::BindFramebuffer(gl::FRAMEBUFFER, frame_data.HostFBO);
}

unsafe fn blit_fb((read_w, read_h): (u32, u32), (write_w, write_h): (u32, u32)) {
    let src_rect = BlitTarget {
        left: 0,
        bottom: 0,
        width: read_w as i32,
        height: read_h as i32,
    };

    let target_rect = BlitTarget {
        left: 0 as u32,
        bottom: 0 as u32,
        width: write_w as i32,
        height: write_h as i32,
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
        gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT,
        gl::NEAREST,
    );
}

ffgl_handler!(RenderGraphHandler);
