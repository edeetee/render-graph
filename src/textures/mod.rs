use std::rc::Rc;

// use super::{def::{ComputedNodeInput, NodeTypes}, shaders::Shader};
use egui::TextureId;
use egui_glium::EguiGlium;
use glium::{
    backend::Facade,
    framebuffer::SimpleFrameBuffer,
    Surface,
    texture::{SrgbTexture2d, UncompressedFloatFormat}, Texture2d,
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

pub fn new_texture_srgb_2d(facade: &impl Facade, (width, height): (u32, u32)) -> Result<SrgbTexture2d, glium::texture::TextureCreationError>  {
    SrgbTexture2d::empty_with_format(
        facade,
        glium::texture::SrgbFormat::U8U8U8U8,
        DEFAULT_MIPMAPS,
        width,
        height,
    )
}

impl ScreenTexture {
    pub fn generate(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
        size: (u32, u32),
    ) -> Self {

        let tex = Rc::new(new_texture_srgb_2d(facade, size).unwrap());

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

pub struct UiTexture {
    screen: ScreenTexture,
}

const DEFAULT_MIPMAPS: glium::texture::MipmapsOption = glium::texture::MipmapsOption::NoMipmap;
const FORMAT_RGBA32: UncompressedFloatFormat = glium::texture::UncompressedFloatFormat::F32F32F32F32;

pub fn new_texture_2d(facade: &impl Facade, (width, height): (u32, u32)) -> Result<Texture2d, glium::texture::TextureCreationError>  {
    Texture2d::empty_with_format(
        facade,
        FORMAT_RGBA32,
        DEFAULT_MIPMAPS,
        width,
        height,
    )
}

impl UiTexture {
    pub fn new(
        facade: &impl Facade,
        egui_glium: &mut EguiGlium,
        size: (u32, u32)
    ) -> Self {

        Self {
            screen: ScreenTexture::generate(facade, egui_glium, size),
        }
    }

    pub fn update_size(&mut self, facade: &impl Facade, egui_glium: &mut EguiGlium, size: (u32, u32)) {
        if self.screen.borrow_tex().dimensions() != size {
            let new_screen = ScreenTexture::generate(facade, egui_glium, size);

            println!("Updating texture size from {:?} to {:?}", self.screen.borrow_tex().dimensions(), size);
        
            egui_glium.painter.replace_native_texture(*self.screen.borrow_id(), new_screen.borrow_tex().clone());

            self.screen = new_screen;
        }
    }

    pub fn copy_from(&mut self, surface: &impl Surface){
        let filter = glium::uniforms::MagnifySamplerFilter::Linear;

        surface.fill(
            self.screen.borrow_fb(),
            filter,
        );
    }

    pub fn size(&self) -> (u32, u32) {
        self.screen.borrow_tex().dimensions()
    }

    pub fn clone_screen_tex_id(&self) -> TextureId {
        self.screen.borrow_id().clone()
    }
}



#[derive(Debug)]
pub struct TextureManager {
    textures: Vec<Rc<Texture2d>>,
    res: (u32, u32)
}

impl Default for TextureManager {
    fn default() -> Self {
        Self {
            textures: Vec::new(),
            res: (1920, 1080)
            // res: [16, 16]
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
        let new_tex = Rc::new(new_texture_2d(facade, self.res).unwrap());
        self.textures.push(new_tex.clone());
        println!("New texture allocated inside {self:?}");

        new_tex
    }

    fn set_res(&mut self, facade: &impl Facade, res: (u32, u32)){
        self.res = res;
        for texture in self.textures.iter_mut() {
            *texture = Rc::new(new_texture_2d(facade, self.res).unwrap());
        }
    }

    // fn get_or_set(&mut self, facade: &impl Facade, index: usize) -> &Texture2d {
    //     if self.textures.get(index).is_none() {
    //         self.textures.insert(index, new_texture_2d(facade, self.res[0], self.res[1]).unwrap())
    //     }

    //     &self.textures[index]
    // }
}
