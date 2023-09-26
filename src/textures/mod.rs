use std::rc::Rc;

use glium::{
    backend::Facade,
    texture::{DepthTexture2d, SrgbTexture2d},
    Texture2d,
};

pub mod ui;

const NO_MIPMAP: glium::texture::MipmapsOption = glium::texture::MipmapsOption::NoMipmap;

pub fn new_texture_2d(
    facade: &impl Facade,
    (width, height): (u32, u32),
) -> Result<Texture2d, glium::texture::TextureCreationError> {
    Texture2d::empty_with_format(
        facade,
        glium::texture::UncompressedFloatFormat::F32F32F32F32,
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
        glium::texture::DepthFormat::F32,
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

#[derive(Debug)]
pub struct TextureManager {
    pub color_textures: Vec<Rc<Texture2d>>,
    pub depth_textures: Vec<Rc<DepthTexture2d>>,
    pub res: (u32, u32),
}

pub const DEFAULT_RES: (u32, u32) = (1920, 1080);

impl Default for TextureManager {
    fn default() -> Self {
        Self {
            color_textures: Vec::new(),
            depth_textures: Vec::new(),
            res: DEFAULT_RES,
        }
    }
}

fn get_unused_or_push<T>(vec: &mut Vec<Rc<T>>, f: impl Fn() -> T) -> Rc<T> {
    if let Some(unused_tex) = vec.iter().filter(|tex| Rc::strong_count(tex) == 1).next() {
        unused_tex.clone()
    } else {
        let new = Rc::new(f());
        vec.push(new.clone());
        new
    }
}

//Handles shared references of textures and will allocate new textures as needed
impl TextureManager {
    pub fn get_color(&mut self, facade: &impl Facade) -> Rc<Texture2d> {
        get_unused_or_push(&mut self.color_textures, || {
            new_texture_2d(facade, self.res).unwrap()
        })
    }

    pub fn get_depth(&mut self, facade: &impl Facade) -> Rc<DepthTexture2d> {
        get_unused_or_push(&mut self.depth_textures, || {
            new_depth_texture_2d(facade, self.res).unwrap()
        })
    }

    pub fn clear(&mut self) {
        self.color_textures.clear();
        self.depth_textures.clear();
    }

    // fn set_res(&mut self, facade: &impl Facade, res: (u32, u32)){
    //     self.res = res;
    //     for texture in self.color_textures.iter_mut() {
    //         *texture = Rc::new(new_texture_depth_2d(facade, self.res).unwrap());
    //     }
    // }

    // fn get_or_set(&mut self, facade: &impl Facade, index: usize) -> &Texture2d {
    //     if self.textures.get(index).is_none() {
    //         self.textures.insert(index, new_texture_2d(facade, self.res[0], self.res[1]).unwrap())
    //     }

    //     &self.textures[index]
    // }
}
