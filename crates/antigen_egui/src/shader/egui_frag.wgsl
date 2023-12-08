[[group(0), binding(1)]] var s_texture: sampler;
[[group(1), binding(0)]] var t_texture: texture_2d<f32>;

[[stage(fragment)]]
fn main(
    [[location(0)]] v_tex_coord: vec2<f32>,
    [[location(1)]] v_color: vec4<f32>,
) -> [[location(0)]] vec4<f32> {
    return v_color * textureSample(
        t_texture,
        s_texture,
        v_tex_coord
    );
}
