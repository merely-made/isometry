// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// The grade: the soul as a data block. Reads the march's colour + distance,
// applies fog, then either quantises against a palette LUT under an ordered
// dither (retro) or passes the ramp through smooth (clay). One shader, every
// look a uniform away.

struct GradeParams {
    // Fog colour the world recedes into.
    fog: vec3<f32>,
    // Distance (0..1 of far) where fog begins.
    fog_start: f32,
    // Palette entries in use; 0 disables quantisation entirely (clay).
    palette_len: u32,
    // Ordered-dither strength in colour units; 0 disables.
    dither: f32,
    // Fog band count; 0 = smooth fog, n = quantised fog steps (retro).
    fog_bands: f32,
    _pad: f32,
};

@group(0) @binding(0) var frame: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;
@group(0) @binding(2) var palette: texture_2d<f32>;
@group(0) @binding(3) var<uniform> params: GradeParams;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VsOut {
    var corners = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4(corners[index], 0.0, 1.0);
    out.uv = corners[index] * vec2(0.5, -0.5) + vec2(0.5, 0.5);
    return out;
}

// Bayer 4x4, normalised to -0.5..0.5.
fn bayer(px: vec2<u32>) -> f32 {
    var m = array<f32, 16>(
        0.0, 8.0, 2.0, 10.0,
        12.0, 4.0, 14.0, 6.0,
        3.0, 11.0, 1.0, 9.0,
        15.0, 7.0, 13.0, 5.0,
    );
    let i = (px.y % 4u) * 4u + (px.x % 4u);
    return m[i] / 16.0 - 0.5;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let sample = textureSampleLevel(frame, frame_sampler, in.uv, 0.0);
    var color = sample.rgb;
    var depth = sample.a;

    // Fog, optionally banded: quantised fog is half the Comanche look.
    var fog_t = clamp((depth - params.fog_start) / max(1.0 - params.fog_start, 0.001), 0.0, 1.0);
    if (params.fog_bands > 0.5) {
        fog_t = floor(fog_t * params.fog_bands) / params.fog_bands;
    }
    color = mix(color, params.fog, fog_t);

    if (params.palette_len == 0u) {
        return vec4(color, 1.0);
    }

    // Ordered dither then nearest palette entry. The palette is a 1D LUT the
    // worldgen painted; nearest-search keeps the shader free of any palette
    // assumptions.
    let px = vec2<u32>(u32(in.pos.x), u32(in.pos.y));
    let dithered = clamp(color + vec3(bayer(px) * params.dither), vec3(0.0), vec3(1.0));

    var best = 0u;
    var best_d = 1e9;
    for (var i = 0u; i < params.palette_len; i = i + 1u) {
        let entry = textureLoad(palette, vec2<i32>(i32(i), 0), 0).rgb;
        let d = dot(dithered - entry, dithered - entry);
        if (d < best_d) {
            best_d = d;
            best = i;
        }
    }
    return vec4(textureLoad(palette, vec2<i32>(i32(best), 0), 0).rgb, 1.0);
}
