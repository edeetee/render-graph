use std::{fmt::Formatter, rc::Rc, sync::OnceLock};

use ffgl::{logln, parameters::ParamValue};
// use egui_node_graph::graph;
// mod ffgl;
use ::ffgl::{ffgl_handler, FFGLHandler};
use glium::{
    backend::Context,
    framebuffer::{RenderBuffer, SimpleFrameBuffer},
    BlitTarget, Frame, Surface,
};
use itertools::Itertools;

use graph::{def::UiValue, GraphState};
use persistence::PersistentState;

mod gl_backend;
mod node_param;
mod validate_gl;

pub struct StaticState {
    pub save_state: PersistentState,
    pub params: Vec<node_param::NodeParam>,
}

static mut INSTANCE: OnceLock<StaticState> = OnceLock::new();

impl StaticState {
    fn new() -> Self {
        let save_state = PersistentState::load_from_default_path();
        let graph = &save_state.graph;

        let params = graph
            .nodes
            .iter()
            .map(|(node_id, node)| {
                let node_name = save_state
                    .node_names
                    .get(node_id)
                    .map(|n| format!("{n}"))
                    .unwrap_or(format!("{node_id:?}"));

                node.inputs
                    .iter()
                    //add reference to closure
                    .map(|(n, id)| (n, id, &graph.inputs[*id]))
                    //move string to closure
                    .filter_map(move |(input_name, _input_id, input)| {
                        node_param::NodeParam::new(input, &node_name, &input_name)
                    })
            })
            .flatten()
            .collect_vec();

        Self { save_state, params }
    }

    pub fn get() -> &'static Self {
        unsafe { INSTANCE.get_or_init(Self::new) }
    }
}

pub struct Instance {
    graph: graph::Graph,
    graph_state: GraphState,
    texture_manager: graph::TextureManager,
    ctx: Rc<Context>,
    params: Vec<node_param::NodeParam>,
    backend: Rc<gl_backend::RawGlBackend>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderGraphHandler")
            .field("graph", &self.graph)
            .finish()
    }
}

impl FFGLHandler for Instance {
    type Param = node_param::NodeParam;

    unsafe fn new(inst_data: &ffgl::FFGLData) -> Self {
        let backend = Rc::new(gl_backend::RawGlBackend::new(inst_data.get_dimensions()));

        logln!("BACKEND: {backend:?}");

        let mut graph = StaticState::get().save_state.graph.clone();

        for (node_id, node) in &graph.nodes {
            logln!("{node_id:?}: {}", node.label);
        }

        let ctx = glium::backend::Context::new(
            backend.clone(),
            false,
            glium::debug::DebugCallbackBehavior::Ignore, // glium::debug::DebugCallbackBehavior::Custom {
                                                         //     callback: Box::new(|src, typ, sev, a, b, c| {
                                                         //         logln!("src{src:?},typ{typ:?},sev{sev:?},a{a:?},b{b:?},c{c:?}");
                                                         //     }),
                                                         //     synchronous: false,
                                                         // },
        )
        .unwrap();

        let texture_manager = graph::TextureManager {
            res: inst_data.get_dimensions(),
            ..Default::default()
        };

        logln!("OPENGL_VERSION {}", ctx.get_opengl_version_string());

        Self {
            graph_state: GraphState::from_persistent_state(
                &mut graph,
                StaticState::get().save_state.node_names.clone(),
                StaticState::get().save_state.animator.clone(),
                &ctx,
            )
            .expect("Failed to load graph state"),
            graph,
            texture_manager,
            ctx,
            params: StaticState::get().params.clone(),
            backend,
        }
    }

    fn params() -> &'static [Self::Param] {
        &StaticState::get().params
    }

    fn params_mut(&mut self) -> &mut [Self::Param] {
        &mut self.params
    }

    unsafe fn draw(
        &mut self,
        inst_data: &ffgl::FFGLData,
        frame_data: &ffgl::ffgl::ProcessOpenGLStruct,
    ) {
        let res = inst_data.get_dimensions();
        self.ctx.rebuild(self.backend.clone()).unwrap();

        let frame = Frame::new(self.ctx.clone(), (res.0, res.1));
        let rb = RenderBuffer::new(
            &self.ctx,
            glium::texture::UncompressedFloatFormat::U8U8U8U8,
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

        //reset to what host expects
        // gl_reset(frame_data);
        // validate::validate_context_state();

        // validate_viewport(&viewport);
    }
}
impl Instance {
    fn render_frame(&mut self, _inst_data: &ffgl::FFGLData, target: &mut impl Surface) {
        self.graph_state.update(&mut self.graph, &self.ctx);

        for param in &self.params {
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

        let resp = self.graph_state.processor.render_shaders(
            &mut self.graph,
            &self.ctx,
            &mut self.texture_manager,
            |_, _| {},
        );

        for end in resp.terminating_textures {
            if let Some(tex) = end {
                tex.as_surface()
                    .fill(target, glium::uniforms::MagnifySamplerFilter::Nearest);
            }
        }

        for (node_id, err) in resp.errors {
            logln!("ERROR on {} {err:?}", self.graph.nodes[node_id].label);
        }

        for (node_id, duration) in resp.times {
            let us = duration.as_micros();
            let name = self
                .graph_state
                .node_names
                .get(node_id)
                .map(|n| format!("{n}"))
                .unwrap_or_else(|| format!("{node_id:?}"));

            if 100 < us {
                logln!("Render time for {name}: {us}us");
            }
        }
    }
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
        gl::COLOR_BUFFER_BIT,
        gl::NEAREST,
    );
}

ffgl_handler!(Instance);
