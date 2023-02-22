use std::path::PathBuf;

use egui::{RichText, Widget};
use egui_glium::EguiGlium;
use egui_node_graph::{NodeId, NodeTemplateTrait};
use glium::{backend::Facade, Display, Surface};

use super::{ShaderGraphProcessor, def::{GraphEditorState, GraphState, GraphResponse, UiNodeData, Graph}, node_tree_ui::TreeState, node_types::{NodeType, AllNodeTypes}, graph::load_from_file_or_default};

pub struct GraphUi {
    processor: ShaderGraphProcessor,
    editor: GraphEditorState, 
    state: GraphState,
    tree: TreeState  
}

impl Default for GraphUi {
   fn default() -> Self {
       Self { 
           editor: GraphEditorState::new(1.0), 
           state: GraphState::default(),
           tree: TreeState::default(),
           processor: ShaderGraphProcessor::default()
       }
   }
}

impl GraphUi {

   pub fn load_from_file_or_default(file: &PathBuf, facade: &impl Facade, egui_glium: &mut EguiGlium) -> Self {
      let (mut editor, events) = load_from_file_or_default(file);

      let mut shader_node_graph = ShaderGraphProcessor::default();

      for event in events {
            shader_node_graph.graph_event(&mut editor.graph, facade, egui_glium, event);
      }

      Self { processor: shader_node_graph, editor: editor, state: GraphState::default(), tree: TreeState::default() }
   }
   
   pub fn editor(&self) -> & GraphEditorState {
       &self.editor
   }

   delegate::delegate! {
        to self.processor {
            pub fn update(&mut self, [&mut self.editor.graph], [&self.state], facade: &impl Facade);
        }
   }

   // pub fn input_params(&self, node_id: NodeId) -> InputParams<'_> {
   //     self.graph_ref()[node_id].inputs.iter().map(|(name, input_id)| {
   //         (name.as_str(), &self.graph_ref()[*input_id])
   //     }).collect()
   // }

   pub fn process_frame(&mut self, display: &Display, egui_glium: &mut EguiGlium) {
      let mut frame = display.draw();
   
      frame.clear_color_and_depth((1., 1., 1., 1.), 0.);
   
      let mut graph_response = None;
   
      let _needs_repaint = egui_glium.run(display, |ctx| {
          graph_response = Some(self.draw(ctx));
      });
   
      if let Some(response) = graph_response {
          for event in response.node_responses {
              self.processor.graph_event(&mut self.editor.graph, display, egui_glium, event);
          }
      }
   
      self.processor.render_shaders(&mut self.editor.graph, display, egui_glium);
   
      egui_glium.paint(display, &mut frame);
   
      frame.finish().unwrap();
   }


   pub fn add_node(&mut self, node_kind: &NodeType, position: egui::Pos2) -> NodeId {
       // println!("Adding node {node_kind:#?}");

       let new_node = self.editor.graph.add_node(
           node_kind.node_graph_label(&mut self.state),
           node_kind.user_data(&mut self.state),
           |graph, node_id| node_kind.build_node(graph, &mut self.state, node_id),
       );
       self.editor.node_positions.insert(
           new_node,
           position,
       );
       self.editor.node_order.push(new_node);

       new_node
   }

   pub fn draw(&mut self, ctx: &egui::Context) -> egui_node_graph::GraphResponse<GraphResponse, UiNodeData> {
       let mut new_node_ty = None;

       egui::SidePanel::left("tree_view").show(ctx, |ui| {
           if let Some(selected_item) = self.tree.draw(ui) {
               new_node_ty = Some(selected_item.ty.clone());
           }
       });

       if !self.state.animations.is_empty() {
           egui::SidePanel::left("animators").show(ctx, |ui| {
               let mut removal = None;
               for (key, updater) in &mut self.state.animations {
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
                   self.state.animations.remove(&removal);
               }

           });
       }

       egui::CentralPanel::default().show(ctx, |ui| {
           ui.set_clip_rect(ctx.available_rect());
           egui::widgets::global_dark_light_mode_switch(ui);

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
                   self.editor.pan_zoom.adjust_zoom(zoom_delta, point.to_vec2(), 0.001, 100.0);
               }
               // self.0.pan_zoom.zoom *= ctx.input().zoom_delta();
               // dbg!(self.0.pan_zoom.zoom);
           }

           let mut graph_resp = self.editor.draw_graph_editor(ui, AllNodeTypes, &mut self.state);
           self.editor.node_finder = None;
           graph_resp.node_responses.append(&mut responses);

           graph_resp
       }).inner
   }
}

