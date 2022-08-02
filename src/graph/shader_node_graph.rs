use egui_glium::EguiGlium;
use egui_node_graph::{NodeId, GraphEditorState, NodeResponse};
use glium::{Frame, backend::Facade, Display, Surface, framebuffer::{SimpleFrameBuffer, RenderBuffer}};
use glium_utils::util::DEFAULT_TEXTURE_FORMAT;
use ouroboros::self_referencing;
use slotmap::{SecondaryMap, SparseSecondaryMap};
use itertools::Itertools;

use super::{def::{*, self}, trait_impl::AllNodeTypes, shader_manager::{NodeShader}};

type EditorState = GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;


#[self_referencing]
pub struct OutputTarget{
    rb: RenderBuffer,

    #[borrows(rb)]
    #[covariant]
    fb: SimpleFrameBuffer<'this>
}

pub struct ShaderNodeGraph
{
    pub graph_state: EditorState,
    output_targets: SparseSecondaryMap<NodeId, OutputTarget>,
    shaders: SecondaryMap<NodeId, NodeShader>
}


impl ShaderNodeGraph {
    pub fn new() -> Self {
        Self { 
            graph_state: GraphEditorState::new(1.0, GraphState::default()),
            output_targets: SparseSecondaryMap::new(),
            shaders: SecondaryMap::new()
        }
    }

    pub fn node_event(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium, event: NodeResponse<def::GraphResponse, NodeData>) {
        match event {
            egui_node_graph::NodeResponse::CreatedNode(node_id) => {
                let node = &mut self.graph_state.graph[node_id];
                
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

    fn render_node_and_inputs(&mut self, surface: & SimpleFrameBuffer<'_>, node_id: NodeId, rendered: &mut Vec<NodeId>) {
        //skip if rendered by another path
        if rendered.contains(&node_id){
            return;
        }

        //depth-first
        for (_, input_id) in &self.graph_state.graph[node_id].inputs {
            if let Some(output_id) = self.graph_state.graph.connection(*input_id){
                let next_node_id = self.graph_state.graph[output_id].node;
                self.render_node_and_inputs(surface, next_node_id, rendered)
            }
        }

        //render after previous preceeding nodes
        let shader_data = &mut self.shaders[node_id];
        shader_data.render();
        rendered.push(node_id);
    }

    fn render_shaders(&mut self, facade: &impl Facade){
        let mut rendered_nodes = vec![];

        for (output_id, output_target) in &self.output_targets {
            // let node = self.state.graph[*output_id];
            // let mut temp_surface = SimpleFrameBuffer::new(facade, output_target).unwrap();
            self.render_node_and_inputs(output_target.borrow_fb(), output_id, &mut rendered_nodes)
            // output_target.with_fb_mut(|fb| {
            //     self.render_node_and_inputs(fb, output_id, &mut rendered_nodes);
            // })
        }

        let rendered_node_names: String = rendered_nodes.iter()
            .map(|node_id| self.graph_state.graph[*node_id].label.clone())
            .intersperse(", ".to_string())
            .collect();

        println!("FINISHED render_shaders: {rendered_node_names}");
    }

    pub fn draw(&mut self, display: &Display, egui_glium: &mut EguiGlium){
        let mut frame = display.draw();
        
        frame.clear_color_and_depth((1.,1.,1.,1.), 0.);

        let mut graph_response = None;

        let _needs_repaint = egui_glium.run(display, |ctx| {
            graph_response = Some(self.draw_egui(ctx));
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

    fn draw_egui(&mut self, ctx: &egui::Context) -> egui_node_graph::GraphResponse<GraphResponse, NodeData> {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("Hello World!");
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let graph_resp = self.graph_state.draw_graph_editor(ui, AllNodeTypes);

            let output = self.output_targets.iter().next()
                .map(|(output_node_id, _)| self.shaders.get(output_node_id))
                .flatten();

            if let Some(cache) = output {
                ui.image(cache.clone_tex_id(), [512., 512.]);
            }

            graph_resp
        }).inner
    }
}

// impl eframe::App for ShaderNodeGraph {
//     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//         self.draw(ctx);
//     }
// }