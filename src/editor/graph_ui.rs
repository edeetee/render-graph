use std::ops::Deref;

use crate::util::MappableTuple;
use egui::{Color32, RichText, Widget};
use egui_glium::EguiGlium;
use egui_node_graph::{
    AnyParameterId, NodeDataTrait, NodeId, NodeResponse, NodeTemplateTrait, UserResponseTrait,
};
use glium::Texture2d;
use glium::{backend::Facade, Display, Surface};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

// use crate::textures::UiTexture;

use crate::common::persistent_state::{PersistentState, WindowState};
use crate::graph::{
    def::{GraphEditorState, GraphResponse, GraphState, UiNodeData},
    node_types::{AllNodeTypes, NodeType},
    GraphChangeEvent, ShaderGraphProcessor,
};

use super::node_textures::NodeUiTextures;
use super::node_tree_ui::TreeState;

pub struct GraphUi {
    processor: ShaderGraphProcessor,
    editor: GraphEditorState,
    graph_state: GraphState,
    tree: TreeState,
    node_textures: NodeUiTextures,
    state: GraphUiState,
}

#[derive(Serialize, Deserialize, Debug)]
enum NodeSelectionActor {
    Mouse(egui::Pos2),
    DraggingOutput(egui::Pos2, NodeId, AnyParameterId),
    // DraggingInput(NodeId),
}

impl NodeSelectionActor {
    fn pos(&self) -> egui::Pos2 {
        match self {
            Self::Mouse(pos) => *pos,
            Self::DraggingOutput(pos, _, _) => *pos,
        }
    }

    fn connection(&self) -> Option<(NodeId, AnyParameterId)> {
        match self {
            Self::Mouse(_) => None,
            Self::DraggingOutput(_, node_id, param_id) => Some((*node_id, *param_id)),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct GraphUiState {
    view_state: ViewState,
    node_selection_actor: Option<NodeSelectionActor>,
    last_connection_in_progress: Option<(NodeId, AnyParameterId)>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum ViewState {
    Graph,
    Output,
}

impl ViewState {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Graph => Self::Output,
            Self::Output => Self::Graph,
        }
    }
}

pub struct GraphUiResponse {
    pub graph_changes: Vec<GraphChangeEvent>,
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
            state: GraphUiState::default(),
        }
    }
}

#[derive(PartialEq, Debug)]
enum GraphUiAction {
    Home,
    ToggleAddNodeModal,
    Escape,
}

impl GraphUiAction {
    fn from_keyboard_pressed(ctx: &egui::Context) -> Option<Self> {
        if ctx.input().key_pressed(egui::Key::H) {
            Some(Self::Home)
        } else if ctx.input().key_pressed(egui::Key::Tab) {
            Some(Self::ToggleAddNodeModal)
        } else if ctx.input().key_pressed(egui::Key::Escape) {
            Some(Self::Escape)
        } else {
            None
        }
    }
}

impl GraphChangeEvent {
    fn from_response<UserResponse: UserResponseTrait, NodeData: NodeDataTrait>(
        response: &NodeResponse<UserResponse, NodeData>,
    ) -> Option<Self> {
        match response {
            NodeResponse::ConnectEventEnded { output, input } => {
                Some(GraphChangeEvent::Connected {
                    output_id: *output,
                    input_id: *input,
                })
            }
            NodeResponse::CreatedNode(node_id) => Some(GraphChangeEvent::CreatedNode(*node_id)),
            NodeResponse::DeleteNodeFull { node_id, node: _ } => {
                Some(GraphChangeEvent::DestroyedNode(*node_id))
            }
            NodeResponse::DisconnectEvent { output, input } => {
                Some(GraphChangeEvent::Disconnected {
                    output_id: *output,
                    input_id: *input,
                })
            }
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
            node_textures: NodeUiTextures::new_from_graph(
                &mut state.editor.graph,
                facade,
                egui_glium,
            ),
            editor: state.editor,
            graph_state: state.state,
            state: state.graph_ui_state.unwrap_or_default(),
            ..Default::default()
        }
    }

    pub fn to_persistent(self, extras: Option<WindowState>) -> PersistentState {
        PersistentState {
            editor: self.editor,
            state: self.graph_state,
            window: extras,
            graph_ui_state: Some(self.state),
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

    pub fn process_frame(&mut self, display: &Display, egui_glium: &mut EguiGlium) {
        let mut frame = display.draw();

        // toggling the view state
        if egui_glium.egui_ctx.input().key_pressed(egui::Key::F1) {
            self.state.view_state.toggle();
        }

        const mono_color: f32 = 0.1;
        frame.clear_color_and_depth((mono_color, mono_color, mono_color, 1.), 0.);

        match self.state.view_state {
            ViewState::Graph => {
                let mut graph_response = None;

                //prepare egui draw
                let _needs_repaint = egui_glium.run(display, |ctx| {
                    graph_response = Some(self.draw(ctx));
                });

                //Update data that stays aligned with the graph
                if let Some(response) = graph_response {

                    for change in response.graph_changes {
                        self.processor
                                .graph_event(&mut self.editor.graph, display, change);

                        match change {
                            GraphChangeEvent::CreatedNode(node_id) => {
                                let node = self.editor.graph.nodes.get_mut(node_id).unwrap();
                                self.node_textures.add(display, egui_glium, node);
                            }
                            GraphChangeEvent::DestroyedNode(node_id) => {
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
                        ui.label(
                            RichText::new("Press F1 to toggle between graph and output view")
                                .color(Color32::WHITE),
                        );
                    });
                });
            }
        }

        let outputs = self.processor.render_shaders(
            &mut self.editor.graph,
            display,
            |node_id, tex: &Texture2d| {
                // frame.
                let surface = tex.as_surface();
                // surface.fill(&frame, glium::uniforms::MagnifySamplerFilter::Nearest);
                self.node_textures
                    .copy_surface(display, egui_glium, node_id, &surface);
            },
        );

        match self.state.view_state {
            ViewState::Graph => {
                egui_glium.paint(display, &mut frame);
            }
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
                    let _dst_dimens = dimens.map(|x| (*x as f64));
                    let _src_dimens = output.dimensions();

                    // logic
                    frame.clear_all((0.0, 0.0, 0.0, 1.0), 0.0, 0);
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

    pub fn add_node(&mut self, node_kind: &NodeType, position: egui::Pos2, connection: Option<(NodeId, AnyParameterId)>) -> Vec<GraphChangeEvent> {
        let mut responses = vec![];

        let new_node = self.editor.graph.add_node(
            node_kind.node_graph_label(&mut self.graph_state),
            node_kind.user_data(&mut self.graph_state),
            |graph, node_id| node_kind.build_node(graph, &mut self.graph_state, node_id),
        );
        self.editor.node_positions.insert(new_node, position);
        self.editor.node_order.push(new_node);

        if let Some((_, AnyParameterId::Output(output_id))) = connection {
            let param = self.editor.graph.get_output(output_id);

            let matched_input_id = self.editor.graph.nodes[new_node]
                .inputs(&self.editor.graph)
                // .iter()
                .find(|input| input.typ == param.typ)
                .map(|input| input.id);

            if let Some(matched_input_id) = matched_input_id {
                self.editor.graph.add_connection(output_id, matched_input_id);
                responses.push(GraphChangeEvent::Connected { output_id: output_id, input_id: matched_input_id});
            }
        }

        responses.push(GraphChangeEvent::CreatedNode(new_node));

        responses
    }

    pub fn draw(&mut self, ctx: &egui::Context) -> GraphUiResponse {
        let action = GraphUiAction::from_keyboard_pressed(ctx);

        if let Some(action) = &action {
            dbg!(action);
        }

        if !self.graph_state.animations.is_empty() {
            self.draw_animators(ctx);
        }

        let mouse_pos: egui::Pos2 = ctx.pointer_latest_pos().unwrap_or(ctx.available_rect().left_top()) - self.editor.pan_zoom.pan;

        if action == Some(GraphUiAction::ToggleAddNodeModal) {
            self.state.node_selection_actor = if self.state.node_selection_actor.is_none() {
                Some(NodeSelectionActor::Mouse(mouse_pos))
            } else {
                None
            };
        }

        //if connection is in progress, save it
        if let Some(connection_in_progress) = self.editor.connection_in_progress {
            self.state.last_connection_in_progress = Some(connection_in_progress);

        //if no connection is in progress and we have a saved one, use it
        } else if let Some(last_connection_in_progress) = self.state.last_connection_in_progress {
            self.state.node_selection_actor = Some(NodeSelectionActor::DraggingOutput(mouse_pos, last_connection_in_progress.0, last_connection_in_progress.1));
            self.state.last_connection_in_progress = None;
        }

        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| self.draw_graph(ui, ctx, &action))
            .inner;

        let mut extra_responses = vec![];

        if let Some(node_selection_actor) = &self.state.node_selection_actor {
            let mut window_is_open = true;

            let new_node_ty = egui::Window::new("New node")
                .default_pos(
                    ctx.pointer_latest_pos()
                        .unwrap_or(ctx.available_rect().center()),
                )
                .open(&mut window_is_open)
                .collapsible(false)
                .show(ctx, |ui| self.tree.draw(ui).map(|leaf| leaf.ty.clone()))
                
                .map(|resp| resp.inner)
                .flatten()
                .flatten();

            if let Some(node_ty) = &new_node_ty {
                dbg!(node_selection_actor);
                let new_node_pos = node_selection_actor.pos();

                extra_responses.extend(self.add_node(node_ty, new_node_pos, node_selection_actor.connection()));

                self.state.node_selection_actor = None;
            }

            if !window_is_open || action == Some(GraphUiAction::Escape) {
                self.state.node_selection_actor = None;
            }
        }

        GraphUiResponse {
            graph_changes: graph_response
                .node_responses
                .iter()
                .filter_map(GraphChangeEvent::from_response)
                .chain(extra_responses)
                .collect_vec(),
        }
    }

    fn draw_graph(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        ui_action: &Option<GraphUiAction>,
    ) -> egui_node_graph::GraphResponse<GraphResponse, UiNodeData> {
        ui.set_clip_rect(ctx.available_rect());
        egui::widgets::global_dark_light_mode_switch(ui);

        if ui_action == &Some(GraphUiAction::Home) {
            self.editor.pan_zoom.pan = egui::Vec2::ZERO;
        }

        if ui.ui_contains_pointer() {
            self.editor.pan_zoom.pan += ctx.input().scroll_delta;

            if let Some(point) = ctx.input().pointer.hover_pos() {
                let zoom_delta = ctx.input().zoom_delta();
                self.editor
                    .pan_zoom
                    .adjust_zoom(zoom_delta, point.to_vec2(), 0.001, 100.0);
            }
        }

        let graph_resp = self
            .editor
            .draw_graph_editor(ui, AllNodeTypes, &mut self.graph_state);

        self.editor.node_finder = None;

        graph_resp
    }

    fn draw_animators(&mut self, ctx: &egui::Context) {
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
}
