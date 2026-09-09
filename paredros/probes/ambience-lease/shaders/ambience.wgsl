// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// The wing's ambience kernels, and the adapter that hands their output
// to renderling.
//
// Motes drift on a curl-noise-ish field with a gentle updraft, wrapped
// into a box. Padded 3D throughout, and z is load-bearing here in a way
// it never was for a 2D canvas: the camera looks at the cloud from an
// angle, so depth is visible truth rather than a reserved lane.

struct Params {
    n: u32,
    dt: f32,
    time: f32,
    swirl: f32,
    updraft: f32,
    extent: f32,
    // Where renderling's transforms start in its slab, and how many u32
    // words each one occupies. The adapter writes translations there.
    transform_base: u32,
    transform_stride: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> positions: array<vec4f>;
@group(0) @binding(2) var<storage, read_write> velocities: array<vec4f>;
// renderling's geometry slab, as raw words. The adapter writes floats
// into it by bitcast: the slab is a u32 array by construction, and a
// TransformDescriptor begins with its translation.
@group(0) @binding(3) var<storage, read_write> slab: array<u32>;

fn hash3(p: vec3f) -> vec3f {
    let q = vec3f(
        dot(p, vec3f(127.1, 311.7, 74.7)),
        dot(p, vec3f(269.5, 183.3, 246.1)),
        dot(p, vec3f(113.5, 271.9, 124.6)),
    );
    return fract(sin(q) * 43758.5453) * 2.0 - 1.0;
}

// A cheap divergence-light flow: swirl about the vertical axis plus a
// hashed jitter that varies slowly in space and time.
@compute @workgroup_size(256)
fn drift(@builtin(global_invocation_id) gid: vec3u) {
    let i = gid.x;
    if (i >= params.n) {
        return;
    }
    let p = positions[i].xyz;
    var v = velocities[i].xyz;

    // Swirl about the vertical axis, normalized so the tangential speed
    // does not grow with radius: an unnormalized radial term flings the
    // outer motes into the wall and the cloud collapses onto its own
    // boundary, which is what the first run of this probe produced.
    let radial = vec3f(-p.z, 0.0, p.x);
    let swirl = select(
        vec3f(0.0),
        normalize(radial) * params.swirl,
        length(radial) > 1e-3,
    );
    let turbulence = hash3(floor(p * 0.05) + vec3f(params.time * 0.15));
    let accel = swirl + turbulence * 4.0 + vec3f(0.0, params.updraft, 0.0);

    v = (v + accel * params.dt) * 0.96;
    var np = p + v * params.dt;

    // Wrap every axis: the box is a torus, so the cloud is perpetual
    // without a spawner and fills its volume instead of piling on a
    // clamped face.
    let e = params.extent;
    let span = 2.0 * e;
    np = np - span * floor((np + vec3f(e)) / span);

    positions[i] = vec4f(np, positions[i].w);
    velocities[i] = vec4f(v, 0.0);
}

// The lease adapter: resident positions to renderling transforms,
// device-local. renderling addresses geometry through a slab rather
// than through bindable buffers, so a consumer with a different memory
// model reads the lease by *writing into its own* storage. This kernel
// is the whole cost of that mismatch.
@compute @workgroup_size(256)
fn publish(@builtin(global_invocation_id) gid: vec3u) {
    let i = gid.x;
    if (i >= params.n) {
        return;
    }
    let p = positions[i].xyz;
    let at = params.transform_base + i * params.transform_stride;
    slab[at] = bitcast<u32>(p.x);
    slab[at + 1u] = bitcast<u32>(p.y);
    slab[at + 2u] = bitcast<u32>(p.z);
}
