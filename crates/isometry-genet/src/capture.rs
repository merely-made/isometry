//! Bounded native-frame capture policy.
//!
//! A normal headed receipt needs one final image. Continuous readback and PNG
//! encoding are deliberately opt-in because they stall the render thread.

use crate::Ctx;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct Capture {
    directory: Option<PathBuf>,
    every_frame: bool,
    saved: Arc<AtomicBool>,
}

impl Capture {
    pub(crate) fn from_env() -> Self {
        Self {
            directory: std::env::var_os("ISOMETRY_CAPTURE_DIR").map(Into::into),
            every_frame: std::env::var("ISOMETRY_CAPTURE_EVERY_FRAME")
                .is_ok_and(|value| value == "1"),
            saved: Arc::new(AtomicBool::new(false)),
        }
    }

    fn should_arm(&self, selftests_complete: bool) -> bool {
        self.directory.is_some()
            && (self.every_frame || (selftests_complete && !self.saved.load(Ordering::Acquire)))
    }

    /// Request a readback for this presentation when the capture policy allows it.
    /// A failed readback or write leaves `saved` clear, so a later natural frame
    /// retries. Capture never creates an idle redraw loop.
    pub(crate) fn arm(&self, ctx: &mut Ctx<'_>, selftests_complete: bool) {
        if !self.should_arm(selftests_complete) {
            return;
        }
        let directory = self.directory.clone().expect("checked by should_arm");
        let every_frame = self.every_frame;
        let saved = self.saved.clone();
        *ctx.capture = Some(Box::new(move |surface, view, width, height| {
            if !every_frame && saved.load(Ordering::Acquire) {
                return;
            }
            let Some(frame) = cambium_genet_winit_host::read_frame(surface, view, width, height)
            else {
                return;
            };
            let path = directory.join("isometry_capture.png");
            let pending = directory.join("isometry_capture.png.tmp");
            let result = std::fs::create_dir_all(&directory).and_then(|_| {
                let file = std::fs::File::create(&pending)?;
                let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                let mut writer = encoder.write_header().map_err(std::io::Error::other)?;
                writer
                    .write_image_data(&frame.rgba)
                    .map_err(std::io::Error::other)?;
                writer.finish().map_err(std::io::Error::other)?;
                std::fs::rename(&pending, &path)
            });
            match result {
                Ok(()) => {
                    saved.store(true, Ordering::Release);
                },
                Err(error) => eprintln!("[isometry] capture failed: {error}"),
            }
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture(every_frame: bool, saved: bool) -> Capture {
        Capture {
            directory: Some(PathBuf::from("receipt")),
            every_frame,
            saved: Arc::new(AtomicBool::new(saved)),
        }
    }

    #[test]
    fn ordinary_capture_waits_for_selftests_and_stops_after_success() {
        let capture = capture(false, false);
        assert!(!capture.should_arm(false));
        assert!(capture.should_arm(true));
        capture.saved.store(true, Ordering::Release);
        assert!(!capture.should_arm(true));
    }

    #[test]
    fn every_frame_capture_remains_armed_during_selftests() {
        let capture = capture(true, true);
        assert!(capture.should_arm(false));
        assert!(capture.should_arm(true));
    }
}
