use std::{fs::read_to_string, time::SystemTime};

use egui_glium::EguiGlium;
use egui_node_graph::{GraphEditorState, NodeId, NodeResponse};
use glium::{
    backend::Facade,
    framebuffer::{RenderBuffer, SimpleFrameBuffer},
    Display, Surface,
};


use itertools::Itertools;
use ouroboros::self_referencing;
use slotmap::{SecondaryMap, SparseSecondaryMap};

use super::{
    connection_types::ComputedInputs,
    def::{self, *},
    graph::ShaderGraph,
    isf::IsfPathInfo,
    shaders::NodeShader,
    textures::NodeTextures,
};

pub(crate) type EditorState =
    GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;

#[self_referencing]
pub struct OutputTarget {
    rb: RenderBuffer,

    #[borrows(rb)]
    #[covariant]
    fb: SimpleFrameBuffer<'this>,
}

#[derive(Default)]
pub struct ShaderGraphProcessor {
    graph: ShaderGraph,
    output_targets: SparseSecondaryMap<NodeId, OutputTarget>,
    textures: SecondaryMap<NodeId, NodeTextures>,
    shaders: SecondaryMap<NodeId, NodeShader>,
    versions: SecondaryMap<NodeId, SystemTime>,
}

impl ShaderGraphProcessor {
    pub fn new() -> Self {
        Default::default()
    }

    fn add_dangling_output(&mut self, facade: &impl Facade, node_id: NodeId) {
        // let is_output_target = node.outputs(&self.graph.graph_ref()).any(|o| o.typ == NodeConnectionTypes::Texture2D);

        let output_target = OutputTargetBuilder {
            rb: RenderBuffer::new(
                facade,
                glium::texture::UncompressedFloatFormat::F32F32F32F32,
                512,
                512,
            )
            .unwrap(),
            fb_builder: |rb| SimpleFrameBuffer::new(facade, rb).unwrap(),
        }
        .build();
        self.output_targets.insert(node_id, output_target);
    }

    pub fn node_event(
        &mut self,
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
        event: NodeResponse<def::GraphResponse, NodeData>,
    ) {
        match event {
            egui_node_graph::NodeResponse::CreatedNode(node_id) => {
                // let node = &self.graph[node_id];

                let textures = NodeTextures::new(facade, egui_glium);
                // new_shader.init_inputs(self.graph[node_id].input_ids().map(|input_id| &mut self.graph.[input_id]));

                self.graph[node_id].user_data.result = Some(textures.clone_screen_tex_id());

                self.textures.insert(node_id, textures);

                let node = &self.graph[node_id];

                let template = &node.user_data.template;

                match template {
                    NodeTypes::Isf { file, .. } => {
                        self.versions
                            .insert(node_id, file.path.metadata().unwrap().modified().unwrap());
                    }
                    _ => {}
                }

                // if matches!(template, NodeTypes::Isf { file, .. }) {
                //     self.versions.insert(node_id, file);
                // }

                //only add if needed ()
                if let Some(shader) = NodeShader::new(template, facade) {
                    if let Ok(shader) = shader {
                        self.shaders.insert(node_id, shader);
                    } else {
                        eprintln!("Error creating shader for node: {:?}", node_id);
                    }
                    // self.shaders.insert(node_id, shader);
                }

                for input in node.inputs(self.graph.graph_ref()) {
                    if input.typ == NodeConnectionTypes::Texture2D {
                        let connected_output = self.graph.graph_ref().connection(input.id);
                        if let Some(output_id) = connected_output {
                            let connected_node_id = self.graph.graph_ref()[output_id].node;

                            self.output_targets.remove(connected_node_id);
                        }
                    }
                }

                self.add_dangling_output(facade, node_id);
            }

            //may create new output target
            NodeResponse::DisconnectEvent { output, .. } => {
                self.add_dangling_output(facade, self.graph.graph_ref().outputs[output].node);
            }

            NodeResponse::ConnectEventEnded { output, .. } => {
                self.output_targets
                    .remove(self.graph.graph_ref()[output].node);
            }

            NodeResponse::DeleteNodeFull { node_id, .. } => {
                // slotmap may pre destroy this
                self.output_targets.remove(node_id);
                self.shaders.remove(node_id);
            }
            _ => {}
        }
    }

    fn render_shaders(&mut self) {
        // let shaders = &mut self.shaders;
        let graph = &self.graph;

        for (output_id, output_target) in &mut self.output_targets {
            let mut rendered_nodes = vec![];

            output_target.with_fb_mut(|fb| {
                fb.clear_color(0., 0., 0., 0.);

                let _rendered_output = graph.map_with_inputs(output_id, &mut |node_id, inputs| {
                    let texture = &mut self.textures[node_id];

                    if rendered_nodes.contains(&node_id) {
                        return self.textures[node_id].tex_for_sampling();
                    }

                    let named_inputs: ComputedInputs = inputs
                        .iter()
                        .filter_map(|(name, input, texture)| {
                            match input.value {
                                NodeValueTypes::Float(f) => Some(ComputedNodeInput::Float(f)),
                                NodeValueTypes::Vec2(v) => Some(ComputedNodeInput::Vec2(v)),
                                NodeValueTypes::Bool(v) => Some(ComputedNodeInput::Bool(v)),
                                NodeValueTypes::Vec4(v) => Some(ComputedNodeInput::Vec4(v)),
                                NodeValueTypes::Color(v) => Some(ComputedNodeInput::Vec4(v.to_array())),

                                //compute elements that don't have values
                                NodeValueTypes::None => match input.typ {
                                    NodeConnectionTypes::Texture2D => texture
                                        .as_ref()
                                        .map(|texture| ComputedNodeInput::Texture(texture.clone())),
                                    _ => None,
                                },
                            }
                            .map(|computed_input| (name.as_str(), computed_input))
                        })
                        .collect();

                    // let shader = &mut self.shaders[node_id];
                    if let Some(shader) = self.shaders.get(node_id) {
                        texture.draw(|surface| {
                            shader.draw(surface, &named_inputs);
                        });
                    }

                    rendered_nodes.push(node_id);

                    texture.tex_for_sampling()
                });
            });

            let _rendered_node_names: String = rendered_nodes
                .iter()
                .map(|node_id| self.graph[*node_id].label.clone())
                .intersperse(", ".to_string())
                .collect();

            // println!("RENDERED {rendered_node_names} to {}", self.graph[output_id].label);
        }
    }

    pub fn reload_ifs_shaders(&mut self, facade: &impl Facade) {
        for (node_id, version) in self.versions.iter_mut() {
            let template = &mut self.graph[node_id].user_data.template;

            if let NodeTypes::Isf { file, isf: _ } = template {
                let new_version = file.path.metadata().unwrap().modified().unwrap();

                if *version < new_version {
                    //iterate version even on error (wait for change to retry)
                    *version = new_version;

                    let name = file.name.clone();

                    match reload_ifs_shader(facade, file.clone()) {
                        Ok((new_template, new_shader)) => {
                            self.shaders.insert(node_id, new_shader);
                            *template = new_template;
                            println!("Reloaded shader: {}", name);
                        }
                        Err(err) => {
                            eprintln!("Error reloading shader {}: {:?}", file.name, err);
                        }
                    }
                }
            }
        }
    }

    pub fn draw(&mut self, display: &Display, egui_glium: &mut EguiGlium) {
        let mut frame = display.draw();

        frame.clear_color_and_depth((1., 1., 1., 1.), 0.);

        let mut graph_response = None;

        let _needs_repaint = egui_glium.run(display, |ctx| {
            graph_response = Some(self.graph.draw(ctx));
        });

        if let Some(response) = graph_response {
            for event in response.node_responses {
                self.node_event(display, egui_glium, event);
            }
        }

        self.render_shaders();

        egui_glium.paint(display, &mut frame);

        if let Some((_, first_output)) = self.output_targets.iter_mut().next() {
            let fb = first_output.borrow_fb();

            // fb.fill(&frame, )
            // let source_rec = Rect{
            //     width: 1,
            //     height: 1,
            //     ..Default::default()
            // };
            // let target_rect = Rect {

            // }
            // let source_rect = Rect::(Pos2::ZERO, [1.,1.].into());
            // let target_rect = Rect::from_two_pos([0.5, 0.5].into(), [1., 1.].into());
            let filter = glium::uniforms::MagnifySamplerFilter::Linear;
            frame.fill(fb, filter);
            // frame.blit_buffers_from_simple_framebuffer(&fb, &source_rect, &target_rect, filter, BlitMask::color());
        }

        frame.finish().unwrap();
    }
}

#[derive(Debug)]
enum IsfShaderLoadError {
    IoError(std::io::Error),
    ParseError(isf::ParseError),
    CompileError(glium::program::ProgramCreationError),
}

impl From<std::io::Error> for IsfShaderLoadError {
    fn from(err: std::io::Error) -> Self {
        IsfShaderLoadError::IoError(err)
    }
}

impl From<glium::program::ProgramCreationError> for IsfShaderLoadError {
    fn from(err: glium::program::ProgramCreationError) -> Self {
        IsfShaderLoadError::CompileError(err)
    }
}

impl From<isf::ParseError> for IsfShaderLoadError {
    fn from(err: isf::ParseError) -> Self {
        IsfShaderLoadError::ParseError(err)
    }
}

fn reload_ifs_shader(
    facade: &impl Facade,
    file: IsfPathInfo,
) -> Result<(NodeTypes, NodeShader), IsfShaderLoadError> {
    let new_template = NodeTypes::Isf {
        isf: isf::parse(&read_to_string(&file.path).unwrap())?,
        file,
    };
    let new_shader = NodeShader::new(&new_template, facade).unwrap()?;

    Ok((new_template, new_shader))
}
