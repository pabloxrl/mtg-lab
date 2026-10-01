use mtg_core::{
    objects::Seat,
    opening::{
        Game, OpeningAction, OpeningKind, Selection,
        terminal::LossReason,
        turns::{TurnAction, TurnKind, TurnSelection},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{self, Write};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<super::script::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native: Option<super::native::Config>,
    pub schema_version: u32,
    pub game: mtg_core::opening::Config,
    pub policies: [String; 2],
    pub master_seed: u64,
    pub first_episode: u64,
    pub episodes: u64,
    pub max_decisions: u64,
    pub deadline_ms: Option<u64>,
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if ![1, 2, 3].contains(&self.schema_version)
            || self.episodes == 0
            || self.max_decisions == 0
            || self.deadline_ms == Some(0)
        {
            return Err("unsupported version or nonpositive budget".into());
        }
        self.first_episode
            .checked_add(self.episodes - 1)
            .ok_or("episode ID range overflows")?;
        if self.schema_version != 3 && self.script.is_some() {
            return Err("script requires schema version 3".into());
        }
        if self.schema_version == 3 {
            if self.policies.iter().any(|p| p != super::script::POLICY) {
                return Err(
                    "both policies must explicitly be semantic-script-v1; no fallback".into(),
                );
            }
            self.script.as_ref().ok_or("script required")?.validate()?;
            self.native
                .as_ref()
                .ok_or("owned execution bounds required")?
                .validate_bounds()?;
        } else if self.schema_version == 2 {
            self.native
                .as_ref()
                .ok_or("native bounds required")?
                .validate(&self.policies)?;
        } else if self.native.is_some() || self.policies.iter().any(|p| p != "pass-v1") {
            return Err("both policies must explicitly be pass-v1; no policy fallback".into());
        }
        let mut g = Game::new().map_err(debug)?;
        g.reset(&self.game, self.master_seed, self.first_episode)
            .map(|_| ())
            .map_err(debug)
    }
}
fn debug(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn emit(w: &mut impl Write, mut value: Value) -> io::Result<()> {
    value["schema_version"] = json!(1);
    serde_json::to_writer(&mut *w, &value)?;
    w.write_all(b"\n")?;
    w.flush()
}
#[derive(Clone, Copy)]
pub enum Stop {
    Sigterm,
    Sigint,
    Deadline,
}
impl Stop {
    pub(crate) fn reason(self) -> &'static str {
        match self {
            Self::Sigterm => "sigterm",
            Self::Sigint => "sigint",
            Self::Deadline => "deadline",
        }
    }
    pub(crate) fn code(self) -> i32 {
        match self {
            Self::Sigterm => 143,
            Self::Sigint => 130,
            Self::Deadline => 4,
        }
    }
}
#[derive(Default, Serialize)]
struct Counts {
    requested: u64,
    started: u64,
    completed: u64,
    truncated: u64,
    failed: u64,
    unfinished: u64,
    not_started: u64,
    wins: [u64; 2],
    draws: u64,
}

/// Each policy action is explicit and accepted by the authoritative core.
/// pass-v1 needs no privileged state or randomness: keep, pass, first cleanup
/// candidate rows. It never attempts to play unsupported cards or abilities.
pub fn step(g: &mut Game) -> Result<(), String> {
    if let Some(d) = g.decision() {
        if d.kind != OpeningKind::KeepOrMulligan {
            return Err("unexpected opening choice for pass-v1".into());
        }
        g.apply(
            d.actor,
            &OpeningAction {
                decision: d.id,
                selection: Selection::Choose(d.candidate(0)),
            },
        )
        .map_err(debug)?;
        if g.decision().is_none() {
            g.start_turns().map_err(debug)?;
        }
        return Ok(());
    }
    let d = g.turn_decision().ok_or("missing decision")?;
    let selection = match d.kind {
        TurnKind::Priority => TurnSelection::Pass(d.candidate(0)),
        TurnKind::Discard { count } => {
            TurnSelection::Discard((0..count).map(|i| d.candidate(i)).collect())
        }
        TurnKind::Combat(_) => return Err("unexpected combat decision for pass-v1".into()),
    };
    g.apply_turn(
        d.actor,
        &TurnAction {
            decision: d.id,
            selection,
        },
    )
    .map_err(debug)?;
    Ok(())
}

pub fn run(
    c: &Config,
    w: &mut impl Write,
    mut control: impl FnMut() -> Option<Stop>,
    mut apply: impl FnMut(&mut Game) -> Result<(), String>,
) -> io::Result<i32> {
    emit(
        w,
        json!({"type":"run","config":c,"config_sha256":hash(&serde_json::to_vec(c)?),"engine_sha256":env!("MTG_ENGINE_SHA256"),"cli_version":env!("CARGO_PKG_VERSION"),"policy_version":"pass-v1","rules_sha256":hash(include_bytes!("../../../data/rules/cr-2026-09-25.json")),"cards_sha256":hash(include_bytes!("../../../data/cards/foundations_micro_v1.json")),"workers":1,"instrumentation":"summary-v1","capture":"none"}),
    )?;
    let mut counts = Counts {
        requested: c.episodes,
        ..Counts::default()
    };
    let mut exit = 0;
    let mut run_reason = "budget_complete";
    for offset in 0..c.episodes {
        if let Some(stop) = control() {
            exit = stop.code();
            run_reason = stop.reason();
            break;
        }
        let episode = c.first_episode + offset;
        counts.started += 1;
        let mut g = Game::new().map_err(debug).and_then(|mut g| {
            g.reset(&c.game, c.master_seed, episode).map_err(debug)?;
            Ok(g)
        });
        let mut decisions = 0;
        let mut error = None;
        let (status, reason, winner) = loop {
            let game = match &mut g {
                Ok(g) => g,
                Err(e) => {
                    error = Some(e.clone());
                    break ("failed", "engine_error", None);
                }
            };
            // A genuine terminal result always wins over a simultaneously observed limit.
            if let Some(outcome) = game.outcome() {
                let reason = if outcome.losses.contains(&Some(LossReason::EmptyDraw)) {
                    "empty_draw"
                } else {
                    "rules_terminal"
                };
                break (
                    "completed",
                    reason,
                    outcome.winner.map(|s| if s == Seat::P0 { 0 } else { 1 }),
                );
            }
            if let Some(stop) = control() {
                exit = stop.code();
                run_reason = stop.reason();
                break (
                    if matches!(stop, Stop::Deadline) {
                        "truncated"
                    } else {
                        "unfinished"
                    },
                    stop.reason(),
                    None,
                );
            }
            if decisions >= c.max_decisions {
                break ("truncated", "decision_limit", None);
            }
            match apply(game) {
                Ok(()) => decisions += 1,
                Err(e) => {
                    error = Some(e);
                    break ("failed", "engine_error", None);
                }
            }
        };
        match status {
            "completed" => {
                counts.completed += 1;
                if let Some(seat) = winner {
                    counts.wins[seat] += 1;
                } else {
                    counts.draws += 1;
                }
            }
            "truncated" => counts.truncated += 1,
            "unfinished" => counts.unfinished += 1,
            _ => {
                counts.failed += 1;
                exit = 3;
                run_reason = "engine_error";
            }
        }
        emit(
            w,
            json!({"type":"episode","episode":episode,"status":status,"reason":reason,"winner":winner,"decisions":decisions,"life":g.as_ref().ok().map(Game::life),"turn":g.as_ref().ok().and_then(Game::turn_position).map(|p|p.0),"error":error}),
        )?;
        if exit != 0 {
            break;
        }
    }
    counts.not_started = c.episodes - counts.started;
    let mut summary = serde_json::to_value(counts)?;
    summary["type"] = json!("summary");
    summary["reason"] = json!(run_reason);
    summary["exit_code"] = json!(exit);
    emit(w, summary)?;
    Ok(exit)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Config {
        Config {
            native: None,
            script: None,
            schema_version: 1,
            game: Default::default(),
            policies: ["pass-v1".into(), "pass-v1".into()],
            master_seed: 42,
            first_episode: 7,
            episodes: 3,
            max_decisions: 3000,
            deadline_ms: None,
        }
    }
    fn rows(bytes: &[u8]) -> Vec<Value> {
        String::from_utf8_lossy(bytes)
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect()
    }
    #[test]
    fn simulate_engine_failure_is_quarantined_and_stops_run() {
        let mut output = vec![];
        let exit = run(
            &config(),
            &mut output,
            || None,
            |_| Err("injected engine boundary failure".into()),
        )
        .unwrap();
        assert_eq!(exit, 3);
        let r = rows(&output);
        assert_eq!(r.len(), 3);
        assert_eq!(r[1]["status"], "failed");
        assert_eq!(r[1]["error"], "injected engine boundary failure");
        assert_eq!(r[1]["winner"], Value::Null);
        assert_eq!(r[1]["decisions"], 0);
        assert_eq!(r[2]["failed"], 1);
        assert_eq!(r[2]["completed"], 0);
        assert_eq!(r[2]["not_started"], 2);
        assert_eq!(r[2]["draws"], 0);
    }
    #[test]
    fn simulate_active_interruption_and_deadline_have_distinct_accounting() {
        for (stop, status, field, code) in [
            (Stop::Sigterm, "unfinished", "unfinished", 143),
            (Stop::Sigint, "unfinished", "unfinished", 130),
            (Stop::Deadline, "truncated", "truncated", 4),
        ] {
            let mut output = vec![];
            let mut polls = 0;
            let exit = run(
                &config(),
                &mut output,
                || {
                    polls += 1;
                    (polls == 3).then_some(stop)
                },
                step,
            )
            .unwrap();
            assert_eq!(exit, code);
            let r = rows(&output);
            assert_eq!(r[1]["episode"], 7);
            assert_eq!(r[1]["decisions"], 1);
            assert_eq!(r[1]["status"], status);
            assert_eq!(r[1]["winner"], Value::Null);
            assert_eq!(r[2][field], 1);
            assert_eq!(r[2]["started"], 1);
            assert_eq!(r[2]["not_started"], 2);
            assert_eq!(r[2]["completed"], 0);
            assert_eq!(r[2]["draws"], 0);
        }
    }
    #[test]
    fn simulate_writer_failure_propagates_instead_of_success() {
        struct Broken {
            remaining: usize,
        }
        impl Write for Broken {
            fn write(&mut self, b: &[u8]) -> io::Result<usize> {
                if self.remaining == 0 {
                    return Err(io::Error::other("disk full"));
                }
                let n = b.len().min(self.remaining);
                self.remaining -= n;
                Ok(n)
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        // Failure during header, later episode records and final footer.
        let mut valid = vec![];
        run(&config(), &mut valid, || None, step).unwrap();
        for limit in [0, 1, valid.len() / 2, valid.len() - 1] {
            let error =
                run(&config(), &mut Broken { remaining: limit }, || None, step).unwrap_err();
            assert_eq!(error.to_string(), "disk full");
        }
    }
    #[test]
    fn simulate_seed_extremes_preserve_rules_derived_passive_outcome() {
        // Opponent hand/library contents cannot affect this policy's selection.
        // Real engine runs from different seeded/shuffled red/green matchups
        // preserve the independently specified passive outcome.
        for seed in [0, 1, u64::MAX] {
            let mut c = config();
            c.master_seed = seed;
            c.episodes = 1;
            let mut output = vec![];
            assert_eq!(run(&c, &mut output, || None, step).unwrap(), 0);
            let r = rows(&output);
            assert_eq!(r[1]["turn"], 68);
            assert_eq!(r[1]["winner"], 0);
            assert_eq!(r[1]["life"], json!([20, 20]));
        }
    }
}
