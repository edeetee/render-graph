use std::{ops::Index, process::Output};

use eframe::glow::Shader;
use egui_glium::EguiGlium;
use egui_node_graph::{NodeId, GraphEditorState, NodeResponse, Node, OutputId};
use glium::{Frame, backend::Facade, Display, Surface, framebuffer::{SimpleFrameBuffer, RenderBuffer}, BlitMask, Rect};
use glium_utils::util::DEFAULT_TEXTURE_FORMAT;
use ouroboros::self_referencing;
use slotmap::{SecondaryMap, SparseSecondaryMap};
use itertools::Itertools;

use super::{def::{*, self}, trait_impl::AllNodeTypes, node_shader::{NodeShader}, shader_graph::ShaderGraph};

pub(crate) type EditorState = GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;

#[self_referencing]
pub struct OutputTarget{
    rb: RenderBuffer,

    #[borrows(rb)]
    #[covariant]
    fb: SimpleFrameBuffer<'this>
}

#[derive(Default)]
pub struct ShaderGraphRenderer
{
    graph: ShaderGraph,
    output_targets: SparseSecondaryMap<NodeId, OutputTarget>,
    shaders: SecondaryMap<NodeId, NodeShader>
}

impl ShaderGraphRenderer {
    pub fn new() -> Self {
        Default::default()
    }

    fn add_dangling_output(&mut self, facade: &impl Facade, node_id: NodeId){
        let node = &self.graph.graph_ref()[node_id];
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

                let new_shader = NodeShader::new(facade, egui_glium, self.graph[node_id].user_data.template);
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
            },

            //may create new output target
            NodeResponse::DisconnectEvent { output, input } => {
                self.add_dangling_output(facade, self.graph.graph_ref().outputs[output].node);
            },

            NodeResponse::ConnectEventEnded { output, input } => {
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

    fn render_shaders(&mut self, facade: &impl Facade){
        let shaders = &mut self.shaders;
        let graph = &self.graph;

        for (output_id, output_target) in &mut self.output_targets {
            let mut rendered_nodes = vec![];

            output_target.with_fb_mut(|fb| {
                fb.clear_color(0., 0., 0., 0.);
            });

            output_target.with_fb_mut(|surface| {
                let _rendered_output = graph.map_to(output_id, 
                    &mut |node_id, _| {
                        if rendered_nodes.contains(&node_id){
                            return;
                        }
                        
                        let shader_data = &mut shaders[node_id];
                        shader_data.render(surface);
                        rendered_nodes.push(node_id);
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

        self.render_shaders(display);

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