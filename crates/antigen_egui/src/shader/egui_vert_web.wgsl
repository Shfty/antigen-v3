[[block]] struct UniformBuffer {
    u_screen_size: vec2<f32>;
};

[[group(0), binding(0)]] var<uniform> params: UniformBuffer;

fn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {
    var cutoff: vec3<f32> = step(srgb, vec3<f32>(10.31475));
    var lower: vec3<f32> = srgb / vec3<f32>(3294.6);
    var higher: vec3<f32> = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));
    return mix(higher, lower, cutoff);
}

struct MyOutputs {
  [[builtin(position)]] v_position: vec4<f32>;
  [[location(0)]] v_tex_coord: vec2<f32>;
  [[location(1)]] v_color: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    [[builtin(vertex_index)]] in_vertex_index: u32,
    [[location(0)]] a_pos: vec2<f32>,
    [[location(1)]] a_tex_coord: vec2<f32>,
    [[location(2)]] a_color: u32,
) -> MyOutputs {
    // [u8; 4] SRGB as u32 -> [r, g, b, a]
    var color: vec4<u32> = vec4<u32>(
        a_color & 255u,
        (a_color >> 8u) & 255u,
        (a_color >> 16u) & 255u,
        (a_color >> 24u) & 255u
    );

    return MyOutputs(
        vec4<f32>(
            2.0 * a_pos.x / params.u_screen_size.x - 1.0,
            1.0 - 2.0 * a_pos.y / params.u_screen_size.y,
            0.0,
            1.0
        ),
        a_tex_coord,
        vec4<f32>(color) / 255.0
    );
}
