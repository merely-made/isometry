// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// Mesocosm presentation over conatus-brick's product-neutral DDA.
//
// ROSTER_MEMBERS and ROSTER_PAIRS are injected ahead of this source from the
// Rust caps, so the two layouts cannot drift.

struct TraceCamera {
    origin: vec3<f32>,
    projection: u32,
    forward: vec3<f32>,
    far: f32,
    right: vec3<f32>,
    _right_pad: f32,
    up: vec3<f32>,
    _up_pad: f32,
    // How far along `forward` an orthographic ray slides to reach the slab's
    // world-vertical front wall: `wall.x * ndc.x + wall.y * ndc.y + wall.z`.
    // Exactly zero for a level section, whose near plane is that wall
    // already. See `SlabWall` on the Rust side.
    wall: vec4<f32>,
};

struct TraceParams {
    camera: TraceCamera,
    space: BrickTraceSpace,
    fog: vec4<f32>,
    look: vec4<f32>,
    // mode, soil, rock, unknown, sky, underground, section centre + clearing Y
    terrain: array<vec4<f32>, 7>,
    // Column-major world-to-clip for the depth join; identity when unused.
    clip_from_world: mat4x4<f32>,
    critter: CritterParams,
};

// The body remains a presentation-side SDF projection. Brick truth decides
// terrain occlusion; the nearer of the two traces owns a pixel.
struct CritterParams {
    bounds: vec4<f32>,
    tint_count: vec4<f32>,
    eyes: array<vec4<f32>, 2>,
    pairs: array<vec4<f32>, 192>,
};

// Every other body in frame: a background silhouette, so no eyes and the
// reduced capsule budget the roster cap buys. `count.x` members are live;
// the rest of the array is unread.
struct RosterPose {
    bounds: vec4<f32>,
    tint_count: vec4<f32>,
    pairs: array<vec4<f32>, ROSTER_PAIRS>,
};

struct RosterParams {
    count: vec4<u32>,
    poses: array<RosterPose, ROSTER_MEMBERS>,
};

@group(0) @binding(2) var<uniform> params: TraceParams;
@group(0) @binding(3) var<uniform> roster: RosterParams;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VsOut {
    var corners = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4(corners[index], 0.0, 1.0);
    out.ndc = corners[index];
    return out;
}

fn material_colour(material: u32) -> vec3<f32> {
    if (params.terrain[0].x > 0.5) {
        if (material == 3u) { return params.terrain[1].xyz; }
        if (material == 2u) { return params.terrain[2].xyz; }
        return params.terrain[3].xyz;
    }
    if (material == 3u) {
        return vec3(0.38, 0.24, 0.13); // soil
    }
    if (material == 2u) {
        return vec3(0.32, 0.34, 0.40); // rock
    }
    return vec3(0.66, 0.20, 0.72);
}

fn capsule_distance(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-6), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

// Smooth union of a running distance and the next capsule's.
fn blend_capsule(distance: f32, next: f32) -> f32 {
    let blend = 0.4;
    let amount = clamp(0.5 + 0.5 * (distance - next) / blend, 0.0, 1.0);
    return mix(distance, next, amount) - blend * amount * (1.0 - amount);
}

fn one_capsule(p: vec3<f32>, index: u32) -> f32 {
    let a = params.critter.pairs[index * 2u];
    let b = params.critter.pairs[index * 2u + 1u];
    return capsule_distance(p, a.xyz, b.xyz, (a.w + b.w) * 0.5);
}

fn critter_distance(p: vec3<f32>) -> f32 {
    let count = u32(params.critter.tint_count.w);
    var distance = one_capsule(p, 0u);
    for (var index = 1u; index < count; index = index + 1u) {
        distance = blend_capsule(distance, one_capsule(p, index));
    }
    return distance;
}

fn eye_distance(p: vec3<f32>) -> f32 {
    let left = length(p - params.critter.eyes[0].xyz) - params.critter.eyes[0].w;
    let right = length(p - params.critter.eyes[1].xyz) - params.critter.eyes[1].w;
    return min(left, right);
}

fn critter_normal(p: vec3<f32>) -> vec3<f32> {
    let epsilon = 0.08;
    return normalize(vec3(
        critter_distance(p + vec3(epsilon, 0.0, 0.0)) - critter_distance(p - vec3(epsilon, 0.0, 0.0)),
        critter_distance(p + vec3(0.0, epsilon, 0.0)) - critter_distance(p - vec3(0.0, epsilon, 0.0)),
        critter_distance(p + vec3(0.0, 0.0, epsilon)) - critter_distance(p - vec3(0.0, 0.0, epsilon)),
    ));
}

fn roster_capsule(member: u32, index: u32, p: vec3<f32>) -> f32 {
    let a = roster.poses[member].pairs[index * 2u];
    let b = roster.poses[member].pairs[index * 2u + 1u];
    return capsule_distance(p, a.xyz, b.xyz, (a.w + b.w) * 0.5);
}

fn roster_distance(member: u32, p: vec3<f32>) -> f32 {
    let count = u32(roster.poses[member].tint_count.w);
    var distance = roster_capsule(member, 0u, p);
    for (var index = 1u; index < count; index = index + 1u) {
        distance = blend_capsule(distance, roster_capsule(member, index, p));
    }
    return distance;
}

fn roster_normal(member: u32, p: vec3<f32>) -> vec3<f32> {
    let e = 0.08;
    return normalize(vec3(
        roster_distance(member, p + vec3(e, 0.0, 0.0)) - roster_distance(member, p - vec3(e, 0.0, 0.0)),
        roster_distance(member, p + vec3(0.0, e, 0.0)) - roster_distance(member, p - vec3(0.0, e, 0.0)),
        roster_distance(member, p + vec3(0.0, 0.0, e)) - roster_distance(member, p - vec3(0.0, 0.0, e)),
    ));
}

// Where a ray overlaps a pose's bounds sphere, as a [near, far] travel span.
// A span whose near exceeds its far is a miss.
fn bounds_span(eye: vec3<f32>, direction: vec3<f32>, bounds: vec4<f32>) -> vec2<f32> {
    let offset = eye - bounds.xyz;
    let projection = dot(offset, direction);
    let radius = dot(offset, offset) - bounds.w * bounds.w;
    if (projection > 0.0 && radius > 0.0) {
        return vec2(1.0, -1.0);
    }
    let discriminant = projection * projection - radius;
    if (discriminant < 0.0) {
        return vec2(1.0, -1.0);
    }
    let root = sqrt(discriminant);
    return vec2(
        max(-projection - root, 0.05),
        min(-projection + root, params.camera.far),
    );
}

fn trace_critter(eye: vec3<f32>, direction: vec3<f32>) -> f32 {
    if (params.critter.tint_count.w < 0.5) {
        return -1.0;
    }
    let span = bounds_span(eye, direction, params.critter.bounds);
    if (span.x > span.y) {
        return -1.0;
    }
    var travel = span.x;
    for (var step = 0; step < 48; step = step + 1) {
        let distance = critter_distance(eye + direction * travel);
        if (distance < 0.02) {
            return travel;
        }
        travel = travel + max(distance, 0.015);
        if (travel > span.y) {
            break;
        }
    }
    return -1.0;
}

struct RosterHit {
    t: f32,
    member: u32,
};

// The nearest roster body along the ray. Members are unordered, so a body
// whose bounds begin past the best hit so far is skipped without marching:
// the cost is one sphere test per member and a march only for contenders.
fn trace_roster(eye: vec3<f32>, direction: vec3<f32>) -> RosterHit {
    var best = RosterHit(-1.0, 0u);
    let members = min(roster.count.x, u32(ROSTER_MEMBERS));
    for (var member = 0u; member < members; member = member + 1u) {
        if (roster.poses[member].tint_count.w < 0.5) {
            continue;
        }
        let span = bounds_span(eye, direction, roster.poses[member].bounds);
        if (span.x > span.y) {
            continue;
        }
        if (best.t > 0.0 && span.x > best.t) {
            continue;
        }
        var travel = span.x;
        var found = false;
        for (var step = 0; step < 48; step = step + 1) {
            let distance = roster_distance(member, eye + direction * travel);
            if (distance < 0.02) {
                found = true;
                break;
            }
            travel = travel + max(distance, 0.015);
            if (travel > span.y) {
                break;
            }
        }
        if (found && (best.t < 0.0 || travel < best.t)) {
            best = RosterHit(travel, member);
        }
    }
    return best;
}

fn bayer(pixel: vec2<u32>) -> f32 {
    var values = array<f32, 16>(
        0.0, 8.0, 2.0, 10.0,
        12.0, 4.0, 14.0, 6.0,
        3.0, 11.0, 1.0, 9.0,
        15.0, 7.0, 13.0, 5.0,
    );
    return values[(pixel.y % 4u) * 4u + (pixel.x % 4u)] / 16.0 - 0.5;
}

// Steps in this many places along each channel, which is the starved-palette
// half of the retro look. Written down because `along_hue` below has to step
// the same ladder for a surface to sit on the same set of colours.
const GRADE_STEPS: f32 = 5.0;
const GRADE_SPAN: f32 = 4.0;

// The grade's step ladder, applied to each channel on its own.
//
// This is the shipped quantiser and stays exactly what it was: it is what the
// section's vertical faces, its bodies and its sky have always been drawn
// with, and every capture on record is of it.
fn steps_per_channel(colour: vec3<f32>, dither: f32) -> vec3<f32> {
    return floor(clamp(colour + vec3(dither), vec3(0.0), vec3(1.0)) * GRADE_STEPS) / GRADE_SPAN;
}

// The same ladder, climbed along the colour's own line instead of along the
// three axes separately.
//
// **Why a second quantiser exists at all.** Per-channel steps are not a
// palette; they are three independent ramps, and a colour crosses their
// boundaries at three different brightnesses. For the faces a level section
// shows that never mattered, because those faces are dark and their channels
// are far apart. It matters enormously for a face lit from above and then
// mixed toward a cool fog: soil, whose blue starts furthest from the fog
// colour, has its blue climb fastest, so partway through the fog the blue has
// taken a step the red and green have not and the face lands on a triple with
// more blue in it than soil has anywhere. Measured on the section's own two
// materials, the sequence a soil top face walked as fog took it was olive
// (64,64,0), grey, grey, then pink-violet (191,127,191) at the fifth fog
// band; rock reached blue-violet (64,64,127) at the first. None of those
// hues is in either material.
//
// Stepping the brightness and leaving the colour on its own line cannot do
// that: the output is the input's hue at one of the ladder's heights, so a
// step is always a step in light and never a step in hue. The look stays
// starved — the reachable colours are (material hue) x (fog band) x (step),
// which is a small, countable set, and it is closer to an actual palette than
// the RGB lattice it replaces.
//
// **Nearest rather than floor**, which is the one place this ladder differs.
// Flooring drops a face that is genuinely brighter back onto the same rung as
// the darker face below it — which for a top face erases the lighting that is
// the entire reason the face reads as a top at all.
fn steps_along_hue(colour: vec3<f32>, dither: f32) -> vec3<f32> {
    let value = max(max(colour.r, colour.g), colour.b);
    if (value < 1.0 / 512.0) {
        return vec3(0.0);
    }
    let stepped = round(clamp(value + dither, 0.0, 1.0) * GRADE_STEPS) / GRADE_SPAN;
    return min(colour * (stepped / value), vec3(1.0));
}

// The grade: fog, then the step ladder.
//
// `along_hue` picks which ladder. It is set for the ground seen from above and
// for nothing else — see the gate at the ground hit below, which is exactly
// false under a level section, so every capture the tree holds of one is
// unchanged to the byte.
fn grade(colour: vec3<f32>, t: f32, pixel: vec2<u32>, along_hue: bool) -> vec3<f32> {
    var fog = clamp((t / params.camera.far - params.fog.w) / max(1.0 - params.fog.w, 0.001), 0.0, 1.0);
    if (params.look.y > 0.5) {
        fog = floor(fog * params.look.y) / params.look.y;
    }
    var out = mix(colour, params.fog.xyz, fog);
    if (params.look.z > 0.5) {
        let dither = bayer(pixel) * params.look.x;
        if (along_hue) {
            out = steps_along_hue(out, dither);
        } else {
            out = steps_per_channel(out, dither);
        }
    }
    return out;
}

// Where a pixel's ray begins and which way it goes.
//
// **The orthographic branch begins on the section's wall, not on the near
// plane.** The two are the same thing for a level section and the advance is
// exactly zero there, so nothing about a level frame moves. Tilt the section
// and the near plane tilts with it, cutting the terrain on a slope; sliding
// each ray along its own direction onto the upright wall keeps the cut a
// vertical section of the world under any camera.
fn camera_ray(ndc: vec2<f32>) -> Ray {
    if (params.camera.projection == 1u) {
        let plane = params.camera.origin + params.camera.right * ndc.x + params.camera.up * ndc.y;
        let advance = dot(params.camera.wall.xyz, vec3(ndc.x, ndc.y, 1.0));
        return Ray(plane + params.camera.forward * advance, params.camera.forward);
    }
    return Ray(
        params.camera.origin,
        normalize(
            params.camera.forward + params.camera.right * ndc.x + params.camera.up * ndc.y,
        ),
    );
}

// One traced pixel: its graded colour and the world point that owns it,
// so the depth entry can place the same surface in the raster's clip space.
struct TraceSample {
    colour: vec4<f32>,
    world: vec3<f32>,
};

fn shade_body(normal: vec3<f32>, tint: vec3<f32>, direction: vec3<f32>) -> vec3<f32> {
    let sun = normalize(vec3(0.4, 0.8, 0.3));
    let light = clamp(dot(normal, sun), 0.0, 1.0);
    let rim = pow(1.0 - clamp(dot(normal, -direction), 0.0, 1.0), 2.0) * 0.35;
    return tint * (0.4 + 0.6 * light) + vec3(rim * 0.6);
}

fn trace_sample(in: VsOut) -> TraceSample {
    let ray = camera_ray(in.ndc);
    let hit = brick_dda(params.space, params.camera.far, ray.origin, ray.direction);
    let pixel = vec2<u32>(u32(in.pos.x), u32(in.pos.y));
    // Terrain owns the pixel from here on; a miss cuts off nothing.
    let terrain = select(1e30, hit.t, hit.found);

    var body_t = -1.0;
    var normal = vec3(0.0, 1.0, 0.0);
    var tint = vec3(0.0);
    var on_eye = false;

    let own = trace_critter(ray.origin, ray.direction);
    if (own > 0.0 && own < terrain) {
        let point = ray.origin + ray.direction * own;
        body_t = own;
        normal = critter_normal(point);
        tint = params.critter.tint_count.xyz;
        on_eye = eye_distance(point) < 0.05;
    }
    let other = trace_roster(ray.origin, ray.direction);
    if (other.t > 0.0 && other.t < terrain && (body_t < 0.0 || other.t < body_t)) {
        let point = ray.origin + ray.direction * other.t;
        body_t = other.t;
        normal = roster_normal(other.member, point);
        tint = roster.poses[other.member].tint_count.xyz;
        on_eye = false;
    }
    if (body_t > 0.0) {
        let point = ray.origin + ray.direction * body_t;
        var base = tint;
        if (on_eye) {
            base = base * 0.12;
        }
        let lit = shade_body(normal, base, ray.direction);
        return TraceSample(vec4(grade(lit, body_t, pixel, false), 1.0), point);
    }

    if (!hit.found) {
        var sky = mix(vec3(0.65, 0.72, 0.80), vec3(0.35, 0.45, 0.62), clamp(ray.direction.y * 3.0 + 0.3, 0.0, 1.0));
        if (params.terrain[0].x > 0.5) {
            let horizontal = vec3(params.camera.forward.x, 0.0, params.camera.forward.z);
            let denominator = dot(ray.direction, horizontal);
            var plane_y = ray.origin.y + ray.direction.y * params.camera.far;
            if (abs(denominator) > 1e-5) {
                let plane_t = dot(params.terrain[6].xyz - ray.origin, horizontal) / denominator;
                plane_y = ray.origin.y + ray.direction.y * plane_t;
            }
            sky = select(params.terrain[5].xyz, params.terrain[4].xyz, plane_y >= params.terrain[6].w);
        }
        let colour = select(
            grade(sky, params.camera.far, pixel, false),
            sky,
            params.terrain[0].x > 0.5,
        );
        return TraceSample(vec4(colour, 1.0), ray.origin + ray.direction * params.camera.far);
    }
    let sun = normalize(vec3(0.4, 0.8, 0.3));
    let light = 0.38 + 0.62 * max(0.0, dot(hit.normal, sun));
    // Where the section's own wall stands inside solid ground.
    //
    // The DDA seeds its normal to `+y` and returns that seed unchanged for a
    // ray that begins inside solid, so such a pixel is not a face at all: it
    // is the wall, and the terrain's cut on it. Asking the map what material
    // the ray's first voxel holds is the direct test — the same voxel the DDA
    // itself opens with — and it costs one texture load on the pixels that
    // already found ground. A ray starting outside the map reads as air here,
    // which is right: it entered through a face.
    let seeded = ray.origin + ray.direction * 0.0001;
    let in_wall = brick_material_at(params.space, vec3<i32>(floor(seeded))) != 0u;
    // The ground seen from above, and only that.
    //
    // **All three halves are load-bearing.** The face normal alone is not the
    // test, because of the seeded `+y` above. The downward ray is not enough
    // either: it is exactly false for a level section — whose forward.y is
    // zero — but a tilted section's whole wall passes it, and the wall would
    // then be drawn as a lit top face in the ground's own colour, which is
    // the flat screen-parallel slate that broke the first tilted capture.
    // `in_wall` is the half that names the wall as a wall. It leaves a level
    // frame exactly where it was, since the pixels it excludes there were
    // already excluded by the ray direction.
    let from_above = hit.normal.y > 0.5 && ray.direction.y < 0.0 && !in_wall;
    return TraceSample(
        vec4(grade(material_colour(hit.material) * light, hit.t, pixel, from_above), 1.0),
        ray.origin + ray.direction * hit.t,
    );
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    return trace_sample(in).colour;
}

struct DepthOut {
    @location(0) colour: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

// The depth join: the traced surface expressed in the raster tenant's own
// clip space, so hardware depth testing settles every pixel between them.
// A miss carries the far point, whose depth clamps to the far plane.
@fragment
fn fs_depth(in: VsOut) -> DepthOut {
    let sample = trace_sample(in);
    let clip = params.clip_from_world * vec4(sample.world, 1.0);
    var out: DepthOut;
    out.colour = sample.colour;
    out.depth = clamp(clip.z / max(clip.w, 1e-6), 0.0, 1.0);
    return out;
}
