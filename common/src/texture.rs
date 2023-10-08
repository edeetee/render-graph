use glium::{
    backend::Facade,
    texture::{DepthTexture2d, SrgbTexture2d},
    Texture2d,
};

pub const DEFAULT_RES: (u32, u32) = (1920, 1080);

const NO_MIPMAP: glium::texture::MipmapsOption = glium::texture::MipmapsOption::NoMipmap;

pub fn new_texture_2d(
    facade: &impl Facade,
    (width, height): (u32, u32),
) -> Result<Texture2d, glium::texture::TextureCreationError> {
    Texture2d::empty_with_format(
        facade,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        NO_MIPMAP,
        width,
        height,
    )
}

pub fn new_depth_texture_2d(
    facade: &impl Facade,
    (width, height): (u32, u32),
) -> Result<DepthTexture2d, glium::texture::TextureCreationError> {
    DepthTexture2d::empty_with_format(
        facade,
        glium::texture::DepthFormat::I16,
        NO_MIPMAP,
        width,
        height,
    )
}

pub fn new_texture_srgb_2d(
    facade: &impl Facade,
    (width, height): (u32, u32),
) -> Result<SrgbTexture2d, glium::texture::TextureCreationError> {
    SrgbTexture2d::empty_with_format(
        facade,
        glium::texture::SrgbFormat::U8U8U8U8,
        NO_MIPMAP,
        width,
        height,
    )
}
