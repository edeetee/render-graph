use glium::{Display, Texture2d, Surface, backend::Context};
use glium::texture::{self, UncompressedFloatFormat, TextureCreationError};


pub fn get_res(display: &Display) -> [u32; 2] {
    let (w, h) = display.get_framebuffer_dimensions();
    return [w, h];
}

pub const DEFAULT_FORMAT: UncompressedFloatFormat = UncompressedFloatFormat::U16U16U16U16;

pub fn gen_texture(display: &Display) -> Result<Texture2d, TextureCreationError> {
    let (width, height) = display.get_framebuffer_dimensions();

    let texture = Texture2d::empty_with_format(
        display, 
        DEFAULT_FORMAT, 
        glium::texture::MipmapsOption::NoMipmap, 
        width, height
    )?;

    texture.as_surface().clear_color(0.0, 0.0, 0.0, 1.0);

    Ok(texture)
}

pub fn print_formats(context: &Context){
    let all_formats = texture::UncompressedFloatFormat::get_formats_list();
    let valid_formats = all_formats.iter()
        .filter(|format| format.is_color_renderable(context) && format.is_supported(context));

    if valid_formats.clone().count() != 0 {
        println!("Valid formats:");
        for format in valid_formats {
            println!("{format:?}")
        }
    } else {
        println!("No valid formats!");
    }
}