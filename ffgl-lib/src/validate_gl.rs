pub(crate) struct TextureType {
    pub(crate) target: u32,
    pub(crate) binding: u32,
}

pub(crate) const TEXTURE_TYPES: [TextureType; 2] = [
    TextureType {
        target: gl::TEXTURE_1D,
        binding: gl::TEXTURE_BINDING_1D,
    },
    TextureType {
        target: gl::TEXTURE_2D,
        binding: gl::TEXTURE_BINDING_2D,
    },
    // Add other texture types here...
];

pub(crate) unsafe fn gl_reset(frame_data: &ffgl::ffgl::ProcessOpenGLStructTag) {
    let mut gl_int = 0;
    gl::UseProgram(0);

    let mut num_samplers = 0;
    gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut num_samplers);

    for texture_type in TEXTURE_TYPES.iter() {
        for sampler in 0..num_samplers {
            gl::ActiveTexture(gl::TEXTURE0 + sampler as u32);
            // Check if textures are unbound for the current texture unit.
            gl::GetIntegerv(texture_type.binding, &mut gl_int);
            // gl::BindTexture(texture_type.target, 0);
        }
    }

    gl::ActiveTexture(gl::TEXTURE0);

    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    gl::BindBuffer(gl::VERTEX_BINDING_BUFFER, 0);
    gl::BindVertexArray(0);
    gl::Disable(gl::BLEND);

    gl::BlendFunc(gl::ONE, gl::ZERO);

    // gl::BindVertexBuffer(0, 0, 0, 0);

    // gl::VertexArrayElementBuffer(vaobj, buffer)
    // gl::BindTextureUnit(0, 0);
    gl::BindFramebuffer(gl::FRAMEBUFFER, frame_data.HostFBO);
}

pub(crate) unsafe fn validate_viewport(viewport: &[i32; 4]) {
    let scissor_enabled = gl::IsEnabled(gl::SCISSOR_TEST);
    assert_eq!(scissor_enabled, gl::FALSE, "SCISSOR_TEST is enabled");

    let mut dims: [i32; 4] = [0; 4];
    gl::GetIntegerv(gl::VIEWPORT, &mut dims[0]);
    assert_eq!(&dims, viewport, "VIEWPORT wrong value: {dims:?}");
}
