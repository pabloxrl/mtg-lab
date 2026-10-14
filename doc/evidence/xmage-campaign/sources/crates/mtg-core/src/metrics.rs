//! Local scalar instrumentation. Counters contain no episode or card identifiers.
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Off,
    Counters,
    SampledTrace,
    FullReplay,
}
impl Mode {
    pub fn is_off(&self) -> bool {
        *self == Self::Off
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct DecisionCounts {
    pub opening: u64,
    pub bottom: u64,
    pub priority: u64,
    pub payment: u64,
    pub target: u64,
    pub mode: u64,
    pub discard: u64,
    pub activation: u64,
    pub attackers: u64,
    pub blockers: u64,
    pub combat_damage: u64,
    pub trigger_target: u64,
    pub trigger_order: u64,
    pub cleanup_discard: u64,
    pub other: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Counters {
    pub resets: u64,
    pub started: u64,
    pub completed: u64,
    pub rules_completed: u64,
    pub truncated: u64,
    pub failed: u64,
    pub incomplete: u64,
    pub decisions: u64,
    pub decisions_by_kind: DecisionCounts,
    pub logical_actions: u64,
    pub committed_actions: u64,
    pub cancelled_actions: u64,
    pub concessions: u64,
    pub rules_work_units: u64,
    pub invalid_actions: u64,
    pub stale_actions: u64,
    pub rejected_resets: u64,
    pub capacity_overflows: u64,
    pub recording_failures: u64,
    pub clock_failures: u64,
    pub boundary_errors: u64,
    pub overflowed: bool,
}

/// Saturation is explicit; an overflowed report is not an exact measurement.
pub(crate) fn add(value: &mut u64, delta: u64, overflowed: &mut bool) {
    match value.checked_add(delta) {
        Some(n) => *value = n,
        None => {
            *value = u64::MAX;
            *overflowed = true;
        }
    }
}
impl Counters {
    pub fn merge(&mut self, other: &Self) {
        self.overflowed |= other.overflowed;
        add(&mut self.resets, other.resets, &mut self.overflowed);
        add(&mut self.started, other.started, &mut self.overflowed);
        add(&mut self.completed, other.completed, &mut self.overflowed);
        add(
            &mut self.rules_completed,
            other.rules_completed,
            &mut self.overflowed,
        );
        add(&mut self.truncated, other.truncated, &mut self.overflowed);
        add(&mut self.failed, other.failed, &mut self.overflowed);
        add(&mut self.incomplete, other.incomplete, &mut self.overflowed);
        add(&mut self.decisions, other.decisions, &mut self.overflowed);
        add(
            &mut self.logical_actions,
            other.logical_actions,
            &mut self.overflowed,
        );
        add(
            &mut self.committed_actions,
            other.committed_actions,
            &mut self.overflowed,
        );
        add(
            &mut self.cancelled_actions,
            other.cancelled_actions,
            &mut self.overflowed,
        );
        add(
            &mut self.concessions,
            other.concessions,
            &mut self.overflowed,
        );
        add(
            &mut self.rules_work_units,
            other.rules_work_units,
            &mut self.overflowed,
        );
        add(
            &mut self.invalid_actions,
            other.invalid_actions,
            &mut self.overflowed,
        );
        add(
            &mut self.stale_actions,
            other.stale_actions,
            &mut self.overflowed,
        );
        add(
            &mut self.rejected_resets,
            other.rejected_resets,
            &mut self.overflowed,
        );
        add(
            &mut self.capacity_overflows,
            other.capacity_overflows,
            &mut self.overflowed,
        );
        add(
            &mut self.recording_failures,
            other.recording_failures,
            &mut self.overflowed,
        );
        add(
            &mut self.clock_failures,
            other.clock_failures,
            &mut self.overflowed,
        );
        add(
            &mut self.boundary_errors,
            other.boundary_errors,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.opening,
            other.decisions_by_kind.opening,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.bottom,
            other.decisions_by_kind.bottom,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.priority,
            other.decisions_by_kind.priority,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.payment,
            other.decisions_by_kind.payment,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.target,
            other.decisions_by_kind.target,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.mode,
            other.decisions_by_kind.mode,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.discard,
            other.decisions_by_kind.discard,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.activation,
            other.decisions_by_kind.activation,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.attackers,
            other.decisions_by_kind.attackers,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.blockers,
            other.decisions_by_kind.blockers,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.combat_damage,
            other.decisions_by_kind.combat_damage,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.trigger_target,
            other.decisions_by_kind.trigger_target,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.trigger_order,
            other.decisions_by_kind.trigger_order,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.cleanup_discard,
            other.decisions_by_kind.cleanup_discard,
            &mut self.overflowed,
        );
        add(
            &mut self.decisions_by_kind.other,
            other.decisions_by_kind.other,
            &mut self.overflowed,
        );
    }
    pub(crate) fn decision(&mut self, kind: &str, starts: bool, status: Completion) {
        add(&mut self.decisions, 1, &mut self.overflowed);
        let count = match kind {
            "keep_or_mulligan" => &mut self.decisions_by_kind.opening,
            "bottom" => &mut self.decisions_by_kind.bottom,
            "priority" => &mut self.decisions_by_kind.priority,
            "payment" => &mut self.decisions_by_kind.payment,
            "growth_target" | "bite_source" | "bite_destination" | "targets_complete" => {
                &mut self.decisions_by_kind.target
            }
            "cast_mode" => &mut self.decisions_by_kind.mode,
            "cast_discard" => &mut self.decisions_by_kind.discard,
            "activation_payment" | "activation_target" => &mut self.decisions_by_kind.activation,
            "attackers" => &mut self.decisions_by_kind.attackers,
            "blockers" => &mut self.decisions_by_kind.blockers,
            "combat_damage" => &mut self.decisions_by_kind.combat_damage,
            "trigger_target" => &mut self.decisions_by_kind.trigger_target,
            "trigger_order" => &mut self.decisions_by_kind.trigger_order,
            "cleanup_discard" => &mut self.decisions_by_kind.cleanup_discard,
            _ => &mut self.decisions_by_kind.other,
        };
        add(count, 1, &mut self.overflowed);
        if starts {
            add(&mut self.logical_actions, 1, &mut self.overflowed);
        }
        match status {
            Completion::Committed => add(&mut self.committed_actions, 1, &mut self.overflowed),
            Completion::Cancelled => add(&mut self.cancelled_actions, 1, &mut self.overflowed),
            Completion::Continuing => (),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Completion {
    Continuing,
    Committed,
    Cancelled,
}

/// Fixed field names and availability tags; no per-game labels or raw errors.
#[derive(Serialize)]
pub struct Report<'a> {
    pub schema_version: u32,
    pub counters: &'a Counters,
    pub availability: Availability,
}
#[derive(Serialize)]
pub struct Availability {
    pub worker_busy_idle: &'static str,
    pub ready_queue_delay: &'static str,
    pub batch_fill: &'static str,
    pub inference_time: &'static str,
    pub encoding_time: &'static str,
    pub policy_time: &'static str,
    pub memory_high_water: &'static str,
    pub sampled_timing: &'static str,
    pub diagnostic_records: &'static str,
}
impl Counters {
    pub fn report(&self) -> Report<'_> {
        Report {
            schema_version: SCHEMA_VERSION,
            counters: self,
            availability: Availability {
                worker_busy_idle: "not_applicable",
                ready_queue_delay: "not_applicable",
                batch_fill: "not_applicable",
                inference_time: "not_applicable",
                encoding_time: "not_measured",
                policy_time: "not_measured",
                memory_high_water: "not_measured",
                sampled_timing: "not_measured",
                diagnostic_records: "not_applicable",
            },
        }
    }
}

/// Fixed-size, public diagnostic checkpoints. No player observations or identifiers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TraceRecord {
    pub decision: u64,
    pub rules_work_units: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceConfig {
    pub every: std::num::NonZeroU64,
    pub capacity: usize,
}
impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            every: std::num::NonZeroU64::new(64).unwrap(),
            capacity: 256,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Trace {
    pub schema_version: u32,
    pub records: Vec<TraceRecord>,
    pub dropped: u64,
    pub overflowed: bool,
}

impl Default for Trace {
    fn default() -> Self {
        Self {
            schema_version: 1,
            records: Vec::new(),
            dropped: 0,
            overflowed: false,
        }
    }
}
