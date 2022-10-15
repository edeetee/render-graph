use std::rc::Rc;

// use super::{def::{ComputedNodeInput, NodeTypes}, shaders::Shader};
use egui::TextureId;

use egui_wgpu::renderer::RenderPass;
use wgpu::{Device, TextureDescriptor, Texture, TextureViewDescriptor, TextureView, ImageCopyTexture};

pub fn srgb_2d_descriptor((width, height): (u32, u32)) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
    }
}

pub fn new_texture_f32_2d(device: &Device, (width, height): (u32, u32)) -> Texture  {
    device.create_texture(&TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
    })
}

/// A texture that can be used by the UI
/// And the GPU can copy to
pub struct UiTexture {
    tex: Texture,
    view: TextureView,
    id: TextureId,
    descriptor: TextureDescriptor<'static>,
}

impl UiTexture {
    pub fn new(
        device: &Device,
        egui: &mut RenderPass,
        size: (u32, u32)
    ) -> Self {

        let descriptor = srgb_2d_descriptor(size);
        let tex = device.create_texture(&srgb_2d_descriptor(size));

        let view = tex.create_view(&TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        // egui.regi

        let id = egui.register_native_texture(device, tex, wgpu::FilterMode::Nearest);

        Self {
            id,
            tex,
            view,
            descriptor
        }
    }

    pub fn update_size(&mut self, device: &Device, egui_render: &mut RenderPass, size: (u32, u32)) {
        let cur_size = (&mut self.descriptor.size.width, &mut self.descriptor.size.height);

        if cur_size != size {
            println!("Updating texture size from {:?} to {:?}", cur_size, size);

            egui_render.free_texture(&self.id);

            *self = Self::new(device, egui_render, size);


            // egui_render.re
        
            // egui_re3n.painter.replace_native_texture(*self.screen.borrow_id(), new_screen.borrow_tex().clone());
            // egui_render.free_texture(&self.id);
            // egui_render.register_native_texture(device, texture, texture_filter)

            // self.tex = tex;
        }
    }

    // pub fn copy_from(&mut self, from: &ImageCopyTexture){
    //     // let filter = glium::uniforms::MagnifySamplerFilter::Linear;

    //     // from.texture.

    //     surface.fill(
    //         self.screen.borrow_fb(),
    //         filter,
    //     );
    // }

    pub fn size(&self) -> (u32, u32) {
        self.screen.borrow_tex().dimensions()
    }

    pub fn clone_screen_tex_id(&self) -> TextureId {
        self.screen.borrow_id().clone()
    }
}



#[derive(Debug)]
pub struct TextureManager {
    textures: Vec<Rc<Texture>>,
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
    pub fn new_target(&mut self, device: &Device) -> Rc<Texture> {
        // let tex = new_texture_2d(facade, self.res[0], self.res[1]).unwrap();
        // self.textures.push(tex.clone());
        // tex
        if let Some(unused_tex) = self.textures.iter().filter(|tex| Rc::strong_count(tex) == 1).next(){
            unused_tex.clone()
        } else {
            self.get_new(device)
        }
    }

    fn get_new(&mut self, device: &Device) -> Rc<Texture> {
        let new_tex = Rc::new(new_texture_f32_2d(device, self.res).unwrap());
        self.textures.push(new_tex.clone());
        println!("New texture allocated inside {self:?}");

        new_tex
    }

    fn set_res(&mut self, device: &Device, res: (u32, u32)){
        self.res = res;
        for texture in self.textures.iter_mut() {
            *texture = Rc::new(new_texture_f32_2d(device, self.res).unwrap());
        }
    }

    // fn get_or_set(&mut self, device: &Device, index: usize) -> &Texture2d {
    //     if self.textures.get(index).is_none() {
    //         self.textures.insert(index, new_texture_2d(facade, self.res[0], self.res[1]).unwrap())
    //     }

    //     &self.textures[index]
    // }
}
