var VERTICES: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-0.5, -0.5),
    vec2<f32>(0.5, -0.5),
    vec2<f32>(-0.5, 0.5),
    vec2<f32>(0.5, 0.5),
);

[[stage(vertex)]]
fn vs_main(
    [[builtin(vertex_index)]] in_vertex_index: u32,
    [[location(0)]] instance_pos: vec2<f32>,
    [[location(1)]] instance_size: vec2<f32>,
) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(VERTICES[in_vertex_index] * instance_size + instance_pos, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}