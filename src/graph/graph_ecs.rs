use shipyard::{Component, NonSendSync, View, IntoIter};

use crate::{isf::{shader::IsfShader, updater::IsfUpdater}, gl_expression::{GlExpressionUpdater, GlExpressionRenderer}, obj_shader::renderer};

use super::{graph_processor::OutputTarget, graph::ShaderGraph, node_shader::NodeShader};

// struct RenderNodesSystem;

// impl<'a> System<'a> for RenderNodesSystem {
//     type SystemData = Read<'a, OutputTarget>;

//     fn run(&mut self, data: Self::SystemData) {
//         todo!()
//     }

    
// }

#[derive(Component)]
struct GlExpressionUpdateComponent(GlExpressionUpdater);

#[derive(Component)]
struct GlExpressionRenderComponent(GlExpressionRenderer);

fn update_gl(
    renderers: View<GlExpressionRenderComponent>,
    updaters: View<GlExpressionUpdateComponent>
) {
    for (render, update) in (&renderers, &updaters).iter() {
        
    }
}

fn update_world() {

}

// fn render_nodes(output_targets: Query<(Entity, &OutputTarget)>, graph: Res<ShaderGraph>, shaders: Query<NodeShader>) {

//     for (output_id, output_target) in &mut self.output_targets {
//         let fb = SimpleFrameBuffer::new(facade, &output_target.rb).unwrap();

//         fb.clear_color(0., 0., 0., 0.);

//         self.graph.map_with_inputs(output_id, &mut |node_id, inputs| {

//             // let target = self.texture_manager.get_color(facade);

//             //Render a shader
//             if let Some(shader) = self.shaders.get_mut(node_id) {
//                 // let mut surface = target.as_surface();

//                 // surface.clear_color(0., 0., 0., 0.);

//                 let target = shader.render(facade, &mut self.texture_manager, ShaderInputs::from(&inputs));

//                 let surface = target.as_surface();

//                 let (w, h) = surface.get_dimensions();
//                 let size = (w/4, h/4);

//                 // node.user_data.
//                 // self.node_textures[node_id].borrow_mut()
//                 let mut ui_texture = (*self.node_textures[node_id]).borrow_mut();
//                 ui_texture.update_size(facade, egui_glium, size);
//                 ui_texture.copy_from(&surface);

//                 Some(target)
//             } else {
//                 None
//             }
//         }, &mut SecondaryMap::new());

//         // println!("RENDERED {rendered_node_names} to {}", self.graph[output_id].label);
//     }
// }