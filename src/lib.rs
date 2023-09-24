mod common;
mod fullscreen_shader;
mod gl_expression;
mod graph;
mod isf;
mod obj_shader;
mod textures;
mod tree_view;
pub mod util;
mod widgets;

#[macro_use]
extern crate partial_application;

#[cfg(feature = "editor")]
mod editor;

#[cfg(feature = "ffgl_plugin")]
pub mod ffgl_plugin; // mod ffgl;
