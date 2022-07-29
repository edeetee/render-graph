use super::{stars::Star, model::Model};
use super::model;

use glium_utils::glium::uniforms;
use glium_utils::glium::{glutin::{self, window::{Fullscreen}, event::{self, Event, ElementState}, event_loop::ControlFlow}, Surface, framebuffer::{RenderBuffer, SimpleFrameBuffer}, Frame, Display};

use glium_utils::modular_shader::modular_shader::{ModularShader, ShaderUpdate};
use glium_utils::util::TogglingFullscreen;
use glium_utils::{model_view_event_loop::{UpdateInfo, DrawInfo}, util::DEFAULT_TEXTURE_FORMAT, modular_shader::{feedback::FeedbackView, instances::{InstancesView, InstanceAttr}, sdf::SdfView}};

use glium_utils::model_view_event_loop;

pub struct Options{
    pub num_stars: usize,
    pub model_options: model::Options
}

struct View<'a>{
    feedback: FeedbackView,
    stars: InstancesView,
    sdf: SdfView,
    
    temp_buffer: SimpleFrameBuffer<'a>,
    display: &'a Display
}

impl<'a> View<'a> {
    fn new(display: &'a Display, model: &Model, render_buffer: &'a RenderBuffer) -> Self {
        let temp_surface = SimpleFrameBuffer::new(display, render_buffer).unwrap();

        let feedback = FeedbackView::new(&display);
        let stars = InstancesView::new(&display, model.stars.iter(), model.mat);
        let sdf = SdfView::new(&display);
    
        Self {
            feedback: feedback,
            stars: stars,
            sdf: sdf,
            temp_buffer: temp_surface,
            display
        }
    }

    fn shaders_iter_mut(&mut self) -> [&mut dyn ModularShader; 2] {
        [&mut self.feedback, &mut self.sdf]
    }

    fn update_shaders(&mut self, update: ShaderUpdate){
        for shader in self.shaders_iter_mut(){
            shader.update(&update)
        }
    }
}

impl From<&Star> for InstanceAttr {
    fn from(star: &Star) -> Self {
        Self {
            instance_pos: star.pos.to_array(),
            instance_rgba: star.rgba,
            instance_scale: star.scale.to_array()
        }
    }
}

pub fn render_stars(options: Options) {
    let mut model = model::Model::new(options.num_stars, Some(options.model_options));

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();

    let display = Display::new(wb, cb, &event_loop).unwrap();
    
    let (width, height) = display.get_framebuffer_dimensions();
    let render_buffer = RenderBuffer::new(&display, DEFAULT_TEXTURE_FORMAT, width, height).unwrap();
    let mut temp_surface = SimpleFrameBuffer::new(&display, &render_buffer).unwrap();

    let mut feedback = FeedbackView::new(&display);
    let mut stars = InstancesView::new(&display, model.stars.iter(), model.mat);
    let mut sdf = SdfView::new(&display);

    let all_shaders: [&mut dyn ModularShader; 2] = [&mut feedback, &mut sdf];

    let update_shaders = |shader_update: ShaderUpdate| {
        for shader in all_shaders {
            shader.update(&shader_update)
        }
    };

    let update = |update_info: UpdateInfo| {
        model.update(update_info.time_since_previous.as_secs_f32());

        let new_instances_iter = model.stars.iter()
            .map(|star| {
                InstanceAttr{
                    instance_pos: star.pos.to_array(),
                    instance_scale: star.scale.to_array(),
                    instance_rgba: star.rgba
                }
            });
    
        stars.write_instances(new_instances_iter);
    };

    let draw = |frame: &mut Frame, info: DrawInfo| {
        //get temp screen
        let draw_surface = &mut temp_surface;
    
        // draw_surface.clear_color(0., 0., 0., 0.);
        sdf.draw_to(draw_surface).unwrap();
    
        //draw feedback
        feedback.draw_to(draw_surface).unwrap();
    
        //draw objects
        stars.draw_to(draw_surface).unwrap();
    
        //copy to feedback
        feedback.feedback_from(draw_surface);
    
        //draw to screen
        draw_surface.fill(frame, uniforms::MagnifySamplerFilter::Linear);
    };

    let event = |ev: Event<()>| {
        match ev {
            event::Event::WindowEvent { event, .. } => {
                match event {
                    event::WindowEvent::Resized(size) => {
                        (&update_shaders)(ShaderUpdate::Resolution([size.width as f32, size.height as f32]));
                    }
                    event::WindowEvent::KeyboardInput { input, .. } => {
                        match input.virtual_keycode {
                            Some(event::VirtualKeyCode::F11) =>
                                {
                                    if input.state == ElementState::Pressed{
                                        display.toggle_fullscreen();
                                    }
                                }
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }

            _ => ()
        };
        
        model_view_event_loop::default_ev_control_flow(ev)
    };

    model_view_event_loop::start(event_loop, &display, update, draw, event);
}

