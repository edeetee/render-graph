use std::cell::Ref;

use nannou::{prelude::*, wgpu::{TextureView, CommandEncoder, VertexBufferLayout, RenderPipeline, PipelineLayout, ShaderModule, FragmentState, TextureBuilder, ToTextureView, TextureUsages, TextureSampleType, ColorWrites, PrimitiveState, VertexAttribute}};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex{
    pub pos: [f32; 2]
}

pub struct ViewModel {
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    feedback_texture_view: wgpu::TextureView,
    num_vertices: usize,
}

impl ViewModel{
    pub fn new(window: &Window, vertices: &[Vertex]) -> ViewModel {
        let device = window.device();
        let samples = window.msaa_samples();

        // Load shader modules.
        let vs_desc = wgpu::include_wgsl!("shaders/vert.wgsl");
        let fs_desc = wgpu::include_wgsl!("shaders/frag.wgsl");
        let vs_mod = device.create_shader_module(&vs_desc);
        let fs_mod = device.create_shader_module(&fs_desc);

        let window_size = window.inner_size_pixels();

        let feedback_texture = TextureBuilder::new()
            .format(Frame::TEXTURE_FORMAT)
            .usage(TextureUsages::all())
            .sample_count(1)
            .size([window_size.0, window_size.1])
            .build(device);

        let feedback_texture_view = feedback_texture.view().build();

        // Create the vertex buffer.
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertices"),
            contents: to_bytes(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (bind_group_layout, bind_group) = make_bind_group_and_layout_from_texture(device, &feedback_texture_view);

        let pipeline_layout = wgpu::create_pipeline_layout(device, Some("Feedback layout"), &[&bind_group_layout], &[]);
        
        let render_pipeline = make_pipeline(window, &pipeline_layout, &vs_mod, &fs_mod);

        ViewModel {
            bind_group,
            vertex_buffer,
            render_pipeline,
            num_vertices: vertices.len(),
            feedback_texture_view
        }
    }

    pub fn render_to(self: &ViewModel, encoder: &mut CommandEncoder, attachment: &TextureView){

        // The render pass can be thought of a single large command consisting of sub commands. Here we
        // begin a render pass that outputs to the frame's texture. Then we add sub-commands for
        // setting the bind group, render pipeline, vertex buffers and then finally drawing.
        let mut render_pass = wgpu::RenderPassBuilder::new()
            .color_attachment(attachment, |color| color)
            .color_attachment(&self.feedback_texture_view, |color| color)
            .begin(encoder);
        
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    
        // We want to draw the whole range of vertices, and we're only drawing one instance of them.
        let vertex_range = 0..self.num_vertices as u32;
        let instance_range = 0..1;
        render_pass.draw(vertex_range, instance_range);
    }
}


fn make_bind_group_and_layout_from_texture(
    device: &wgpu::Device,
    texture: &wgpu::TextureView
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {

    let multisampled = texture.base_mip_level() != 0;

    let sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
    let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
    let sampler = device.create_sampler(&sampler_desc);

    let layout = wgpu::BindGroupLayoutBuilder::new()
        .texture(
            wgpu::ShaderStages::FRAGMENT,
            multisampled,
            texture.dimension(),
            texture.sample_type(),
        )
        .sampler(wgpu::ShaderStages::FRAGMENT, sampler_filtering)
        .build(device);

    let bind = wgpu::BindGroupBuilder::new()
        .texture_view(texture)
        .sampler(&sampler)
        .build(device, &layout);

    (layout, bind)
}

fn make_pipeline(window: &Window, pipeline_layout: &PipelineLayout, vs_mod: &ShaderModule, fs_mod: &ShaderModule) -> RenderPipeline{
    let format = Frame::TEXTURE_FORMAT;
    let sample_count = window.msaa_samples();

    // let builder = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
    //     .fragment_shader(&fs_mod)
    //     .color_format(format)
    //     .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
    //     .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
    //     .sample_count(sample_count);

    // let custom_builder = wgpu::RenderPipelineBuilder{

    // }

    let vertex_buffers = [
        vertex_buffer_layout::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x2])
    ];

    let targets = [
        wgpu::ColorTargetState{
            format,
            blend: None,
            write_mask: ColorWrites::all()
        },
        wgpu::ColorTargetState{
            format,
            blend: None,
            write_mask: ColorWrites::all()
        }
    ];

    let desc = wgpu::RenderPipelineDescriptor{
        label: Some("Feedback pipeline"),
        layout: Some(pipeline_layout),

        vertex: wgpu::VertexState { 
            module: vs_mod, 
            entry_point: "main", 
            buffers: &vertex_buffers
        },
        primitive: PrimitiveState{
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },

        fragment: Some(wgpu::FragmentState{
            module: fs_mod,
            entry_point: "main",
            targets: &targets
        }),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            ..Default::default()
        }
    };

    window.device().create_render_pipeline(&desc)
}

// See the `nannou::wgpu::bytes` documentation for why this is necessary.
fn to_bytes<T>(data: &[T]) -> &[u8]
    where T: Copy + Sized
{
    unsafe { wgpu::bytes::from_slice(data) }
}

fn vertex_buffer_layout<V>(attrs: &'static [VertexAttribute]) -> wgpu::VertexBufferLayout<'static>{ 
    let array_stride = std::mem::size_of::<V>() as wgpu::BufferAddress;
    let step_mode = wgpu::VertexStepMode::Vertex;

    wgpu::VertexBufferLayout {
        array_stride,
        step_mode,
        attributes: attrs,
    }
}