use std::ops::Deref;

use crate::textures::TextureManager;
use crate::util::MappableTuple;
use crate::widgets::debug::debug_options;
use egui::style::{Margin, DebugOptions};
use egui::{Color32, RichText, Widget, Rect, Vec2};
use egui_glium::EguiGlium;
use egui_node_graph::{
    AnyParameterId, NodeDataTrait, NodeId, NodeResponse, NodeTemplateTrait, UserResponseTrait,
};
use glium::Texture2d;
use glium::{backend::Facade, Display, Surface};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use try_utils::some;

// use crate::textures::UiTexture;

use crate::common::persistent_state::{PersistentState, WindowState};
use crate::graph::{
    def::{GraphEditorState, GraphResponse, GraphState, UiNodeData},
    node_types::{AllNodeTypes, NodeType},
    GraphChangeEvent, ShaderGraphProcessor,
};


use super::node_textures::NodeUiTextures;
use super::node_tree_ui::{TreeState, LeafIndex};

pub struct GraphUi {
    processor: ShaderGraphProcessor,
    editor: GraphEditorState,
    graph_state: GraphState,
    tree: TreeState,
    node_textures: NodeUiTextures,
    state: GraphUiState,
    texture_manager: TextureManager,
}

#[derive(Serialize, Deserialize, Debug)]
enum NodeSelectionActor {
    Mouse(egui::Pos2),
    DraggingOutput(egui::Pos2, NodeId, AnyParameterId),
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

pub enum RenderRequest {
    Leaf(LeafIndex),
}

#[derive(Default)]
pub struct GraphUiResult {
    pub graph_changes: Vec<GraphChangeEvent>,
    pub render_requests: Vec<RenderRequest>
}

impl GraphUiResult {
    fn union(self, other: Self) -> Self{
        Self {
            graph_changes: self.graph_changes.into_iter().chain(other.graph_changes.into_iter()).collect_vec(),
            render_requests: self.render_requests.into_iter().chain(other.render_requests.into_iter()).collect_vec(),
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
            texture_manager: TextureManager::default(),
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
    ToggleViewState
}

impl GraphUiAction {
    fn from_keyboard_pressed(ctx: &egui::Context) -> Option<Self> {
        if ctx.input().key_pressed(egui::Key::H) {
            Some(Self::Home)
        } else if ctx.input().key_pressed(egui::Key::Tab) {
            Some(Self::ToggleAddNodeModal)
        } else if ctx.input().key_pressed(egui::Key::Escape) {
            Some(Self::Escape)
        } else if ctx.input().key_pressed(egui::Key::F1) {
            Some(Self::ToggleViewState)
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

pub struct RenderContext<'a> {
    display: &'a Display,
    egui_glium: &'a mut EguiGlium,
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

    delegate::delegate! {
         to self.processor {
             pub fn update(&mut self, [&mut self.editor.graph], [&self.graph_state], facade: &impl Facade);
         }
    }

    pub fn process_frame(&mut self, display: &Display, egui_glium: &mut EguiGlium) {
        let mut frame = display.draw();

        let action = GraphUiAction::from_keyboard_pressed(&egui_glium.egui_ctx);
        if action == Some(GraphUiAction::ToggleViewState) {
            self.state.view_state.toggle();
        }

        const MONO_COLOR: f32 = 0.1;
        frame.clear_color_and_depth((MONO_COLOR, MONO_COLOR, MONO_COLOR, 1.), 0.);

        
        let mut render_requests = vec![];

        match self.state.view_state {
            ViewState::Graph => {
                let mut graph_response = None;

                //prepare egui draw
                let _needs_repaint = egui_glium.run(display, |ctx| {
                    graph_response = Some(self.draw(ctx));
                });

                //Update data that stays aligned with the graph
                if let Some(response) = graph_response {

                    render_requests = response.render_requests;

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

        let render_previews_connection = self.state.node_selection_actor.as_ref().map(|actor| match actor {
            NodeSelectionActor::Mouse(_) => None,
            NodeSelectionActor::DraggingOutput(_, node_id, param_id) => Some((*node_id,*param_id)),
        })
            .flatten();

        let preview_tex_input = self.texture_manager.get_color(display);
        preview_tex_input.as_surface().clear_color(1.0, 1.0, 1.0, 1.0);

        let outputs = self.processor.render_shaders(
            &mut self.editor.graph,
            display,
            &mut self.texture_manager,
            |node_id, tex: &Texture2d| {
                let surface = tex.as_surface();

                if let Some((preview_target_node_id, param_id)) = render_previews_connection {
                    if node_id == preview_target_node_id {
                        surface.fill(&preview_tex_input.as_surface(), glium::uniforms::MagnifySamplerFilter::Linear);
                    }
                }

                self.node_textures
                    .copy_surface(display, egui_glium, node_id, &surface);
            },
        );

        let preview_requests = render_requests.iter().filter_map(|leaf |some!(leaf, if RenderRequest::Leaf)).cloned().collect_vec();

        for leaf_id in preview_requests {
            let leaf = &mut self.tree.leaves[leaf_id];

            leaf.render(display, egui_glium, &mut self.texture_manager, Some(preview_tex_input.as_ref()));
        }
        

        match self.state.view_state {
            ViewState::Graph => {
                egui_glium.paint(display, &mut frame);
            }
            ViewState::Output => {
                //for some reason required to make the frame correctly map onto the output
                egui_glium.paint(display, &mut frame);

                if let Some(output) = outputs.first().cloned().flatten() {
                    let filter = glium::uniforms::MagnifySamplerFilter::Nearest;
                    let dimens = display.get_framebuffer_dimensions();

                    let _dst_dimens = dimens.map(|x| (*x as f64));
                    let _src_dimens = output.dimensions();

                    frame.clear_all((0.0, 0.0, 0.0, 1.0), 0.0, 0);
                    output.as_surface().fill(&mut frame, filter);
                }
            }
        }

        frame.finish().unwrap();
    }

    pub fn add_node(&mut self, node_kind: &NodeType, position: egui::Pos2, connection: Option<(NodeId, AnyParameterId)>) -> Vec<GraphChangeEvent> {
        let mut responses = vec![];


        let num_copies = self.editor.graph.nodes.iter().filter(|(n_id,n)| n.user_data.template == *node_kind).count();

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

    pub fn draw(&mut self, ctx: &egui::Context) -> GraphUiResult {
        // let graph_response = egui::CentralPanel::default()
        //     .show(ctx, |ui| self.draw_graph(ui, ctx, &None))
        //     .inner;

        // return GraphUiResult::default();

        let action = GraphUiAction::from_keyboard_pressed(ctx);

        if let Some(action) = &action {
            dbg!(action);
        }

        egui::TopBottomPanel::top("Titlebar").show(ctx, |ui| {});

        if !self.graph_state.animations.is_empty() {
            self.draw_animators(ctx);
        }

        if action == Some(GraphUiAction::ToggleAddNodeModal) {
            self.state.node_selection_actor = if self.state.node_selection_actor.is_none() {
                Some(NodeSelectionActor::Mouse(self.interaction_pos_on_graph(ctx)))
            } else {
                None
            };
        }

        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| self.draw_graph(ui, ctx, &action))
            .inner;

        let node_responses = graph_response.node_responses;

        //if connection sucessfully ended
        if node_responses.iter().any(|resp| matches!(resp, NodeResponse::ConnectEventEnded { .. } | NodeResponse::DisconnectEvent { .. })) {
            self.state.last_connection_in_progress = None;
        }

        //if connection started, save it
        if let Some(NodeResponse::ConnectEventStarted(node_id, param_id)) = node_responses.iter().find(|resp| matches!(resp, NodeResponse::ConnectEventStarted(..))) {
            self.state.last_connection_in_progress = Some((*node_id, *param_id));

        //if we were just connecting
        } else if let Some(last_connection_in_progress) = self.state.last_connection_in_progress {
            //and it has ended
            if self.editor.connection_in_progress.is_none() {
                self.state.node_selection_actor = Some(NodeSelectionActor::DraggingOutput(self.interaction_pos_on_graph(ctx), last_connection_in_progress.0, last_connection_in_progress.1));

                let previewed_node = &self.editor.graph.nodes[last_connection_in_progress.0];
                // dbg!(previewed_node);

                self.state.last_connection_in_progress = None;
            }
        }

        let extra_responses = self.draw_node_selector_window(action, ctx);

        GraphUiResult {
            graph_changes: node_responses
                .iter()
                .filter_map(GraphChangeEvent::from_response)
                .collect_vec(),
            ..Default::default()
        }.union(extra_responses)
    }

    fn interaction_pos_on_graph(&self, ctx: &egui::Context) -> egui::Pos2 {
        ctx.pointer_latest_pos().unwrap_or(ctx.available_rect().left_top()) - self.editor.pan_zoom.pan
    }

    fn draw_node_selector_window(&mut self, action: Option<GraphUiAction>, ctx: &egui::Context) -> GraphUiResult {
        let node_selection_window = egui::Window::new("New node");
        let mut extra_responses = vec![];

        let mut tree_result = None;

        if let Some(node_selection_actor) = &self.state.node_selection_actor {
            let mut window_is_open = true;
            let new_node_pos = node_selection_actor.pos();

            let modal_rect = Rect::from_center_size(new_node_pos+self.editor.pan_zoom.pan, Vec2::new(256.0, 256.0));

            let selection_window_resp = node_selection_window
                .default_rect(modal_rect)
                .open(&mut window_is_open)
                .scroll2([false, true])
                .collapsible(false)
                .show(ctx, |ui| self.tree.draw(ui));
            
            tree_result = selection_window_resp.map(|resp| resp.inner)
                .flatten();

            let new_node_ty = tree_result.as_ref().map(|res| res.clicked).flatten().map(|clicked| self.tree.leaves[clicked].ty.clone());

            if let Some(node_ty) = &new_node_ty {
                dbg!(node_selection_actor);

                extra_responses.extend(self.add_node(node_ty, new_node_pos, node_selection_actor.connection()));

                self.state.node_selection_actor = None;
            }

            if !window_is_open || action == Some(GraphUiAction::Escape) {
                self.state.node_selection_actor = None;
            }
        } else {
            // ctx.memory().reset_areas();
        }

        GraphUiResult {
            graph_changes: extra_responses,
            render_requests: tree_result.map(|result|
                result.in_view.into_iter().map(RenderRequest::Leaf).collect()
            ).unwrap_or_default()
        }
    }

    fn draw_graph(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        ui_action: &Option<GraphUiAction>,
    ) -> egui_node_graph::GraphResponse<GraphResponse, UiNodeData> {
        debug_options(ctx, ui);

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
        egui::Window::new("Animators").show(ctx, |ui| {
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
