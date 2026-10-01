//! Privileged, bounded input routing; all legality and application stay in Driver.
use mtg_core::{
    episode::{Driver, RecordContext},
    game::actions::Record,
    objects::Seat,
};
use serde::{Deserialize, Serialize};

pub const POLICY: &str = "semantic-script-v1";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub privacy: String,
    pub max_bytes: usize,
    pub max_records: usize,
    pub records: Vec<Entry>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub episode: u64,
    pub decision: u64,
    pub seat: Seat,
    pub record: String,
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.privacy != "privileged" {
            return Err("unsupported script version or privacy declaration".into());
        }
        if self.max_bytes == 0
            || self.max_bytes > 1_048_576
            || self.max_records == 0
            || self.max_records > 100_000
        {
            return Err("invalid script bounds".into());
        }
        // Count the UTF-8 bytes of the complete compact entry array, including
        // envelopes and escaped record strings, before any episode is started.
        if self.records.len() > self.max_records
            || serde_json::to_vec(&self.records)
                .map_err(|_| "invalid script")?
                .len()
                > self.max_bytes
        {
            return Err("script input bound exceeded".into());
        }
        Ok(())
    }
}

/// The explicit episode/decision position scopes a static script input. Live
/// process-local revision/generation tokens are obtained only after this check.
/// The cursor advances only after acceptance, including out-of-band concession.
pub fn submit(
    c: &Config,
    cursor: &mut usize,
    episode: u64,
    d: &mut Driver,
) -> Result<(), &'static str> {
    let e = c.records.get(*cursor).ok_or("script_missing")?;
    if e.episode != episode || e.decision != d.privileged_history().len() as u64 {
        return Err("script_position");
    }
    let record: Record = serde_json::from_str(&e.record).map_err(|_| "script_record")?;
    let context = if record.decision == "concession" {
        RecordContext::Concession {
            episode: d.episode_id().ok_or("script_context")?,
        }
    } else {
        let decision = d
            .observe(e.seat)
            .map_err(|_| "script_context")?
            .decision
            .ok_or("script_seat")?;
        RecordContext::Decision {
            revision: decision.revision,
            generation: decision.generation,
        }
    };
    d.submit_record(e.seat, context, e.record.as_bytes())
        .map_err(|_| "script_rejected")?;
    *cursor += 1;
    Ok(())
}

#[cfg(test)]
#[path = "script_tests.rs"]
mod tests;
