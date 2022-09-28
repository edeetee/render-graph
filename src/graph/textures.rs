use std::{
    rc::{Rc},
};

// use super::{def::{ComputedNodeInput, NodeTypes}, shaders::Shader};
use egui::TextureId;
use egui_glium::EguiGlium;
use glium::{
    backend::Facade,
    framebuffer::{SimpleFrameBuffer},
    texture::{SrgbTexture2d, UncompressedFloatFormat},
    Surface, Texture2d,
};

use ouroboros::self_referencing;

#[self_referencing]
struct ScreenTexture {
    tex: Rc<SrgbTexture2d>,
    id: TextureId,

    #[borrows(tex)]
    #[covariant]
    fb: SimpleFrameBuffer<'this>
}

impl ScreenTexture {
    pub fn generate(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
    ) -> Self {

        let mipmaps = glium::texture::MipmapsOption::NoMipmap;
        let format = glium::texture::SrgbFormat::U8U8U8U8;

        let tex = Rc::new(
            SrgbTexture2d::empty_with_format(
                facade,
                format,
                mipmaps,
                512,
                512,
            )
            .unwrap(),
        );

        let id = egui_glium
            .painter
            .register_native_texture(tex.clone());

        ScreenTextureBuilder {
            id,
            tex,
            fb_builder: |tex: &Rc<SrgbTexture2d>| {
                SimpleFrameBuffer::new(facade, tex.as_ref()).unwrap()
            },
        }
        .build()
    }
}

pub struct NodeTextures {
    screen: ScreenTexture,
}

const DEFAULT_MIPMAPS: glium::texture::MipmapsOption = glium::texture::MipmapsOption::NoMipmap;
const FORMAT_RGBA32: UncompressedFloatFormat = glium::texture::UncompressedFloatFormat::F32F32F32F32;

fn new_texture_2d(facade: &impl Facade, width: u32, height: u32) -> Result<Texture2d, glium::texture::TextureCreationError>  {
    Texture2d::empty_with_format(
        facade,
        FORMAT_RGBA32,
        DEFAULT_MIPMAPS,
        width,
        height,
    )
}

impl NodeTextures {
    pub fn new(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
    ) -> Self {

        Self {
            screen: ScreenTexture::generate(facade, egui_glium),
        }
    }

    pub fn copy_from(&mut self, surface: &impl Surface){
        let filter = glium::uniforms::MagnifySamplerFilter::Nearest;

        surface.fill(
            self.screen.borrow_fb(),
            filter,
        );
    }

    pub fn clone_screen_tex_id(&self) -> TextureId {
        self.screen.borrow_id().clone()
    }
}


#[derive(Debug)]
pub struct TextureManager {
    textures: Vec<Rc<Texture2d>>,
    res: [u32; 2]
}

impl Default for TextureManager {
    fn default() -> Self {
        Self {
            textures: Vec::new(),
            // res: [1920, 1080]
            res: [16, 16]
        }
    }
}

//Handles shared references of textures and will allocate new textures as needed
impl TextureManager {
    pub fn new_target(&mut self, facade: &impl Facade) -> Rc<Texture2d> {
        // let tex = new_texture_2d(facade, self.res[0], self.res[1]).unwrap();
        // self.textures.push(tex.clone());
        // tex
        if let Some(unused_tex) = self.textures.iter().filter(|tex| Rc::strong_count(tex) == 1).next(){
            unused_tex.clone()
        } else {
            self.get_new(facade)
        }
    }

    fn get_new(&mut self, facade: &impl Facade) -> Rc<Texture2d> {
        let new_tex = Rc::new(new_texture_2d(facade, self.res[0], self.res[1]).unwrap());
        self.textures.push(new_tex.clone());
        println!("New texture allocated inside {self:?}");

        new_tex
    }

    fn set_res(&mut self, facade: &impl Facade, res: [u32; 2]){
        self.res = res;
        for texture in self.textures.iter_mut() {
            *texture = Rc::new(new_texture_2d(facade, self.res[0], self.res[1]).unwrap());
        }
    }

    // fn get_or_set(&mut self, facade: &impl Facade, index: usize) -> &Texture2d {
    //     if self.textures.get(index).is_none() {
    //         self.textures.insert(index, new_texture_2d(facade, self.res[0], self.res[1]).unwrap())
    //     }

    //     &self.textures[index]
    // }
}



// This will infinitely return new textures to be given to inputs for rendering.
// 
// - Best used with a zip function
// - Requires exclusive control of the texture manager
// - Doesn't use the target texture in iteration
// struct FramedTextureManager<'a, F: Facade> {
//     manager: &'a mut TextureManager,
//     target: &'a Texture2d,
//     facade: &'a F,
//     index: usize
// }

// impl <'a, F: Facade> FramedTextureManager<'a, F>{
//     pub fn get_texture(&'a mut self) -> &'a Texture2d {
//         let result = self.manager.get_or_set(self.facade, self.index);
//         self.index += 1;

//         if result.get_id() != self.target.get_id() {
//             result
//         } else {
//             self.get_texture()
//         }
//     }
// }

// impl <'a, F: Facade> Drop for FramedTextureManager<'a, F>{
//     fn drop(&mut self) {
//         todo!()
//     }
// }