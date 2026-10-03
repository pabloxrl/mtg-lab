//! Owned storage schema corresponding to mtg_core::trajectory and views v1.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisibleCard {
    pub card: String,
    pub owner: u8,
    pub controller: u8,
    pub tapped: bool,
    pub creature: Option<[u32; 3]>,
    pub summoning_sick: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub haste: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicZone {
    pub zone: String,
    pub cards: Vec<VisibleCard>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revelation {
    pub card: String,
    pub owner: u8,
    /// Historical zone only; never refreshed from the current hidden state.
    pub zone_at_reveal: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningView {
    pub generation: u64,
    pub kind: String,
    pub count: usize,
    /// Ordered legal rows: keep/mulligan, or card keys for ordered bottoming.
    pub candidates: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerView {
    pub schema_version: u32,
    pub seat: u8,
    pub life: [i64; 2],
    pub hand_counts: [usize; 2],
    pub library_counts: [usize; 2],
    pub hand: Vec<VisibleCard>,
    pub public_zones: Vec<PublicZone>,
    pub remembered: Vec<Revelation>,
    pub starting_seat: u8,
    /// Turn number, active seat and step; absent during opening.
    pub turn: Option<(u64, u8, String)>,
    pub mana: [[u32; 6]; 2],
    pub acting_seat: Option<u8>,
    pub opening: Option<OpeningView>,
    pub terminal: Option<TerminalView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalView {
    pub winner: Option<u8>,
    pub losses: [Option<String>; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Versions {
    pub schema: u32,
    pub engine: String,
    pub rules: String,
    pub cards: String,
    pub action: String,
    pub observation: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeKey {
    /// Collector-generated globally unique run UUID; never a seed.
    pub run: String,
    pub ordinal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub id: EpisodeKey,
    pub versions: Versions,
    pub deck_hashes: [String; 2],
    pub config_hash: String,
    pub policies: [String; 2],
    pub starting_seat: u8,
    pub limits: Limits,
    /// Opaque reference to separately permissioned seeds/replay configuration.
    pub restricted_replay: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub decisions: Option<u64>,
    pub turns: Option<u64>,
    pub wall_time_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub semantic: String,
    pub features: Vec<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyInfo {
    pub checkpoint: Option<String>,
    pub log_probability: Option<f64>,
    pub value: Option<f64>,
    pub recurrent_state: Option<String>,
    pub exploration: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub kind: String,
    pub logical_action: u64,
    pub micro_choice: u64,
    pub candidates: Vec<Candidate>,
    pub legal_mask: Vec<bool>,
    pub selected: usize,
    pub policy: PolicyInfo,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum End {
    Completed,
    Truncated(Limit),
    Failed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Limit {
    Decisions,
    Turns,
    WallTime,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub episode: EpisodeKey,
    pub index: usize,
    pub seat_index: usize,
    pub actor: u8,
    pub observation: PlayerView,
    pub choice: Choice,
    pub action: String,
    pub reward: [i8; 2],
    pub next_actor: Option<u8>,
    pub terminated: bool,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Footer {
    pub end: End,
    pub complete: bool,
    pub returns: [i8; 2],
    /// Reward from an ending outside a recorded policy action (e.g. concession).
    pub boundary_reward: [i8; 2],
    pub decisions: usize,
    pub logical_actions: usize,
    pub final_observations: [PlayerView; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RewardConvention {
    SparseZeroSumTerminal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DiscountConvention {
    UndiscountedEpisodic,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CaptureSelection {
    AllDecisions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Episode {
    pub reward_convention: RewardConvention,
    pub discount_convention: DiscountConvention,
    pub capture_selection: CaptureSelection,
    pub header: Header,
    pub decisions: Vec<Decision>,
    pub footer: Option<Footer>,
}
