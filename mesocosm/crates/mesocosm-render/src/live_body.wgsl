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
    @location(7) expression: vec4<f32>,
    @location(8) expression_tail: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) local_position: vec3<f32>,
    @location(3) expression: vec4<f32>,
    @location(4) expression_tail: vec4<f32>,
    @location(5) inspection_accent: f32,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
    let model = mat4x4<f32>(input.model_x, input.model_y, input.model_z, input.model_w);
    let world = model * vec4<f32>(input.position, 1.0);
    return VertexOut(
        frame.clip_from_world * world,
        input.color * input.tint.xyz,
        world.xyz,
        input.position,
        input.expression,
        input.expression_tail,
        input.tint.w,
    );
}

fn process_colour(index: u32) -> vec3<f32> {
    // Native-process palette, ordered exactly as Process::ALL. These marks
    // mean an allocated process, not a kingdom or a guessed material.
    switch index {
        case 0u: { return vec3<f32>(0.82, 0.30, 0.25); } // contract
        case 1u: { return vec3<f32>(0.82, 0.55, 0.20); } // intake
        case 2u: { return vec3<f32>(0.32, 0.72, 0.92); } // sense
        case 3u: { return vec3<f32>(0.34, 0.76, 0.30); } // fix
        default: { return vec3<f32>(0.78, 0.25, 0.72); } // secrete
    }
}

fn tissue_mark(local_position: vec3<f32>, expression: vec4<f32>, secrete: f32) -> vec3<f32> {
    // A regular voxel-aligned screen. The mosaic has no render-voxel
    // coordinates, so this is a density reading of caller-supplied fractions,
    // never a fabricated cell-to-voxel assignment.
    let cell = floor(local_position + vec3<f32>(0.001));
    let phase = fract((cell.x + 3.0 * cell.y + 5.0 * cell.z) / 16.0);
    let contract_end = expression.x;
    let intake_end = contract_end + expression.y;
    let sense_end = intake_end + expression.z;
    let fix_end = sense_end + expression.w;
    let secrete_end = fix_end + secrete;
    if (phase < contract_end) { return process_colour(0u); }
    if (phase < intake_end) { return process_colour(1u); }
    if (phase < sense_end) { return process_colour(2u); }
    if (phase < fix_end) { return process_colour(3u); }
    if (phase < secrete_end) { return process_colour(4u); }
    return vec3<f32>(-1.0);
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
    let mark = tissue_mark(input.local_position, input.expression, input.expression_tail.x);
    var colour = input.color;
    if (all(mark >= vec3<f32>(0.0))) {
        // Keep the face's structural light/dark contrast under the process
        // mark, so expression does not flatten a readable voxel body.
        let shade = max(max(input.color.x, input.color.y), input.color.z);
        // Retain the caller's body tint, including its deadened carrion tone.
        colour = mix(input.color, mark * shade, 0.72);
    }
    // Selection remains an accent. Replacing the material colour with amber
    // would conceal the tissue reading the inspection asked for.
    colour = mix(colour, vec3<f32>(1.0, 0.55, 0.10), input.inspection_accent * 0.28);
    // Lens FRAME_FORMAT is Rgba8Unorm: it stores display values directly.
    return vec4<f32>(srgb(colour), 1.0);
}
