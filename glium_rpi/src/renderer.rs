use glium::{glutin::{self, window::Fullscreen, event::{self, Event}, event_loop::ControlFlow}, Surface, framebuffer::{RenderBuffer, SimpleFrameBuffer}, Frame};

use crate::{model::{Model}, render_loop::{UpdateInfo, DrawInfo}, feedback::FeedbackView, instances::{InstancesView, InstanceAttr}, util::DEFAULT_FORMAT};
use super::model;
use super::render_loop;

pub struct Options{
    pub num_stars: usize,
    pub model_options: model::Options
}

struct View<'a>{
    feedback: FeedbackView<'a>,
    stars: InstancesView<'a>,
    temp_buffer: SimpleFrameBuffer<'a>,
    res: [f32; 2]
}

pub fn main(options: Options) {
    let model = model::Model::new(options.num_stars, Some(options.model_options));

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();

    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    let monitor_handle = display.gl_window().window().available_monitors().next().unwrap();
    let fs = Fullscreen::Borderless(Some(monitor_handle));
    display.gl_window().window().set_fullscreen(Some(fs));

    let (width, height) = display.get_framebuffer_dimensions();
    let render_buffer = RenderBuffer::new(&display, DEFAULT_FORMAT, width, height).unwrap();
    let temp_surface = SimpleFrameBuffer::new(&display, &render_buffer).unwrap();

    let view_state = View {
        feedback: FeedbackView::new(&display),
        stars: InstancesView::new(&display, &model.stars),
        temp_buffer: temp_surface,
        res: [width, height].map(|s| s as f32)
    };

    render_loop::start(event_loop, display, model, view_state, update, draw, event);
}

fn update(model: &mut Model, view: &mut View, update_info: UpdateInfo) {
    model.update(update_info.time_since_previous.as_secs_f32());

    let new_instances_iter = model.stars.iter()
        .map(|star| {
            InstanceAttr{
                instance_pos: star.pos.to_array(),
                instance_radius: star.radius,
                instance_rgba: star.rgba
            }
        });

    view.stars.write_instances(new_instances_iter);
}

fn draw(frame: &mut Frame, model: &Model, view: &mut View, info: DrawInfo) {
    //get temp screen
    let draw_surface = &mut view.temp_buffer;

    draw_surface.clear_color(0., 0., 0., 0.);

    //draw feedback
    view.feedback.draw_to(draw_surface, view.res, info.time_since_previous.as_secs_f32(), model.feedback_displace).unwrap();

    //draw objects
    view.stars.draw_to(draw_surface, model.mat).unwrap();

    //copy to feedback
    view.feedback.fill_from(draw_surface);

    //draw to screen
    draw_surface.fill(frame, glium::uniforms::MagnifySamplerFilter::Linear);
}

fn event(ev: Event<()>, _model: &mut Model, view: &mut View) -> Option<ControlFlow> {
    match ev {
        event::Event::WindowEvent { event, .. } => match event {
            event::WindowEvent::CloseRequested => {
                Some(glutin::event_loop::ControlFlow::Exit)
            },
            event::WindowEvent::Resized(size) => {
                view.res = [size.width as f32, size.height as f32];
                None
            }
            event::WindowEvent::KeyboardInput { input, .. } => {
                if input.virtual_keycode == Some(event::VirtualKeyCode::Escape) {
                    Some(glutin::event_loop::ControlFlow::Exit)
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    } 
}