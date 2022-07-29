use glium::backend::Facade;
use glium::glutin::window::Fullscreen;
use glium::{Display, Texture2d, Surface, backend::Context};
use glium::texture::{self, UncompressedFloatFormat, TextureCreationError};

pub fn get_res(display: &Display) -> [u32; 2] {
    let (w, h) = display.get_framebuffer_dimensions();
    return [w, h];
}

pub const DEFAULT_TEXTURE_FORMAT: UncompressedFloatFormat = UncompressedFloatFormat::U16U16U16U16;

pub fn gen_texture<F: Facade>(facade: &F) -> Result<Texture2d, TextureCreationError> {
    let (width, height) = facade.get_context().get_framebuffer_dimensions();

    let texture = Texture2d::empty_with_format(
        facade, 
        DEFAULT_TEXTURE_FORMAT, 
        glium::texture::MipmapsOption::NoMipmap, 
        width, height
    )?;

    texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

    Ok(texture)
}

pub fn print_formats(context: &Context){
    let all_formats = texture::UncompressedFloatFormat::get_formats_list();

    let valid_formats = all_formats.iter()
        .filter(|format| format.is_color_renderable(context) && format.is_supported(context))
        .collect::<Vec<_>>();
    
    if valid_formats.len() != 0 {
        println!("Valid formats:");
        for format in valid_formats {
            println!("{format:?}")
        }
    } else {
        println!("No valid formats!");
    }
}

pub const DEFAULT_FULLSCREEN_MODE: Option<Fullscreen> = Some(Fullscreen::Borderless(None));

pub trait TogglingFullscreen{
    fn toggle_fullscreen(&self);
}

impl TogglingFullscreen for Display {
    fn toggle_fullscreen(&self) {
        let gl_window = self.gl_window();
        let current_mode = gl_window.window().fullscreen();

        let new_fullscreen_mode = if current_mode.is_some() {
            None
        } else {
            DEFAULT_FULLSCREEN_MODE
        };

        gl_window.window().set_fullscreen(new_fullscreen_mode);
    }
}