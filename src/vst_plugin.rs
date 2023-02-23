//! Minimal VST plugin with an editor window.
//!
//! The editor window is blank. Clicking anywhere in the window will print "Click!" to stdout.

use core::ffi::c_void;
use egui_glium::EguiGlium;
use glium::{backend::{Context, Facade}, glutin::WindowedContext};
// use glium::{Display, glutin::{window::WindowBuilder, ContextBuilder, event_loop::{EventLoopWindowTarget, EventLoop}}, backend::{glutin::{GlutinBackend}, Facade}};
use glutin::{config::ConfigTemplateBuilder, surface::SurfaceAttributesBuilder, context::ContextAttributesBuilder, display::Display, prelude::*};
use vst::{
    editor::Editor,
    plugin::{HostCallback, Info, Plugin},
    plugin_main,
};
use vst_window::{setup, EventSource, WindowEvent};

#[derive(Default)]
struct BasicPlugin {
    editor_placeholder: Option<MyPluginEditor>,
}

impl Plugin for BasicPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "Basic Plugin with Editor".to_string(),
            unique_id: 13579,

            ..Default::default()
        }
    }

    fn new(_host: HostCallback) -> Self {
        Self {
            editor_placeholder: Some(MyPluginEditor::default()),
        }
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        self.editor_placeholder
            .take()
            .map(|editor| Box::new(editor) as Box<dyn Editor>)
    }
}

plugin_main!(BasicPlugin);

#[derive(Default)]
struct MyPluginEditor {
    renderer: Option<MyRenderer>,
    window_events: Option<EventSource>,
}

const WINDOW_DIMENSIONS: (i32, i32) = (300, 200);

impl Editor for MyPluginEditor {
    fn size(&self) -> (i32, i32) {
        (WINDOW_DIMENSIONS.0 as i32, WINDOW_DIMENSIONS.1 as i32)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn open(&mut self, parent: *mut c_void) -> bool {
        if self.window_events.is_none() {
            let (window_handle, event_source) = setup(parent, WINDOW_DIMENSIONS);
            self.renderer = Some(MyRenderer::new(window_handle));
            self.window_events = Some(event_source);
            true
        } else {
            false
        }
    }

    fn is_open(&mut self) -> bool {
        self.window_events.is_some()
    }

    fn close(&mut self) {
        drop(self.renderer.take());
        drop(self.window_events.take());
    }

    fn idle(&mut self) {
        if let Some(window_events) = &mut self.window_events {
            while let Some(event) = window_events.poll_event() {
                match event {
                    WindowEvent::MouseClick(_) => println!("Click!"),
                    _ => (),
                }
            }
        }
        if let Some(renderer) = &mut self.renderer {
            renderer.draw_frame();
        }
    }
}

struct MyRenderer;

impl MyRenderer {
    pub fn new<W: raw_window_handle::HasRawWindowHandle>(_handle: W) -> Self {
        // EventLoopWindowTarget::
        // WindowBuilder::new().build(window_target)
        // let display = ContextBuilder::new().build_windowed(wb, el)
        let ct = ConfigTemplateBuilder::new()
            .compatible_with_native_window(_handle)
            .build();

        let sa = SurfaceAttributesBuilder::new()
            .build(_handle, WINDOW_DIMENSIONS.0, WINDOW_DIMENSIONS.1);

        let display = unsafe {
                Display::new(raw_window_handle::RawDisplayHandle::AppKit(()), glutin::display::DisplayApiPreference::Cgl)
                    .unwrap()
        };

        let context = unsafe {
            let config = display.find_configs(ct);

            display.create_context(&config, &sa)
        }.unwrap();

        let glium_ctx = glium::backend::glutin::headless::Headless::new(context)
            .unwrap();

        // context.make_current(surface)

        // let windowed_ctx = WindowedContext::

        // let glium_display = glium::Display
        
        // EguiGlium::new(display, event_loop)
        
        Self
    }
    pub fn draw_frame(&mut self) {
        /* ... */
    }
}
