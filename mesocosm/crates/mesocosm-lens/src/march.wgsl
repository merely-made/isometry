// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// The march: per-pixel raymarch over a heightfield, the Voxel Space lineage
// generalised from per-column spans to free pitch. Inputs are two images and
// a camera; outputs are colour (rgb) and normalised hit distance (a), which
// the grade pass reads as its fog and depth term.

struct MarchParams {
    // Camera position in map units (x, height, z).
    eye: vec3<f32>,
    yaw: f32,
    pitch: f32,
    // Horizontal field of view in radians.
    fov: f32,
    // Furthest march distance, map units.
    far: f32,
    // Map side length in texels (square maps).
    map_side: f32,
};

// A capsule-chain body: up to 16 segments, each xyz + radius, smooth-
// unioned. seg_count = 0 means no critter in frame. The bounding sphere
// lets rays that miss skip the whole trace.
// vec4-only on purpose: vec3 uniform packing is where layouts quietly
// disagree between host and shader, and this struct already lost one debug
// session to a perfectly round artefact.
struct CritterParams {
    // xyz = bounds centre, w = bounds radius.
    bounds: vec4<f32>,
    // xyz = tint, w = capsule count.
    tint_count: vec4<f32>,
    // Two eye spheres: xyz centre, w radius.
    eyes: array<vec4<f32>, 2>,
    // Capsule j is pairs[2j] (a.xyz, ra) to pairs[2j+1] (b.xyz, rb).
    // POSE_PAIRS is injected from MAX_CAPSULES rather than written twice.
    pairs: array<vec4<f32>, POSE_PAIRS>,
};

@group(0) @binding(0) var height_map: texture_2d<f32>;
@group(0) @binding(1) var color_map: texture_2d<f32>;
@group(0) @binding(2) var map_sampler: sampler;
@group(0) @binding(3) var<uniform> params: MarchParams;
@group(0) @binding(4) var<uniform> critter: CritterParams;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VsOut {
    // One fullscreen triangle.
    var corners = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4(corners[index], 0.0, 1.0);
    out.ndc = corners[index];
    return out;
}

fn terrain_height(at: vec2<f32>) -> f32 {
    let uv = at / params.map_side;
    return textureSampleLevel(height_map, map_sampler, uv, 0.0).r * 255.0;
}

fn terrain_color(at: vec2<f32>) -> vec3<f32> {
    let uv = at / params.map_side;
    return textureSampleLevel(color_map, map_sampler, uv, 0.0).rgb;
}

fn capsule_distance(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-6), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

// The body's distance field: capsules between consecutive segments, blended
// by a smooth minimum. The blend IS the biology reading: parts flow into
// each other the way kleptoplasty means them to.
fn one_capsule(p: vec3<f32>, j: u32) -> f32 {
    let a = critter.pairs[2u * j];
    let b = critter.pairs[2u * j + 1u];
    return capsule_distance(p, a.xyz, b.xyz, (a.w + b.w) * 0.5);
}

fn critter_distance(p: vec3<f32>) -> f32 {
    let count = u32(critter.tint_count.w);
    // Seeded from the first capsule, never from a big sentinel: mix(1e9,
    // cd, 1.0) computes 1e9 + (cd - 1e9) at f32 precision, which cancels
    // catastrophically to ~0 and turns the whole field into a hit. The
    // giant-sphere artefact that cost this probe an evening was exactly
    // that: a distance field reading zero everywhere inside the bounds.
    var d = one_capsule(p, 0u);
    // Small against the segment radii: enough to fillet the joints, not
    // enough to inflate the body into a ball.
    let k = 0.4;
    for (var j = 1u; j < count; j = j + 1u) {
        let cd = one_capsule(p, j);
        // Polynomial smooth min.
        let h = clamp(0.5 + 0.5 * (d - cd) / k, 0.0, 1.0);
        d = mix(d, cd, h) - k * h * (1.0 - h);
    }
    return d;
}

fn eye_distance(p: vec3<f32>) -> f32 {
    let a = length(p - critter.eyes[0].xyz) - critter.eyes[0].w;
    let b = length(p - critter.eyes[1].xyz) - critter.eyes[1].w;
    return min(a, b);
}

fn critter_normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.08;
    return normalize(vec3(
        critter_distance(p + vec3(e, 0.0, 0.0)) - critter_distance(p - vec3(e, 0.0, 0.0)),
        critter_distance(p + vec3(0.0, e, 0.0)) - critter_distance(p - vec3(0.0, e, 0.0)),
        critter_distance(p + vec3(0.0, 0.0, e)) - critter_distance(p - vec3(0.0, 0.0, e)),
    ));
}

// Sphere-trace the body along the ray; returns hit distance or -1.
fn trace_critter(eye: vec3<f32>, dir: vec3<f32>) -> f32 {
    if (critter.tint_count.w < 0.5) {
        return -1.0;
    }
    // Ray-sphere prefilter against the bounds.
    let oc = eye - critter.bounds.xyz;
    let b = dot(oc, dir);
    let c = dot(oc, oc) - critter.bounds.w * critter.bounds.w;
    if (b > 0.0 && c > 0.0) {
        return -1.0;
    }
    let disc = b * b - c;
    if (disc < 0.0) {
        return -1.0;
    }
    var t = max(-b - sqrt(disc), 0.05);
    let t_exit = -b + sqrt(disc);
    for (var i = 0; i < 48; i = i + 1) {
        let d = critter_distance(eye + dir * t);
        if (d < 0.02) {
            return t;
        }
        t = t + max(d, 0.015);
        if (t > t_exit) {
            break;
        }
    }
    return -1.0;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    // Ray direction from yaw, pitch, and the pixel's ndc offset.
    let half_fov = params.fov * 0.5;
    let ray_yaw = params.yaw + in.ndc.x * half_fov;
    // Vertical fov scales by the frame's implied aspect through ndc.y.
    let ray_pitch = params.pitch + in.ndc.y * half_fov * 0.5625;
    let dir = vec3(
        sin(ray_yaw) * cos(ray_pitch),
        sin(ray_pitch),
        cos(ray_yaw) * cos(ray_pitch),
    );

    // March with a growing step: fine nearby where detail reads, coarse far
    // away where fog eats it anyway. On a hit, one binary refinement pass
    // tightens the surface so nearby terrain does not stairstep.
    // Budget check: 0.35-unit steps to ~12 units, then 3% geometric growth
    // reaches past 2000 units well inside 300 iterations, so params.far is
    // the binding limit rather than the loop. An earlier cut grew at 1.2%
    // and silently topped out near 155 units, which painted every distant
    // vista as sky.
    var t = 0.5;
    var hit = false;
    var at = params.eye;
    for (var i = 0; i < 300; i = i + 1) {
        at = params.eye + dir * t;
        if (terrain_height(at.xz) > at.y) {
            hit = true;
            break;
        }
        if (t > params.far) {
            break;
        }
        t = t + max(0.35, t * 0.03);
    }

    let body_t = trace_critter(params.eye, dir);
    if (body_t > 0.0 && (!hit || body_t < t)) {
        let p = params.eye + dir * body_t;
        let normal = critter_normal(p);
        let sun = normalize(vec3(0.4, 0.8, 0.3));
        let light = clamp(dot(normal, sun), 0.0, 1.0);
        // A rim term so the silhouette pops off the terrain, which is most
        // of small-body legibility under a starved palette.
        let rim = pow(1.0 - clamp(dot(normal, -dir), 0.0, 1.0), 2.0) * 0.35;
        var base = critter.tint_count.xyz;
        // An eye is a dark dot wherever the eye field comes closer than the
        // body surface: the cheapest face a creature can have.
        if (eye_distance(p) < 0.05) {
            base = base * 0.12;
        }
        let lit = base * (0.4 + 0.6 * light) + vec3(rim * 0.6);
        return vec4(lit, clamp(body_t / params.far, 0.0, 0.999));
    }

    if (!hit) {
        // Sky: a vertical gradient the grade pass tints. Distance 1.0 marks
        // "no terrain" for the fog term.
        let sky = mix(vec3(0.65, 0.72, 0.80), vec3(0.35, 0.45, 0.62), clamp(ray_pitch * 3.0 + 0.3, 0.0, 1.0));
        return vec4(sky, 1.0);
    }

    var lo = max(t - max(0.35, t * 0.03), 0.0);
    var hi = t;
    for (var i = 0; i < 6; i = i + 1) {
        let mid = (lo + hi) * 0.5;
        let p = params.eye + dir * mid;
        if (terrain_height(p.xz) > p.y) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    at = params.eye + dir * hi;

    // Slope shading from the heightfield gradient: cheap, and the grade
    // pass's ramp decides how harsh it reads.
    let e = 1.5;
    let dx = terrain_height(at.xz + vec2(e, 0.0)) - terrain_height(at.xz - vec2(e, 0.0));
    let dz = terrain_height(at.xz + vec2(0.0, e)) - terrain_height(at.xz - vec2(0.0, e));
    let normal = normalize(vec3(-dx, 2.0 * e, -dz));
    let sun = normalize(vec3(0.4, 0.8, 0.3));
    let light = clamp(dot(normal, sun), 0.0, 1.0);

    let base = terrain_color(at.xz);
    let lit = base * (0.45 + 0.55 * light);
    return vec4(lit, clamp(hi / params.far, 0.0, 0.999));
}
