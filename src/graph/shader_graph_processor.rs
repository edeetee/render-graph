use egui_glium::EguiGlium;
use egui_node_graph::{NodeId, GraphEditorState, NodeResponse};
use glium::{backend::Facade, Display, Surface, framebuffer::{SimpleFrameBuffer, RenderBuffer}};
use glium_utils::util::DEFAULT_TEXTURE_FORMAT;
use ouroboros::self_referencing;
use slotmap::{SecondaryMap, SparseSecondaryMap};
use itertools::Itertools;

use super::{def::{*, self}, node_shader::{NodeShader}, shader_graph::ShaderGraph};

pub(crate) type EditorState = GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;

#[self_referencing]
pub struct OutputTarget{
    rb: RenderBuffer,

    #[borrows(rb)]
    #[covariant]
    fb: SimpleFrameBuffer<'this>
}

#[derive(Default)]
pub struct ShaderGraphProcessor
{
    graph: ShaderGraph,
    output_targets: SparseSecondaryMap<NodeId, OutputTarget>,
    shaders: SecondaryMap<NodeId, NodeShader>
}

impl ShaderGraphProcessor {
    pub fn new() -> Self {
        Default::default()
    }

    fn add_dangling_output(&mut self, facade: &impl Facade, node_id: NodeId){
        // let is_output_target = node.outputs(&self.graph.graph_ref()).any(|o| o.typ == NodeConnectionTypes::Texture2D);

        let output_target = OutputTargetBuilder {
            rb: RenderBuffer::new(facade, DEFAULT_TEXTURE_FORMAT, 512, 512).unwrap(),
            fb_builder: |rb| SimpleFrameBuffer::new(facade, rb).unwrap()
        }.build();
        self.output_targets.insert(node_id, output_target);
    }

    pub fn node_event(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium, event: NodeResponse<def::GraphResponse, NodeData>) {
        match event {
            egui_node_graph::NodeResponse::CreatedNode(node_id) => {

                if let Some(new_shader) = NodeShader::new(facade, egui_glium, self.graph[node_id].user_data.template){
                    // new_shader.init_inputs(self.graph[node_id].input_ids().map(|input_id| &mut self.graph.[input_id]));

                    self.graph[node_id].user_data.result = Some(new_shader.clone_tex_id());
    
                    self.shaders.insert(node_id, new_shader);
    
                    let node = &self.graph[node_id];
    
                    for input in node.inputs(self.graph.graph_ref()) {
                        if input.typ == NodeConnectionTypes::Texture2D{
                            let connected_output = self.graph.graph_ref().connection(input.id);
                            if let Some(output_id) = connected_output {
                                let connected_node_id = self.graph.graph_ref().outputs[output_id].node;
    
                                self.output_targets.remove(connected_node_id);
                            }
                        }
                    }
    
                    self.add_dangling_output(facade, node_id);
                }
            },

            //may create new output target
            NodeResponse::DisconnectEvent { output, .. } => {
                self.add_dangling_output(facade, self.graph.graph_ref().outputs[output].node);
            },

            NodeResponse::ConnectEventEnded { output, .. } => {
                self.output_targets.remove(self.graph.graph_ref()[output].node);
            },

            NodeResponse::DeleteNodeFull { node_id, .. } => {
                // slotmap may pre destroy this
                self.output_targets.remove(node_id);
                self.shaders.remove(node_id);
            }
            _ => {}
        }
    }

    fn render_shaders(&mut self){
        let shaders = &mut self.shaders;
        let graph = &self.graph;

        for (output_id, output_target) in &mut self.output_targets {
            let mut rendered_nodes = vec![];

            output_target.with_fb_mut(|fb| {
                fb.clear_color(0., 0., 0., 0.);
            });

            output_target.with_fb_mut(|surface| {

                let _rendered_output = graph.map_to(output_id, 
                    &mut |node_id, inputs| {
                        if rendered_nodes.contains(&node_id){
                            return shaders[node_id].tex_rc();
                        }
                        
                        let named_inputs = graph[node_id].inputs.iter()
                            .filter_map(|(name, id)|{
                                let input = &self.graph.graph_ref()[*id];

                                 match &input.value {
                                    NodeValueTypes::Float(f) => Some(ComputedNodeInput::Float(f)),
                                    NodeValueTypes::Vec2(v) => Some(ComputedNodeInput::Vec2(v)),
                                    NodeValueTypes::Bool(v) => Some(ComputedNodeInput::Bool(v)),
                                    _ => {
                                        match input.typ {
                                            NodeConnectionTypes::Texture2D => {
                                                let output_id = graph.graph_ref().connection(input.id)?;
                                                let connected_node = graph.graph_ref().outputs[output_id].node;

                                                let input_match = inputs.iter()
                                                    .find(|input| input.0 == connected_node)?;

                                                Some(ComputedNodeInput::Texture(input_match.1.clone()))
                                            },
                                            _ => None
                                        }
                                    }
                                }.map(|computed_input| (name.as_str(), computed_input))
                            });
                        
                        // shader_data.update(named_inputs);
                        
                        let shader_data = &mut shaders[node_id];
                        
                        shader_data.render(surface, named_inputs);
                        rendered_nodes.push(node_id);

                        shader_data.tex_rc()
                    }
                );

            });

            let rendered_node_names: String = rendered_nodes.iter()
                .map(|node_id| self.graph[*node_id].label.clone())
                .intersperse(", ".to_string())
                .collect();

            println!("RENDERED {rendered_node_names} to {}", self.graph[output_id].label);
        }

    }

    pub fn draw(&mut self, display: &Display, egui_glium: &mut EguiGlium){
        let mut frame = display.draw();
        
        frame.clear_color_and_depth((1.,1.,1.,1.), 0.);

        let mut graph_response = None;

        let _needs_repaint = egui_glium.run(display, |ctx| {
            graph_response = Some(self.graph.draw(ctx));
        });

        if let Some(response) = graph_response {
            for event in response.node_responses{
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