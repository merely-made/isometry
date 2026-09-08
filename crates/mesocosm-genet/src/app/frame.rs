// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Host {
    pub(super) fn frame(&mut self, event_loop: &ActiveEventLoop) {
        // Point the driver's per-body window at whoever is being followed
        // before this frame's ticks run, so the accounts on the dev tile cover
        // them (DT2). Nothing outside `--dev` calls it.
        self.watch_followed();
        let replay_done = self.advance();
        let body_review = self.grafting.open || self.creator.is_some();
        // What the world answered this frame, as the describe-strings an
        // `assert event` matches. (DT4)
        self.note_outcomes();
        // A follow target that stopped being alive is reported and dropped
        // here, before anything reads the centre.
        self.update_follow();
        self.update_inspection();

        // Presentation reads of the stepped world, taken before the device is
        // borrowed: the section follows the critter, the HUD backdrop wants
        // the meshed scene, and neither is world state.
        //
        // **The follow centre, not the played position**: `--dev` may have put
        // the camera on somebody else, and `follow_at` falls back to the
        // played body when it has not.
        let at = self.follow_at();
        // Where the *played* body stands, which is a different question and
        // the one the HUD backdrop's own scene item asks.
        let played_at = self.runtime.world().position().unwrap_or(at);
        let half = section::half_height_or_default(self.config.slab_half_height);
        let mut centre = self.habitat.as_ref().map_or_else(
            || section::centre_on(at, self.pan, half, self.config.camera),
            |habitat| {
                let mut centre =
                    [0, 1, 2].map(|i| (habitat.bounds.min[i] + habitat.bounds.max[i]) as f32 * 0.5);
                let [right, up, _] =
                    section::camera_basis(self.config.camera, Some(self.config.terrarium_pitch));
                for i in 0..3 {
                    centre[i] += right[i] * self.pan.x + up[i] * self.pan.y;
                }
                centre
            },
        );
        if let (Some(habitat), Some(gpu)) = (&self.habitat, &mut self.gpu) {
            gpu.section.configure_terrarium(
                habitat,
                self.config.terrarium_pitch,
                self.config.cutaway,
                played_at,
            );
        }
        if let (Some(habitat), Some(window)) = (&self.habitat, &self.window) {
            let exposed = self.config.cutaway == section::Cutaway::Always
                || (self.config.cutaway == section::Cutaway::Occupied
                    && section::terrarium_occupied(habitat, played_at));
            window.set_title(&format!(
                "Mesocosm | clearing and burrow | {} | cutaway {} ({}) | Z/V turn | H graft tissue | O express discovery",
                self.config.camera.name(),
                self.config.cutaway.name(),
                if exposed { "open" } else { "closed" }
            ));
        }
        let body_scale = if self.habitat.is_some() {
            section::TERRARIUM_BODY_SCALE
        } else {
            1.0
        };
        let mut view_half = half;
        let mut preview_depth = section::SLAB_DEPTH;
        if body_review {
            let frame = self
                .gpu
                .as_ref()
                .map(|gpu| (gpu.config.width, gpu.config.height))
                .unwrap_or((960, 540));
            if let Some((focused, fitted, depth)) = grafting::framing(
                self.creator
                    .as_ref()
                    .map(creator::Creator::world)
                    .unwrap_or_else(|| {
                        self.grafting
                            .preview
                            .as_deref()
                            .unwrap_or(self.runtime.world())
                    }),
                frame,
                self.config.camera,
                self.habitat.as_ref().map(|_| self.config.terrarium_pitch),
                body_scale,
            ) {
                centre = focused;
                view_half = fitted;
                preview_depth = depth;
            }
        }
        if let Some(gpu) = &mut self.gpu {
            gpu.section.set_half_height(view_half);
            gpu.section.set_body_preview(body_review, preview_depth);
            gpu.section.configure_bodies(
                if body_review {
                    section::BodyMode::Voxels
                } else {
                    self.config.body_mode
                },
                self.config.body_budget,
            );
        }
        // Only the section sees this disposable candidate. HUD, receipts and
        // all simulation work keep reading the authoritative runtime world.
        let tint = self
            .creator
            .as_ref()
            .map(creator::Creator::world)
            .unwrap_or(self.runtime.world())
            .controlled()
            .map_or([0.42, 0.62, 0.46], |organism| look_of(organism).0);
        let (pose, dropped) = match section::pose_of_scaled(
            self.creator
                .as_ref()
                .map(creator::Creator::world)
                .unwrap_or_else(|| {
                    self.grafting
                        .preview
                        .as_deref()
                        .unwrap_or(self.runtime.world())
                }),
            tint,
            body_scale,
            self.habitat.is_some(),
        ) {
            Some((pose, dropped)) => (Some(pose), dropped),
            None => (None, 0),
        };
        self.body_capsules_dropped = dropped;
        // An ant farm needs the ants: everything else alive in the slab the
        // camera already cuts, posed the same way and coloured by its own
        // guise. Nothing here reaches an intent.
        let roster = self
            .gpu
            .as_ref()
            .filter(|_| !body_review)
            .map(|gpu| gpu.section.slab_window(centre))
            .map(|window| {
                section::roster_of_scaled(
                    self.runtime.world(),
                    window,
                    |organism| look_of(organism).0,
                    body_scale,
                    self.habitat.is_some(),
                )
            })
            .unwrap_or_default();
        let scene = self.scene();
        let dirty = self.runtime.drain_ground_dirty();
        let steps = self.steps;
        // What the world said about the intents this frame fed it. Refusals
        // were polite inside `World::apply` and silent outside it until the
        // cambium lane landed; this is the whole of their route to a screen.
        let outcomes = self.runtime.last_outcomes().to_vec();
        // The bounded ecology windows, read off the driver that reduced them.
        // Presentation only: the panel shows what the reducer found and writes
        // nothing back, so the trace and the hash never learn it exists.
        let trend = self.runtime.trend();
        // The question the driver is holding at, if it is holding at one.
        // Cloned off the driver so the world can be borrowed below; presentation
        // only, like everything else on this side of the frame.
        let checkpoint = self.runtime.checkpoint().cloned();
        // The played line's turn, at a lineage checkpoint. The cursor is kept in
        // range here rather than in the key handler, so a review that came back
        // shorter than the last one cannot leave it pointing past the table.
        let review = self.runtime.review().cloned();
        let board_row = match &review {
            Some(review) if !review.rows.is_empty() => self.board_row.min(review.rows.len() - 1),
            _ => 0,
        };
        self.board_row = board_row;
        // The dev lane's own reading (DT1). `None` outside `--dev`, which is
        // also when nothing below touches `lanes.dev` at all.
        let dev = self.dev_reading();
        let graft_menu = self
            .creator
            .as_ref()
            .map(|c| &c.reading)
            .or_else(|| self.grafting.open.then_some(&self.grafting.reading));
        let focused_body = self.config.dev.then(|| self.followed()).flatten();

        let world = self.runtime.world();
        let Some(gpu) = &mut self.gpu else { return };
        let (backdrop_centre, surface_y) =
            self.habitat
                .as_ref()
                .map_or((centre, at[1] as f32), |habitat| {
                    (
                        [0, 1, 2]
                            .map(|i| (habitat.bounds.min[i] + habitat.bounds.max[i]) as f32 * 0.5),
                        habitat.clearing[1] as f32,
                    )
                });
        gpu.section.configure_terrain(
            self.config.terrain_style.resolved(self.habitat.is_some()),
            backdrop_centre,
            surface_y,
        );
        let candidate = self
            .creator
            .as_ref()
            .map(creator::Creator::world)
            .unwrap_or_else(|| self.grafting.preview.as_deref().unwrap_or(world));
        let graft_selection = self
            .grafting
            .open
            .then(|| grafting::selection(&gpu.section, candidate, self.grafting.root))
            .flatten();
        let subject = if body_review {
            candidate.controlled().map(|o| o.id)
        } else {
            focused_body
        };
        gpu.section
            .set_body_focus(subject, graft_selection.or(self.inspection.selected));
        // wgpu 29 returns an enum rather than a Result here: a suboptimal
        // texture is still drawable, and a lost or outdated surface wants
        // reconfiguring rather than an error.
        let surface_texture = match gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                gpu.surface.configure(&gpu.device, &gpu.config);
                return;
            },
            _ => return,
        };

        let view = surface_texture.texture.create_view(&Default::default());
        let section_frame = SectionFrame {
            world: candidate,
            volumes: &self.volumes,
            ground: world.ground(),
            dirty: &dirty,
            centre,
            pose: pose.as_ref(),
            roster: &roster,
        };
        if let Some(lanes) = &mut gpu.chrome {
            let frame = (gpu.config.width, gpu.config.height);
            // The section is one closed tenant producer. Submit its caller-owned
            // encoder before Netrender imports the resulting display texture.
            let mut tenant_encoder =
                gpu.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("mesocosm section tenant"),
                    });
            if let Err(error) = gpu.section.render(&mut tenant_encoder, section_frame) {
                eprintln!("section: {error}");
            }
            gpu.queue.submit(Some(tenant_encoder.finish()));

            if let Some((body, loose)) = &scene {
                let mut items = Vec::with_capacity(loose.len() + 1);
                items.push(SceneItem::new(body, played_at));
                for (mesh, place, tint, warns, colour, scale) in loose {
                    items.push(SceneItem::creature(
                        mesh, *place, *tint, *warns, *colour, *scale,
                    ));
                }
                lanes.hud.render_backdrop(&items, steps);
            }
            lanes.hud.refresh(&lanes.device, world);
            // After the minimap, so a refusal reads over the section rather
            // than under it. Presentation only: nothing it reads is written
            // back, so the trace and the hash never learn it exists.
            lanes
                .vitals
                .refresh(&lanes.device, world, &outcomes, steps, &trend);
            // The dev lane is ordinary chrome beside the vitals panel. The
            // checkpoint and board remain last because either stops the world.
            lanes
                .board
                .refresh(&lanes.device, review.as_ref(), board_row);
            let held = checkpoint.as_ref().filter(|_| !lanes.board.standing());
            lanes.checkpoint.refresh(&lanes.device, held);
            lanes.grafting.update(&lanes.device, graft_menu);

            let framed = lanes.device.frame_master(
                gpu.section.display_texture(),
                frame,
                gpu.section.body_stats().fallback_bodies as u64,
            );
            gpu.last_tenant_receipt = Some(framed.receipt);
            let master_view = framed.texture.create_view(&Default::default());
            let mut chrome_encoder =
                gpu.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("mesocosm chrome into master"),
                    });
            if !body_review {
                lanes
                    .hud
                    .composite(&lanes.device, &mut chrome_encoder, &master_view, frame);
                lanes
                    .vitals
                    .composite(&lanes.device, &mut chrome_encoder, &master_view, frame);
                devtime::composite_dev_lane(
                    lanes,
                    dev.as_ref(),
                    &mut chrome_encoder,
                    &master_view,
                    frame,
                );
            }
            lanes
                .checkpoint
                .composite(&lanes.device, &mut chrome_encoder, &master_view, frame);
            lanes
                .board
                .composite(&lanes.device, &mut chrome_encoder, &master_view, frame);
            lanes
                .grafting
                .composite(&lanes.device, &mut chrome_encoder, &master_view, frame);
            gpu.queue.submit(Some(chrome_encoder.finish()));

            let mut present_encoder =
                gpu.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("mesocosm master to surface"),
                    });
            lanes
                .device
                .draw_surface(&mut present_encoder, &view, &master_view, frame);
            gpu.queue.submit(Some(present_encoder.finish()));
        } else {
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("chromeless frame"),
                });
            if let Err(error) = gpu.section.draw(&mut encoder, &view, section_frame) {
                eprintln!("section: {error}");
            }
            gpu.queue.submit(Some(encoder.finish()));
        }
        // wgpu 30 moved presentation from SurfaceTexture to Queue.
        gpu.queue.present(surface_texture);

        self.frames += 1;
        let hit_limit = self.config.frames.is_some_and(|limit| self.frames >= limit);
        // **A scenario decides when its own run ends** (DT4). It is pumped
        // after the frame is drawn, so an `assert text` reads what was just on
        // screen and a `capture` writes exactly that frame; and neither a
        // finished replay nor a frame limit closes the window under it, or the
        // assertions after the last step would never run.
        if self.scenario.is_some() {
            self.drive_scenario(event_loop, hit_limit);
        } else if replay_done || hit_limit {
            self.finish(event_loop);
        }
    }
}
