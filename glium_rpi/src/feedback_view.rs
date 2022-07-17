use glam::Vec3;
use glium::{Display, vertex, VertexBuffer, implement_vertex, Program, Texture2d, Surface, index, uniform, uniforms, DrawParameters, Smooth, Blend};
use stars::Stars;
use crate::util::*;

#[derive(Copy, Clone)]
struct VertexAttr {
    position: [f32; 3]
}
implement_vertex!(VertexAttr, position);

const FULLSCREEN_TRI: [[f32; 3]; 3] = [
    [-1.0, -1.0, 0.0],
    [3.0, -1.0, 0.0],
    [-1.0, 3.0, 0.0]
];

pub struct FeedbackView<'a> {
    vert_buffer: VertexBuffer<VertexAttr>,
    texture: Texture2d,
    program: Program,
    display: &'a Display,
    params: DrawParameters<'a>
}

impl<'a> FeedbackView<'a>{
    pub fn new(display: &'a Display) -> Self {
    
        let verts = FULLSCREEN_TRI
            .map(|arr| VertexAttr {position: arr} );
        let vert_buffer = VertexBuffer::new(display, &verts).unwrap();
    
        let program = Program::from_source(
            display,
            include_str!("feedback.vert"),
            include_str!("feedback.frag"),
            None
        ).unwrap();

        let feedback_texture = gen_texture(&display).unwrap();

        let params = glium::DrawParameters {
            dithering: true,
            smooth: Some(Smooth::Fastest),
            blend: Blend::alpha_blending(),
            .. Default::default()
        };

        Self{
            display,
            vert_buffer,
            texture: feedback_texture,
            program,
            params
        }
    }

    ///draw from the feedback texture to the surface
    ///to be used with 
    /// ```
    /// Self::fill_from()
    /// ```
    pub fn draw_to<S: Surface>(&self, surface: &mut S, res: [f32; 2], elapsed_s: f32, displace: [f32; 2]) -> Result<(), glium::DrawError>
    {
        let feedback_sampler = self.texture.sampled()
            .wrap_function(uniforms::SamplerWrapFunction::BorderClamp);

        //draw (exits if error)
        surface.draw(
            &self.vert_buffer,
            &index::NoIndices(index::PrimitiveType::TrianglesList),
            &self.program,
            &uniform! {
                feedback_texture: feedback_sampler,
                size: res,
                displace: displace,
                feedback_mult: elapsed_s*10.0
            },
            &self.params
        )?;

        //copy to texture
        surface.fill(&self.texture.as_surface(),glium::uniforms::MagnifySamplerFilter::Linear);

        Ok(())
    }

    ///copy the surface to the feedback texture
    ///to be used with
    /// ```
    /// Self::draw()
    /// ```
    pub fn fill_from<S: Surface>(&self, surface: &S){
        // feedback_texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);
    
        surface.fill(&self.texture.as_surface(),glium::uniforms::MagnifySamplerFilter::Linear);
    }
}