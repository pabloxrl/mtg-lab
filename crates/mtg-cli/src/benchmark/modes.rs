//! Additive schema-2 settings and receipts. No game rules live here.
use super::*;
use std::num::NonZeroUsize;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReplayConfig {
    pub max_bytes: usize,
    pub max_records: NonZeroUsize,
    pub persistence: String,
}
impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            max_bytes: 67_108_864,
            max_records: NonZeroUsize::new(20_000).unwrap(),
            persistence: "in_memory".into(),
        }
    }
}
impl ReplayConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=67_108_864).contains(&self.max_bytes)
            || self.max_records.get() > 20_000
            || self.persistence != "in_memory"
        {
            return Err("unsupported replay capacity or persistence; use independent canonical capture for durable storage".into());
        }
        Ok(())
    }
}

#[derive(Default, Debug, Serialize)]
pub(super) struct Execution {
    pub not_started: u64,
    pub trace_selected: u64,
    pub trace_retained: u64,
    pub trace_dropped: u64,
    pub replay_verified: u64,
    pub replay_bytes: u64,
    pub replay_failed: u64,
    pub replay_incomplete: u64,
    pub replay_ns: u128,
    pub publication_ns: u128,
    pub published: u64,
    pub publication_failed: u64,
}
impl Execution {
    pub fn from_timing(t: &native::PolicyTiming) -> Self {
        let r = t.replay.as_ref().unwrap();
        Self {
            replay_verified: r.verified,
            replay_bytes: r.bytes.as_ref().map_or(0, |b| b.len() as u64),
            replay_failed: r.failed,
            replay_incomplete: r.incomplete,
            replay_ns: r.elapsed_ns,
            publication_ns: t.publication_ns,
            ..Self::default()
        }
    }
    pub fn read_rows(&mut self, rows: &[Value]) {
        for row in rows {
            if let Some(trace) = row.get("diagnostics") {
                self.trace_retained += trace["records"].as_array().unwrap().len() as u64;
                self.trace_dropped += trace["dropped"].as_u64().unwrap();
                self.trace_selected = self.trace_retained + self.trace_dropped;
            }
            if row["type"] == "publication" {
                if row["status"] == "published" {
                    self.published += 1;
                } else {
                    self.publication_failed += 1;
                }
            }
        }
    }
    pub fn merge(&mut self, r: Self) {
        self.not_started += r.not_started;
        self.trace_selected += r.trace_selected;
        self.trace_retained += r.trace_retained;
        self.trace_dropped += r.trace_dropped;
        self.replay_verified += r.replay_verified;
        self.replay_bytes += r.replay_bytes;
        self.replay_failed += r.replay_failed;
        self.replay_incomplete += r.replay_incomplete;
        self.replay_ns += r.replay_ns;
        self.publication_ns += r.publication_ns;
        self.published += r.published;
        self.publication_failed += r.publication_failed;
    }
}

#[cfg(test)]
#[path = "modes_tests.rs"]
mod tests;
