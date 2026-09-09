// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-owned frame validation and presentation policy.

/// Whether a validation result suppresses the frame whose scope produced it
/// or is latched for the first still-unpresented frame observed afterward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationPolicy {
    AwaitedDiagnostic,
    Optimistic,
}

impl PresentationPolicy {
    /// Read `PAREDROS_FRAME_POLICY=awaited` or `optimistic`. Interactive room
    /// rendering defaults to optimistic presentation.
    pub fn from_env() -> Self {
        let value = std::env::var("PAREDROS_FRAME_POLICY").unwrap_or_default();
        match value.to_ascii_lowercase().as_str() {
            "" => Self::Optimistic,
            "awaited" | "awaiteddiagnostic" | "awaited-diagnostic" => Self::AwaitedDiagnostic,
            "optimistic" => Self::Optimistic,
            _ => panic!("PAREDROS_FRAME_POLICY must be awaited or optimistic, got {value:?}"),
        }
    }
}

/// Result of the host's frame-health gate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameDecision {
    Proceed,
    Suppress,
    RebuildAll(SharedFault),
}

/// A validation error attributed to one tenant operation and frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationRecord {
    pub tenant_name: String,
    pub producer_path: String,
    pub frame: u64,
    pub error: String,
}

/// A fault which affects the shared device and requires rebuild-all handling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharedFault {
    UncapturedError { error: String },
    DeviceLost { reason: String, message: String },
    PollFailure { error: String },
}

/// Host-owned state machine. It deliberately does not attempt tenant-local
/// recovery; shared faults are named as rebuild-all dispositions for the
/// caller's outer lifecycle.
#[derive(Debug)]
pub struct FrameHealth {
    policy: PresentationPolicy,
    pending_optimistic: Option<ValidationRecord>,
    shared_fault: Option<SharedFault>,
    validations: Vec<ValidationRecord>,
}

impl FrameHealth {
    pub fn new(policy: PresentationPolicy) -> Self {
        Self {
            policy,
            pending_optimistic: None,
            shared_fault: None,
            validations: Vec::new(),
        }
    }

    pub fn policy(&self) -> PresentationPolicy {
        self.policy
    }

    /// Resolve an earlier optimistic validation before trace advance or
    /// surface acquisition. Shared faults remain latched until outer rebuild.
    pub fn begin_frame(&mut self, _frame: u64) -> FrameDecision {
        if let Some(fault) = self.shared_fault.clone() {
            FrameDecision::RebuildAll(fault)
        } else if self.pending_optimistic.take().is_some() {
            FrameDecision::Suppress
        } else {
            FrameDecision::Proceed
        }
    }

    /// Resolve the validation scope while still on the room event-loop thread.
    pub fn finish_validation(
        &mut self,
        tenant_name: impl Into<String>,
        producer_path: impl Into<String>,
        frame: u64,
        error: Option<String>,
    ) -> FrameDecision {
        let Some(error) = error else {
            return FrameDecision::Proceed;
        };
        let record = ValidationRecord {
            tenant_name: tenant_name.into(),
            producer_path: producer_path.into(),
            frame,
            error,
        };
        self.validations.push(record.clone());
        match self.policy {
            PresentationPolicy::AwaitedDiagnostic => FrameDecision::Suppress,
            PresentationPolicy::Optimistic => {
                self.pending_optimistic = Some(record);
                FrameDecision::Proceed
            },
        }
    }

    /// Latch an uncaptured host/shared error. Tenant attribution belongs to a
    /// scoped validation record, so callbacks never infer it here.
    pub fn latch_uncaptured_error(&mut self, error: impl Into<String>) {
        self.shared_fault = Some(SharedFault::UncapturedError {
            error: error.into(),
        });
    }

    /// Latch device loss as a rebuild-all fault. Recreating the shared device,
    /// tenant, and renderer is intentionally outside this slice.
    pub fn latch_device_lost(&mut self, reason: impl Into<String>, message: impl Into<String>) {
        self.shared_fault = Some(SharedFault::DeviceLost {
            reason: reason.into(),
            message: message.into(),
        });
    }

    /// Latch a failed nonblocking device poll as a shared rebuild-all fault.
    pub fn latch_poll_failure(&mut self, error: impl Into<String>) {
        self.shared_fault = Some(SharedFault::PollFailure {
            error: error.into(),
        });
    }

    pub fn shared_fault(&self) -> Option<&SharedFault> {
        self.shared_fault.as_ref()
    }

    pub fn validations(&self) -> &[ValidationRecord] {
        &self.validations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeFrameLoop {
        acquired: usize,
        presented: usize,
    }

    impl FakeFrameLoop {
        fn run(
            &mut self,
            health: &mut FrameHealth,
            frame: u64,
            prior_error: Option<&str>,
            current_error: Option<&str>,
        ) {
            if let Some(error) = prior_error {
                health.finish_validation("tenant", "producer", frame - 1, Some(error.into()));
            }
            if !matches!(health.begin_frame(frame), FrameDecision::Proceed) {
                return;
            }
            if current_error.is_some() && health.policy() == PresentationPolicy::AwaitedDiagnostic {
                let decision = health.finish_validation(
                    "tenant",
                    "producer",
                    frame,
                    current_error.map(str::to_owned),
                );
                if matches!(
                    decision,
                    FrameDecision::Suppress | FrameDecision::RebuildAll(_)
                ) {
                    return;
                }
            }
            self.acquired += 1;
            self.presented += 1;
        }
    }

    #[test]
    fn awaited_validation_suppresses_the_scoped_frame_before_present() {
        let mut health = FrameHealth::new(PresentationPolicy::AwaitedDiagnostic);
        let mut loop_ = FakeFrameLoop::default();
        loop_.run(&mut health, 1, None, Some("bad tenant copy"));
        assert_eq!(loop_.acquired, 0);
        assert_eq!(loop_.presented, 0);
        assert_eq!(health.validations()[0].frame, 1);
        assert_eq!(health.validations()[0].tenant_name, "tenant");
        loop_.run(&mut health, 2, None, None);
        assert_eq!(loop_.presented, 1);
    }

    #[test]
    fn optimistic_validation_presents_current_then_suppresses_next_unpresented() {
        let mut health = FrameHealth::new(PresentationPolicy::Optimistic);
        let mut loop_ = FakeFrameLoop::default();
        // F1 presents while its validation scope remains unresolved.
        loop_.run(&mut health, 1, None, None);
        assert_eq!((loop_.acquired, loop_.presented), (1, 1));
        loop_.run(&mut health, 2, Some("bad tenant copy"), None);
        assert_eq!((loop_.acquired, loop_.presented), (1, 1));
        loop_.run(&mut health, 3, None, None);
        assert_eq!((loop_.acquired, loop_.presented), (2, 2));
    }

    #[test]
    fn shared_fault_suppresses_until_outer_rebuild() {
        let mut health = FrameHealth::new(PresentationPolicy::Optimistic);
        health.latch_device_lost("destroyed", "rebuild all");
        let mut loop_ = FakeFrameLoop::default();
        loop_.run(&mut health, 1, None, None);
        assert_eq!((loop_.acquired, loop_.presented), (0, 0));
        assert!(matches!(
            health.begin_frame(2),
            FrameDecision::RebuildAll(SharedFault::DeviceLost { .. })
        ));
        assert!(matches!(
            health.shared_fault(),
            Some(SharedFault::DeviceLost { .. })
        ));
    }

    #[test]
    fn poll_failure_is_a_shared_rebuild_all_fault() {
        let mut health = FrameHealth::new(PresentationPolicy::Optimistic);
        health.latch_poll_failure("poll failed");
        assert!(matches!(
            health.begin_frame(1),
            FrameDecision::RebuildAll(SharedFault::PollFailure { .. })
        ));
    }
}
