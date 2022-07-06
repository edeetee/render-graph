struct FragmentOutput {
    [[location(0)]] feedback_color: vec4<f32>;
    [[location(1)]] out_color: vec4<f32>;
};

[[group(0), binding(0)]]
var tex: texture_2d<f32>;
[[group(0), binding(1)]]
var tex_sampler: sampler;

[[stage(fragment)]]
fn main([[location(0)]] tex_coords: vec2<f32>) -> FragmentOutput {
    let feedbackPrev: vec4<f32> = textureSample(tex, tex_sampler, tex_coords);

    let feedbackNew = vec4<f32>(1.0, 0.0, 0.0, 1.0);

    return FragmentOutput(feedbackNew, feedbackNew);
}
