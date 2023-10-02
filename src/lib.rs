mod common;
mod fullscreen_shader;
mod gl_expression;
mod graph;
mod isf;
mod obj_shader;
mod textures;
pub mod util;

#[macro_use]
extern crate partial_application;

#[cfg(feature = "editor")]
mod editor;
#[cfg(feature = "editor")]
mod tree_view;
#[cfg(feature = "editor")]
mod widgets;

#[cfg(feature = "ffgl_plugin")]
pub mod ffgl_plugin; // mod ffgl;
