use glium::{Display, Texture2d, Surface, uniform, uniforms::{self}, DrawParameters, Blend, BlendingFunction, LinearBlendingFactor, DrawError};

use crate::{util::*};
use super::{fullscreen_shader::{FullscreenFrag}, modular_shader::{ModularShader}};


pub struct FeedbackView {
    texture: Texture2d,
    fullscreen: FullscreenFrag,

    pub size: [f32; 2],
    pub displace: [f32; 2],
    pub feedback_mult: f32,
}

const FEEDBACK_BLEND: Blend = Blend{
    color: BlendingFunction::Addition {
        source: LinearBlendingFactor::SourceAlpha,
        destination: LinearBlendingFactor::DestinationAlpha,
    },
    alpha: BlendingFunction::Addition {
        source: LinearBlendingFactor::SourceAlpha,
        destination: LinearBlendingFactor::OneMinusSourceAlpha
    },
    constant_value: (0.0, 0.0, 0.0, 0.0)
};

impl ModularShader for FeedbackView {
    fn draw_to<S: Surface>(&self, surface: &mut S) -> Result<(), DrawError> {
        let feedback_sampler = self.texture.sampled()
            .wrap_function(uniforms::SamplerWrapFunction::BorderClamp);

        let uniforms = uniform! {
            feedback_texture: feedback_sampler,
            size: self.size,
            displace: self.displace,
            feedback_mult: self.feedback_mult
        };

        self.fullscreen.draw(surface, &uniforms)
    }
}

// impl ResHolder for FeedbackView{
//     fn res_ref(&mut self) -> &mut [f32; 2] {
//         &mut self.size
//     }
// }

impl FeedbackView {
    pub fn new(display: &Display) -> Self {
        
        let feedback_texture = gen_texture(&display).unwrap();

        let params = DrawParameters {
            dithering: true,
            // smooth: Some(Smooth::Fastest),
            blend: FEEDBACK_BLEND,
            .. Default::default()
        };

        Self{
            fullscreen: FullscreenFrag::new_with_params(&display, include_str!("feedback.frag"), params),

            texture: feedback_texture,
            size: [1., 1.],
            displace: [0., 0.],
            feedback_mult: 0.99
        }
    }

    ///copy the surface to the feedback texture
    ///to be used with
    /// ```
    /// Self::draw_to()
    /// ```
    pub fn feedback_from<S: Surface>(&self, surface: &S){    
        surface.fill(&self.texture.as_surface(),glium::uniforms::MagnifySamplerFilter::Linear);
    }
}