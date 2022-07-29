pub mod event_loop_render;
pub mod util;
pub mod modular_shader;

pub use glium;
pub use glam;

// #[macro_export]
// macro_rules! implement_uniform {
//     (struct $name:ident { $($fname:ident : $ftype:ty),* }) => {
//         struct $name {
//             $($fname : $ftype),*
//         }

//         // use glium::uniforms::Uniforms;

//         impl Uniforms for $name {
//             fn visit_values<'a, F: FnMut(&str, uniforms::UniformValue<'a>)>(&'a self, f: F) {
//                 $(f(stringify!($fname), self.$fname.as_uniform_value());),
//                 *
//             }
//         }
//     }
// }