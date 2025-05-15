/* type-----------------------------------------------------------------------*/
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
}

/* constant-------------------------------------------------------------------*/
// full screen vertex clip + texture coordinates
// large triangle optimization from
// https://webgpufundamentals.org/webgpu/lessons/webgpu-large-triangle-to-cover-clip-space.html
const FULL_SCREEN_QUAD = array<vec4<f32>, 3>(
    vec4(-1.0, -1.0,  0.0,  1.0),
    vec4( 3.0, -1.0,  2.0,  1.0),
    vec4(-1.0,  3.0,  0.0, -1.0),
);

/* uniform--------------------------------------------------------------------*/
@group(0) @binding(0) var<uniform> param: Param;
@group(0) @binding(1) var<uniform> stat: Stat;

/* buffer---------------------------------------------------------------------*/
@group(0) @binding(2) var<storage, read_write> frame: array<vec3<f32>>;

/* function-------------------------------------------------------------------*/
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(FULL_SCREEN_QUAD[index].xy, 0.0, 1.0),
        FULL_SCREEN_QUAD[index].zw,
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let idx = u32(in.tex_coord.x * f32(param.window_size.x)) +
        u32(in.tex_coord.y * f32(param.window_size.y)) * param.window_size.x + 1;
    return vec4<f32>(frame[idx] / f32(stat.frame_counter + 1), 1.0);
}
