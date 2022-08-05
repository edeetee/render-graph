use std::ops::Index;

use eframe::glow::Shader;
use egui_glium::EguiGlium;
use egui_node_graph::{NodeId, GraphEditorState, NodeResponse, Node};
use glium::{Frame, backend::Facade, Display, Surface, framebuffer::{SimpleFrameBuffer, RenderBuffer}};
use glium_utils::util::DEFAULT_TEXTURE_FORMAT;
use ouroboros::self_referencing;
use slotmap::{SecondaryMap, SparseSecondaryMap};
use itertools::Itertools;

use super::{def::{*, self}, trait_impl::AllNodeTypes, shader_manager::{NodeShader}, shader_graph::ShaderGraph};

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

    pub fn node_event(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium, event: NodeResponse<def::GraphResponse, NodeData>) {
        match event {
            egui_node_graph::NodeResponse::CreatedNode(node_id) => {
                let node = &mut self.graph[node_id];
                
                let new_shader = NodeShader::new(facade, egui_glium, node.user_data.template);
                node.user_data.result = Some(new_shader.clone_tex_id());
                self.shaders.insert(node_id, new_shader);

                if node.user_data.template == NodeTypes::Output {
                    let output_target = OutputTargetBuilder {
                        rb: RenderBuffer::new(facade, DEFAULT_TEXTURE_FORMAT, 512, 512).unwrap(),
                        fb_builder: |rb| SimpleFrameBuffer::new(facade, rb).unwrap()
                    }.build();
                    self.output_targets.insert(node_id, output_target);
                    // self.output_targets.push(node_id)
                }
            },

            NodeResponse::DeleteNodeFull { node_id, .. } => {
                // if let Some(output_index) = self.output_targets.iter().position(|id| *id == node_id){
                //     self.output_targets.remove(output_index);
                // }

                // slotmap may pre destroy this
                self.output_targets.remove(node_id);
                self.shaders.remove(node_id);
            }
            _ => {}
        }
    }

    // fn render_node_and_inputs(
    //     &self, 
    //     surface: & SimpleFrameBuffer<'_>, 
    //     node_id: NodeId, 
    //     // shaders: &mut SecondaryMap<NodeId, NodeShader>, 
    //     // rendered: &mut Vec<NodeId>
    // ) {
    //     //skip if rendered by another path
    //     if rendered.contains(&node_id){
    //         return;
    //     }
    
    //     //depth-first
    //     for (_, input_id) in &self.graph_state.graph[node_id].inputs {
    //         if let Some(output_id) = self.graph_state.graph.connection(*input_id){
    //             let next_node_id = self.graph_state.graph[output_id].node;
    //             self.render_node_and_inputs(surface, next_node_id, shaders, rendered)
    //         }
    //     }
    
    //     //render after previous preceeding nodes
    //     let shader_data = &mut shaders[node_id];
    //     shader_data.render();
    //     rendered.push(node_id);
    // }

    fn render_shaders(&mut self, facade: &impl Facade){
        let mut rendered_nodes = vec![];
        let mut shaders = &mut self.shaders;
        let graph = &self.graph;

        for (output_id, output_target) in &self.output_targets {
            
            // let mut temp_surface = SimpleFrameBuffer::new(facade, output_target).unwrap();

            let _rendered_output = graph.map(output_id, 
                &mut |node_id, _| {
                    if rendered_nodes.contains(&node_id){
                        return;
                    }
                    
                    let shader_data = &mut shaders[node_id];
                    shader_data.render(output_target.borrow_fb());
                    rendered_nodes.push(node_id);
                }
            );

        }

        let rendered_node_names: String = rendered_nodes.iter()
            .map(|node_id| self.graph[*node_id].label.clone())
            .intersperse(", ".to_string())
            .collect();

        println!("FINISHED render_shaders: {rendered_node_names}");
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
        
        // frame.clear_color_and_depth((1.,1.,1.,1.), 0.);

        egui_glium.paint(display, &mut frame);

        frame.finish().unwrap();
    }
}

// impl eframe::App for ShaderNodeGraph {
//     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//         self.draw(ctx);
//     }
// }