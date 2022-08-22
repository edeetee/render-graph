use glium::{Texture2d, Surface, uniform, uniforms::{self}, DrawParameters, Blend, BlendingFunction, LinearBlendingFactor, DrawError, backend::Facade, framebuffer::SimpleFrameBuffer};

use crate::{util::*};
use super::{fullscreen_shader::{FullscreenFrag}};


pub struct FeedbackView {
    texture: Texture2d,
    fullscreen: FullscreenFrag,

    pub size: [f32; 2],
    pub displace: [f32; 2],
    pub feedback_gain: f32,
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

impl FeedbackView {
    fn draw_to(&self, surface: &mut impl Surface) -> Result<(), DrawError> {
        let feedback_sampler = self.texture.sampled()
            .wrap_function(uniforms::SamplerWrapFunction::BorderClamp);

        let uniforms = uniform! {
            feedback_texture: feedback_sampler,
            // size: self.size,
            displace: self.displace,
            feedback_mult: self.feedback_gain
        };

        self.fullscreen.draw(surface, uniforms)
    }
}

// impl ResHolder for FeedbackView{
//     fn res_ref(&mut self) -> &mut [f32; 2] {
//         &mut self.size
//     }
// }

impl FeedbackView {
    pub fn new<F: Facade>(facade: &F) -> Self {
        
        let feedback_texture = gen_texture(facade).unwrap();

        let params = DrawParameters {
            dithering: true,
            // smooth: Some(Smooth::Fastest),
            blend: FEEDBACK_BLEND,
            .. Default::default()
        };

        Self{
            fullscreen: FullscreenFrag::new_with_params(facade, include_str!("feedback.frag"), params),
            texture: feedback_texture,
            size: [1., 1.],
            displace: [0., 0.],
            feedback_gain: 0.99
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