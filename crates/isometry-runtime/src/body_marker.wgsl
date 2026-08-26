struct Params {
    target_origin: vec4<f32>,
    basis_x_y: vec4<f32>,
    basis_z_marker: vec4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<uniform> params: Params;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) slot: u32,
) -> VsOut {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>( 0.5, -0.5),
        vec2<f32>(-0.5,  0.5),
        vec2<f32>(-0.5,  0.5),
        vec2<f32>( 0.5, -0.5),
        vec2<f32>( 0.5,  0.5),
    );
    let body = positions[slot];
    var out: VsOut;
    if body.w <= 0.0 {
        out.position = vec4<f32>(2.0, 2.0, 0.0, 1.0);
        out.color = vec4<f32>(0.0);
        return out;
    }

    let basis_x = params.basis_x_y.xy;
    let basis_y = params.basis_x_y.zw;
    let basis_z = params.basis_z_marker.xy;
    let marker = params.basis_z_marker.zw;
    let center = params.target_origin.zw
        + body.x * basis_x
        + body.y * basis_y
        + body.z * basis_z;
    let pixel = center + corners[vertex_index] * marker;
    let viewport = params.target_origin.xy;
    out.position = vec4<f32>(
        (pixel.x / viewport.x) * 2.0 - 1.0,
        1.0 - (pixel.y / viewport.y) * 2.0,
        0.0,
        1.0,
    );
    out.color = params.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
