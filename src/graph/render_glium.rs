use egui::frame;
use glium::{glutin::{window::Fullscreen, self}, framebuffer::{RenderBuffer, SimpleFrameBuffer}};
use glium_utils::{model_view_event_loop, util::DEFAULT_TEXTURE_FORMAT};

const DEFAULT_FULLSCREEN_MODE: Option<Fullscreen> = Some(Fullscreen::Borderless(None));

struct View {

}

pub fn render_glium() {
    // let model = model::Model::new(options.num_stars, Some(options.model_options));
    let model = ();

    let event_loop = glutin::event_loop::EventLoop::new();
    let display = create_display(&event_loop);

    let mut egui_glium = egui_glium::EguiGlium::new(&display);
    
    let (width, height) = display.get_framebuffer_dimensions();
    let render_buffer = RenderBuffer::new(&display, DEFAULT_TEXTURE_FORMAT, width, height).unwrap();
    let frame_buffer = SimpleFrameBuffer::new(&display, &render_buffer).unwrap();
    // let texture_buffer = frame_buffer.into()

    // egui_glium.painter.register_native_texture(native)

    // egui_glium.painter.register_native_texture(frame_buffer.());

    // let view_state = View::new(&display, &model, &render_buffer);

    model_view_event_loop::start(event_loop, &display, model, view_state, update, draw, event);
}

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("egui_glium example");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}