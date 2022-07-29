
use std::rc::Rc;

use egui::{DragValue, TextureId};
use egui_glium::EguiGlium;
use egui_node_graph::{Graph, NodeDataTrait, NodeId, WidgetValueTrait, GraphEditorState, NodeResponse, Node};
use glium::{Frame, texture::SrgbTexture2d, framebuffer::SimpleFrameBuffer, backend::Facade, Display, glutin::event::WindowEvent, Surface};
use glium_utils::modular_shader::{modular_shader::ModularShader, instances::InstancesView, sdf::SdfView};
use slotmap::{SecondaryMap, SparseSecondaryMap};

use super::{def::{*, self}, trait_impl::AllNodeTypes};

impl NodeDataTrait for NodeData {
    type Response = GraphResponse;
    type UserState = GraphState;
    type DataType = NodeConnectionTypes;
    type ValueType = NodeValueTypes;

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _state: &Self::UserState,
    ) -> Vec<egui_node_graph::NodeResponse<Self::Response, Self>>
    where
        Self::Response: egui_node_graph::UserResponseTrait,
    {
        let me = &graph[node_id];

        if let Some(tex_id) = &me.user_data.result {
            ui.image(*tex_id, ui.available_size());
        } else {
            ui.label("NO IMAGE AVAILABLE");
        }
        
        vec![]
    }
}

impl WidgetValueTrait for NodeValueTypes {
    type Response = GraphResponse;

    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<Self::Response> {
        match self {
            NodeValueTypes::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut value[0]));
                    ui.label("y");
                    ui.add(DragValue::new(&mut value[1]));
                });
            }
            NodeValueTypes::Float { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
            NodeValueTypes::None => {}
        }

        Vec::new()
    }
}

// -------------------------------------------------------------------


type EditorState = GraphEditorState<NodeData, NodeConnectionTypes, NodeValueTypes, NodeTypes, GraphState>;

pub struct ShaderNodeGraph<'a>
{
    pub state: EditorState,
    output_nodes: Vec<NodeId>,
    shaders: SecondaryMap<NodeId, ShaderData<'a>>,
    pub egui_glium: EguiGlium,
    display: &'a Display
}

impl<'a> ShaderNodeGraph<'a> {
    pub fn new(display: &'a Display) -> Self {
        Self { 
            state: GraphEditorState::new(1.0, GraphState::default()),
            output_nodes: Vec::new(),
            egui_glium: egui_glium::EguiGlium::new(&display),
            shaders: SecondaryMap::new(),
            display
        }
    }

    pub fn node_event(&mut self, event: NodeResponse<def::GraphResponse, NodeData>) {
        match event {
            egui_node_graph::NodeResponse::CreatedNode(node_id) => {
                let node = &mut self.state.graph[node_id];
                
                let new_shader = ShaderData::new(self.display, &mut self.egui_glium, node.user_data.template);
                node.user_data.result = Some(new_shader.tex_id);
                self.shaders[node_id] = new_shader;

                if node.user_data.template == NodeTypes::Output {
                    self.output_nodes.push(node_id)
                }
            },

            NodeResponse::DeleteNodeFull { node_id, node } => {
                if let Some(output_index) = self.output_nodes.iter().position(|id| *id == node_id){
                    self.output_nodes.swap_remove(output_index);
                }

                self.shaders.remove(node_id);
            }
            _ => {}
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent) -> bool {
        self.egui_glium.on_event(event)
    }

    fn render_node_and_inputs(&self, frame: &mut Frame, node_id: NodeId) {
        let shader_data = &self.shaders[node_id];

        if let Some(shader) = shader_data.modular_shader.as_ref() {
            shader.draw_to(frame);
        }
    }

    pub fn render_shaders(&mut self){
        for output_id in &self.output_nodes {
            // let node = self.state.graph[*output_id];
            let mut frame = self.display.draw();
            self.render_node_and_inputs(&mut frame, *output_id);
            frame.finish().unwrap();
        }
    }

    pub fn draw(&mut self){
        let mut frame = self.display.draw();

        // let egui_glium = &mut self.egui_glium;

        let _needs_repaint = self.egui_glium.run(&self.display, |ctx| {
            let graph_response = self.draw_egui(ctx);

            for event in graph_response.node_responses{
                self.node_event(event);
            }
        });

        self.render_shaders();

        self.egui_glium.paint(self.display, &mut frame);

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
            let graph_resp = self.state.draw_graph_editor(ui, AllNodeTypes);

            let output = self.output_nodes.first()
                .map(|output_node_id| self.shaders.get(*output_node_id))
                .flatten();

            if let Some(cache) = output {
                ui.image(cache.tex_id, [512., 512.]);
            }

            graph_resp
        }).inner
    }

    pub fn draw_default_output(&self, frame: Frame){
        // self.state.
    }
}

// impl eframe::App for ShaderNodeGraph {
//     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//         self.draw(ctx);
//     }
// }

pub struct ShaderData<'a> {
    tex: SrgbTexture2d,
    pub tex_fb: SimpleFrameBuffer<'a>,
    pub tex_id: TextureId,
    modular_shader: Option<Box<dyn ModularShader<Frame>>>
}

const DEFAULT_RES: [u32; 2] = [1920, 1080];

impl ShaderData<'_> {
    pub fn new<F: Facade>(facade: &F, egui_glium: &mut EguiGlium, template: NodeTypes) -> Self {
        let output_texture = SrgbTexture2d::empty_with_format(
            facade, 
            glium::texture::SrgbFormat::U8U8U8U8, 
            glium::texture::MipmapsOption::NoMipmap, 
            DEFAULT_RES[0].into(), 
            DEFAULT_RES[1].into()
        ).unwrap();
    
        let output_texture_rc = Rc::new(output_texture);
        let output_frame_buffer = SimpleFrameBuffer::new(facade, output_texture_rc.clone()).unwrap();
        let output_texture_egui = egui_glium.painter.register_native_texture(output_texture_rc);

        let modular_shader: Option<Box<dyn ModularShader<_>>> = match template {
            // NodeTypes::Instances => InstancesView::new(facade),
            NodeTypes::Sdf => Some(Box::new(SdfView::new(facade))),
            _ => None
        };

        Self {
            tex: output_texture,
            tex_fb: output_frame_buffer,
            tex_id: output_texture_egui,
            modular_shader
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if let Some(shader) = self.modular_shader {
            shader.draw_to(frame);
        }
    }
}

