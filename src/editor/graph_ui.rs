

use egui::{RichText, Widget, Color32};
use egui_glium::EguiGlium;
use egui_node_graph::{NodeDataTrait, NodeId, NodeResponse, NodeTemplateTrait, UserResponseTrait};
use glium::{Texture2d};
use glium::{backend::Facade, Display, Surface};
use crate::util::MappableTuple;
use serde::{Serialize, Deserialize};

// use crate::textures::UiTexture;

use crate::common::persistent_state::{PersistentState, WindowState};
use crate::graph::{
    def::{GraphEditorState, GraphResponse, GraphState, UiNodeData},
    node_types::{AllNodeTypes, NodeType},
    GraphChangeEvent, ShaderGraphProcessor,
};

use super::node_textures::{NodeUiTextures};
use super::node_tree_ui::TreeState;

pub struct GraphUi {
    processor: ShaderGraphProcessor,
    editor: GraphEditorState,
    graph_state: GraphState,
    tree: TreeState,
    node_textures: NodeUiTextures,
    extra_graphui_state: GraphUiState
}

#[derive(Default, Serialize, Deserialize)]
pub struct GraphUiState {
    view_state: ViewState,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum ViewState{
    Graph,
    Output
}

impl ViewState {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Graph => Self::Output,
            Self::Output => Self::Graph,
        }
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::Graph
    }
}

impl Default for GraphUi {
    fn default() -> Self {
        Self {
            editor: GraphEditorState::new(1.0),
            graph_state: GraphState::default(),
            tree: TreeState::default(),
            processor: ShaderGraphProcessor::default(),
            node_textures: NodeUiTextures::default(),
            extra_graphui_state: GraphUiState::default()
        }
    }
}

impl GraphChangeEvent
{
    fn from_response<UserResponse: UserResponseTrait, NodeData: NodeDataTrait>(response: &NodeResponse<UserResponse,NodeData>) -> Option<Self> {
        match response {
            NodeResponse::ConnectEventEnded { output, input } => Some(GraphChangeEvent::Connected {
                output_id: *output,
                input_id: *input,
            }),
            NodeResponse::CreatedNode(node_id) => Some(GraphChangeEvent::CreatedNode(*node_id)),
            NodeResponse::DeleteNodeFull { node_id, node: _ } => {
                Some(GraphChangeEvent::DestroyedNode(*node_id))
            }
            NodeResponse::DisconnectEvent { output, input } => Some(GraphChangeEvent::Disconnected {
                output_id: *output,
                input_id: *input,
            }),
            _ => None,
        }
    }
}

impl GraphUi {
    pub fn new_from_persistent(
        mut state: PersistentState,
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
    ) -> Self {
        Self {
            processor: ShaderGraphProcessor::new_from_graph(&mut state.editor.graph, facade),
            node_textures: NodeUiTextures::new_from_graph(&mut state.editor.graph, facade, egui_glium),
            editor: state.editor,
            graph_state: state.state,
            extra_graphui_state: state.graph_ui_state.unwrap_or_default(),
            ..Default::default()
        }
    }

    pub fn to_persistent(self, extras: Option<WindowState>) -> PersistentState {
        PersistentState {
            editor: self.editor,
            state: self.graph_state,
            window: extras,
            graph_ui_state: Some(self.extra_graphui_state)
        }
    }

    pub fn editor(&self) -> &GraphEditorState {
        &self.editor
    }

    delegate::delegate! {
         to self.processor {
             pub fn update(&mut self, [&mut self.editor.graph], [&self.graph_state], facade: &impl Facade);
         }
    }

    // fn 

    pub fn process_frame(&mut self, display: &Display, egui_glium: &mut EguiGlium) {
        let mut frame = display.draw();

        // toggling the view state
        if egui_glium.egui_ctx.input().key_pressed(egui::Key::F1) {
            self.extra_graphui_state.view_state.toggle();
        }

        const mono_color: f32 = 0.1;
        frame.clear_color_and_depth((mono_color, mono_color, mono_color, 1.), 0.);

        match self.extra_graphui_state.view_state {
            ViewState::Graph => {
                let mut graph_response = None;

                //prepare egui draw
                let _needs_repaint = egui_glium.run(display, |ctx| {
                    graph_response = Some(self.draw(ctx));
                });

                //Update data that stays aligned with the graph
                if let Some(response) = graph_response {
                    for response in response.node_responses {
                        if let Some(event) = GraphChangeEvent::from_response(&response) {
                            self.processor
                                .graph_event(&mut self.editor.graph, display, event);
                        }

                        match response {
                            NodeResponse::CreatedNode(node_id) => {
                                let node = self.editor.graph.nodes.get_mut(node_id).unwrap();
                                self.node_textures.add(display, egui_glium, node);
                            },
                            NodeResponse::DeleteNodeFull { node_id, .. } => {
                                self.node_textures.remove(node_id);
                            }
                
                            _ => {}
                        }
                    }
                }
            }
            ViewState::Output => {
                //prepare egui draw
                let _needs_repaint = egui_glium.run(display, |ctx| {
                    egui::Window::new("Output").show(ctx, |ui| {
                        ui.label(RichText::new("Press F1 to toggle between graph and output view").color(Color32::WHITE));
                    });
                });
            }
        }

        let outputs = self.processor.render_shaders(&mut self.editor.graph, display,
            |node_id, tex: &Texture2d| {
            // frame.
            let surface = tex.as_surface();
            // surface.fill(&frame, glium::uniforms::MagnifySamplerFilter::Nearest);
            self.node_textures.copy_surface(display, egui_glium, node_id, &surface);
        });

        match self.extra_graphui_state.view_state {
            ViewState::Graph => {
                egui_glium.paint(display, &mut frame);
            },
            ViewState::Output => {
                //for some reason required to make the frame correctly map onto the output
                egui_glium.paint(display, &mut frame);
                
                if let Some(output) = outputs.first().cloned().flatten() {
                    // println!("OUTPUT");
                    let filter = glium::uniforms::MagnifySamplerFilter::Nearest;
                    let dimens = display.get_framebuffer_dimensions();
                    // let ppp = display.gl_window().window().scale_factor();
                    // println!("${dimens:?}");

                    // let src_dimens = dimens.map(|x| ((*x as f64)/ppp));
                    let _dst_dimens = dimens.map(|x| ((*x as f64)));
                    let _src_dimens = output.dimensions();

                    // logic
                    frame.clear_all((0.0,0.0,0.0,1.0), 0.0, 0);
                    // output.as_surface().blit_color(
                    //     &Rect{left: 0, bottom: 0, width: src_dimens.0 as u32, height: src_dimens.1 as u32}, 
                    //     &frame, 
                    //     &glium::BlitTarget { left: 0, bottom: 0, width: dst_dimens.0 as i32, height: dst_dimens.1 as i32}, 
                    //     filter
                    // );
                    output.as_surface().fill(&mut frame, filter);
                }
            }
        }

        frame.finish().unwrap();
    }

    pub fn add_node(&mut self, node_kind: &NodeType, position: egui::Pos2) -> NodeId {
        // println!("Adding node {node_kind:#?}");

        let new_node = self.editor.graph.add_node(
            node_kind.node_graph_label(&mut self.graph_state),
            node_kind.user_data(&mut self.graph_state),
            |graph, node_id| node_kind.build_node(graph, &mut self.graph_state, node_id),
        );
        self.editor.node_positions.insert(new_node, position);
        self.editor.node_order.push(new_node);

        new_node
    }

    pub fn draw(
        &mut self,
        ctx: &egui::Context,
    ) -> egui_node_graph::GraphResponse<GraphResponse, UiNodeData> {
        let mut new_node_ty = None;

        // ctx

        egui::SidePanel::left("tree_view").show(ctx, |ui| {
            if let Some(selected_item) = self.tree.draw(ui) {
                new_node_ty = Some(selected_item.ty.clone());
            }
        });

        if !self.graph_state.animations.is_empty() {
            egui::SidePanel::left("animators").show(ctx, |ui| {
                let mut removal = None;
                for (key, updater) in &mut self.graph_state.animations {
                    let (node_id, param_name) = key;

                    let node = &self.editor.graph.nodes[*node_id];

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{}.{}", node.label, param_name)));
                            if ui.button("REMOVE").clicked() {
                                removal = Some(key.clone());
                            }
                        });
                        updater.ui(ui);
                    });
                }

                if let Some(removal) = removal {
                    self.graph_state.animations.remove(&removal);
                }
            });
        }

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.set_clip_rect(ctx.available_rect());
                egui::widgets::global_dark_light_mode_switch(ui);

                if ctx.input().key_pressed(egui::Key::H) {
                    self.editor.pan_zoom.pan = egui::Vec2::ZERO;
                }

                let mut responses = vec![];

                let editor_rect = ui.max_rect();

                if let Some(node_ty) = new_node_ty {
                    let pos = editor_rect.left_top() - self.editor.pan_zoom.pan;
                    let new_node_id = self.add_node(&node_ty, pos);
                    responses.push(egui_node_graph::NodeResponse::CreatedNode(new_node_id));
                }

                if ui.ui_contains_pointer() {
                    self.editor.pan_zoom.pan += ctx.input().scroll_delta;

                    if let Some(point) = ctx.input().pointer.hover_pos() {
                        let zoom_delta = ctx.input().zoom_delta();
                        self.editor
                            .pan_zoom
                            .adjust_zoom(zoom_delta, point.to_vec2(), 0.001, 100.0);
                    }
                    // self.0.pan_zoom.zoom *= ctx.input().zoom_delta();
                    // dbg!(self.0.pan_zoom.zoom);
                }

                let mut graph_resp =
                    self.editor
                        .draw_graph_editor(ui, AllNodeTypes, &mut self.graph_state);

                self.editor.node_finder = None;
                graph_resp.node_responses.append(&mut responses);

                graph_resp
            })
            .inner
    }
}
