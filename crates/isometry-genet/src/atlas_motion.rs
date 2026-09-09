//! Frame scheduling for retained atlas labels. Geographic hit targets stay fixed.

use super::*;

impl App {
    pub(crate) fn drive_atlas_motion(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if !ctx.runner.state().overmap_open {
            self.overmap_frame_at = None;
            return false;
        }
        let now = Instant::now();
        let dt = self.overmap_frame_at.map_or(0.0, |last| {
            now.saturating_duration_since(last).as_secs_f32().min(0.1)
        });
        let tick = ctx.runner.update_deferred(|ui| {
            ui.sync_overmap_motion_settings();
            ui.overmap_motion.tick(dt, ui.overmap_reduced_motion)
        });
        self.overmap_frame_at = tick.active.then_some(now);
        // Geographic semantic targets never animate. Reconcile other UI once,
        // when the final offset reaches home, rather than on every frame.
        if tick.changed && !tick.active {
            ctx.runner.update(|_| {});
        }
        tick.active
    }
}
