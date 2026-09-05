struct Frame {
    clip_from_world: mat4x4<f32>,
    slab_normal_min: vec4<f32>,
    slab_max_enabled: vec4<f32>,
    bounds_min: vec4<f32>,
    bounds_max: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> frame: Frame;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) model_x: vec4<f32>,
    @location(3) model_y: vec4<f32>,
    @location(4) model_z: vec4<f32>,
    @location(5) model_w: vec4<f32>,
    @location(6) tint: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    let model = mat4x4<f32>(input.model_x, input.model_y, input.model_z, input.model_w);
    let world = model * vec4<f32>(input.position, 1.0);
    return VertexOut(frame.clip_from_world * world, input.color * input.tint.xyz, world.xyz);
}

fn srgb(linear: vec3<f32>) -> vec3<f32> {
    let low = linear * 12.92;
    let high = 1.055 * pow(max(linear, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, linear <= vec3<f32>(0.0031308));
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    if (frame.slab_max_enabled.w > 0.5) {
        let slab = dot(frame.slab_normal_min.xyz, input.world_position);
        if (slab < frame.slab_normal_min.w || slab > frame.slab_max_enabled.x) {
            discard;
        }
    }
    if (frame.bounds_min.w > 0.5 &&
        (any(input.world_position < frame.bounds_min.xyz) ||
         any(input.world_position > frame.bounds_max.xyz))) {
        discard;
    }
    // Lens FRAME_FORMAT is Rgba8Unorm: it stores display values directly.
    return vec4<f32>(srgb(input.color), 1.0);
}
