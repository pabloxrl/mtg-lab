//! Strict owned durable counterpart of canonical trajectory v2 and policy v1.
use crate::{
    Error,
    schema::{
        CaptureSelection, DiscountConvention, End, EpisodeKey, Header, PlayerView, RewardConvention,
    },
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum VisibleZone {
    Hand,
    Battlefield,
}
/// Row in the authorized observation, valid only with its decision generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisibleRef {
    pub zone: VisibleZone,
    pub row: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Keep,
    Mulligan,
    Bottom {
        card: VisibleRef,
    },
    Pass,
    PlayLand {
        card: VisibleRef,
    },
    Activate {
        card: VisibleRef,
    },
    FinishActivation,
    CancelActivation,
    TapMana {
        card: VisibleRef,
    },
    Pay {
        color: u8,
    },
    Mode {
        mode: u8,
    },
    FinishPayment,
    CancelPayment,
    Cast {
        card: VisibleRef,
    },
    Target {
        card: VisibleRef,
    },
    FinishTargets,
    CancelTargets,
    SelectAttackers {
        cards: Vec<VisibleRef>,
    },
    SelectBlockers {
        blocks: Vec<(VisibleRef, VisibleRef)>,
    },
    AssignDamage {
        attacker: VisibleRef,
        amounts: Vec<(VisibleRef, u32)>,
    },
    FinishCombat,
    Discard {
        card: VisibleRef,
    },
    Spell,
    Combat,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    pub revision: u64,
    pub schema_version: u32,
    pub generation: u64,
    pub choices: Vec<Command>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    pub revision: u64,
    pub generation: u64,
    pub actor: u8,
    pub kind: String,
    pub count: usize,
    pub candidates: Vec<Command>,
    pub legal_mask: Vec<bool>,
    pub factored: Option<CombatChoices>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub schema_version: u32,
    pub view: PlayerView,
    pub decision: Option<Domain>,
    pub pending: Option<PendingSpell>,
    pub stack: Vec<StackSpell>,
    pub combat: Vec<CombatAttack>,
    pub unsupported_families: [String; 0],
}
/// Factored domains, not a table of every subset/map/integer composition.
/// Present only for the actor at a combat decision. All rows use the current view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatChoices {
    pub attackers: Vec<VisibleRef>,
    pub blockers: Vec<VisibleRef>,
    pub selected: Vec<VisibleRef>,
    pub blocks: Vec<(VisibleRef, VisibleRef)>,
    /// Pairs excluded from the attacker/blocker Cartesian product; not whole maps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_blocks: Vec<(VisibleRef, VisibleRef)>,
    pub damage: Vec<DamageAllocation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageAllocation {
    pub attacker: VisibleRef,
    pub power: u32,
    pub blockers: Vec<VisibleRef>,
    /// Lethal damage needed per blocker; remainder may go to defender only when all are met.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trample_lethal: Option<Vec<u32>>,
    pub amounts: Option<Vec<(VisibleRef, u32)>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatAttack {
    pub attacker: VisibleRef,
    pub blocked: bool,
    pub blockers: Vec<VisibleRef>,
}
/// Actor-only provisional state. References use the accompanying visible rows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingSpell {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<u8>,
    pub card: VisibleRef,
    pub targets: Vec<Option<VisibleRef>>,
    pub sources: Vec<Option<VisibleRef>>,
    pub pool: Option<[u32; 6]>,
    pub remaining: Option<ManaCost>,
}
/// Bottom-to-top committed stack. A departed target is null, never rebound to
/// another object or looked up in a hidden zone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackSpell {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<u8>,
    pub row: usize,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ability: bool,
    pub targets: Vec<Option<VisibleRef>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManaCost {
    pub colored: [u32; 6],
    pub generic: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_probability: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurrent_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exploration: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ActionStatus {
    Continuing,
    Committed,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub submission: Submission,
    pub logical_action: u64,
    pub micro_choice: u64,
    pub status: ActionStatus,
    pub policy: PolicyInfo,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub episode: EpisodeKey,
    pub index: usize,
    pub seat_index: usize,
    pub actor: u8,
    pub observation: Observation,
    pub choice: Choice,
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
    pub cancelled_actions: usize,
    pub final_observations: [Observation; 2],
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

/// Validate the stored authorized domain, never infer hidden game state or replay.
pub fn validate(e: &Episode) -> Result<(), Error> {
    if e.header.versions.schema != 2 {
        return Err(Error::Invalid);
    }
    // Reuse the v1 metadata, view, identity, boundary and reward accounting
    // validator internally. The structured domain below is validated separately;
    // this accounting projection is never serialized or exposed to consumers.
    crate::validate(&accounting(e))?;
    let f = e.footer.as_ref().ok_or(Error::Incomplete)?;
    if f.cancelled_actions
        != e.decisions
            .iter()
            .filter(|d| d.choice.status == ActionStatus::Cancelled)
            .count()
        || !f.final_observations.iter().all(observation_valid)
    {
        return Err(Error::Invalid);
    }
    for (i, d) in e.decisions.iter().enumerate() {
        let o = &d.observation;
        if !observation_valid(o)
            || !valid_submission(o, &d.choice.submission)
            || d.choice.status != action_status(o, &d.choice.submission)
        {
            return Err(Error::Invalid);
        }
        if i > 0 {
            let prev = &e.decisions[i - 1];
            let sequence = if prev.choice.status == ActionStatus::Continuing {
                prev.actor == d.actor
                    && prev.choice.logical_action == d.choice.logical_action
                    && prev.choice.micro_choice.checked_add(1) == Some(d.choice.micro_choice)
            } else {
                prev.choice.logical_action.checked_add(1) == Some(d.choice.logical_action)
                    && d.choice.micro_choice == 0
            };
            let old = prev.observation.decision.as_ref().ok_or(Error::Invalid)?;
            let new = o.decision.as_ref().ok_or(Error::Invalid)?;
            if !sequence || (new.revision, new.generation) <= (old.revision, old.generation) {
                return Err(Error::Invalid);
            }
        }
    }
    if let Some(last) = e.decisions.last() {
        let old = last.observation.decision.as_ref().ok_or(Error::Invalid)?;
        for next in f
            .final_observations
            .iter()
            .filter_map(|o| o.decision.as_ref())
        {
            if (next.revision, next.generation) <= (old.revision, old.generation) {
                return Err(Error::Invalid);
            }
        }
    }
    Ok(())
}
fn accounting(e: &Episode) -> crate::schema::Episode {
    use crate::schema as v1;
    let mut header = e.header.clone();
    header.versions.schema = 1;
    v1::Episode {
        reward_convention: e.reward_convention.clone(),
        discount_convention: e.discount_convention.clone(),
        capture_selection: e.capture_selection.clone(),
        header,
        decisions: e
            .decisions
            .iter()
            .map(|d| v1::Decision {
                episode: d.episode.clone(),
                index: d.index,
                seat_index: d.seat_index,
                actor: d.actor,
                observation: d.observation.view.clone(),
                choice: v1::Choice {
                    kind: "structured".into(),
                    logical_action: d.choice.logical_action,
                    micro_choice: d.choice.micro_choice,
                    candidates: vec![v1::Candidate {
                        semantic: "structured".into(),
                        features: vec![],
                    }],
                    legal_mask: vec![true],
                    selected: 0,
                    policy: v1::PolicyInfo {
                        checkpoint: d.choice.policy.checkpoint.clone(),
                        log_probability: d.choice.policy.log_probability,
                        value: d.choice.policy.value,
                        recurrent_state: d.choice.policy.recurrent_state.clone(),
                        exploration: d.choice.policy.exploration.clone(),
                    },
                },
                action: "structured".into(),
                reward: d.reward,
                next_actor: d.next_actor,
                terminated: d.terminated,
                truncated: d.truncated,
            })
            .collect(),
        footer: e.footer.as_ref().map(|f| v1::Footer {
            end: f.end.clone(),
            complete: f.complete,
            returns: f.returns,
            boundary_reward: f.boundary_reward,
            decisions: f.decisions,
            logical_actions: f.logical_actions,
            final_observations: f.final_observations.clone().map(|o| o.view),
        }),
    }
}
fn unique<T: PartialEq>(items: &[T]) -> bool {
    items
        .iter()
        .enumerate()
        .all(|(i, x)| !items[..i].contains(x))
}
fn valid_ref(o: &Observation, r: &VisibleRef, zone: VisibleZone) -> bool {
    r.zone == zone
        && r.row
            < match zone {
                VisibleZone::Hand => o.view.hand.len(),
                VisibleZone::Battlefield => o
                    .view
                    .public_zones
                    .iter()
                    .find(|z| z.zone == "battlefield")
                    .map_or(0, |z| z.cards.len()),
            }
}
fn command_valid(o: &Observation, c: &Command) -> bool {
    let hand = |r| valid_ref(o, r, VisibleZone::Hand);
    let field = |r| valid_ref(o, r, VisibleZone::Battlefield);
    match c {
        Command::Bottom { card }
        | Command::Discard { card }
        | Command::Cast { card }
        | Command::PlayLand { card } => hand(card),
        Command::Activate { card } | Command::Target { card } | Command::TapMana { card } => {
            field(card)
        }
        Command::Mode { mode } => *mode <= 1,
        Command::Pay { color } => *color < 6,
        Command::SelectAttackers { cards } => unique(cards) && cards.iter().all(field),
        Command::SelectBlockers { blocks } => {
            unique(&blocks.iter().map(|(b, _)| b).collect::<Vec<_>>())
                && blocks.iter().all(|(b, a)| field(b) && field(a))
        }
        Command::AssignDamage { attacker, amounts } => {
            field(attacker)
                && unique(&amounts.iter().map(|(b, _)| b).collect::<Vec<_>>())
                && amounts.iter().all(|(b, _)| field(b))
        }
        _ => true,
    }
}
fn observation_valid(o: &Observation) -> bool {
    let field = |r| valid_ref(o, r, VisibleZone::Battlefield);
    let actor = o.view.acting_seat == Some(o.view.seat);
    if o.schema_version != 1
        || !crate::view(&o.view, o.view.seat, o.view.starting_seat)
        || o.decision.is_some() != actor
        || o.pending.is_some() && !actor
        || !unique(
            &o.view
                .public_zones
                .iter()
                .map(|z| &z.zone)
                .collect::<Vec<_>>(),
        )
    {
        return false;
    }
    if let Some(p) = &o.pending
        && (!(if o
            .decision
            .as_ref()
            .is_some_and(|d| matches!(d.kind.as_str(), "activation_target" | "activation_payment"))
        {
            field(&p.card)
        } else {
            valid_ref(o, &p.card, VisibleZone::Hand)
        }) || !p.targets.iter().flatten().all(field)
            || !p.sources.iter().flatten().all(field)
            || p.pool.is_some() != p.remaining.is_some()
            || p.mode.is_some_and(|m| m > 1))
    {
        return false;
    }
    let stack_len = o
        .view
        .public_zones
        .iter()
        .find(|z| z.zone == "stack")
        .map_or(0, |z| z.cards.len());
    if o.stack.len() != stack_len
        || !o.stack.iter().enumerate().all(|(i, s)| {
            s.row == i && s.mode.is_none_or(|m| m <= 1) && s.targets.iter().flatten().all(field)
        })
        || !unique(&o.combat.iter().map(|a| a.attacker).collect::<Vec<_>>())
        || !o.combat.iter().all(|a| {
            field(&a.attacker)
                && unique(&a.blockers)
                && a.blockers.iter().all(field)
                && (a.blocked || a.blockers.is_empty())
        })
    {
        return false;
    }
    if let Some(d) = &o.decision {
        let combat = ["attackers", "blockers", "combat_damage"].contains(&d.kind.as_str());
        let pending = [
            "cast_mode",
            "cast_discard",
            "activation_target",
            "activation_payment",
            "growth_target",
            "bite_source",
            "bite_destination",
            "targets_complete",
        ]
        .contains(&d.kind.as_str());
        if d.actor != o.view.seat
            || ![
                "keep_or_mulligan",
                "bottom",
                "priority",
                "activation_target",
                "activation_payment",
                "growth_target",
                "bite_source",
                "bite_destination",
                "targets_complete",
                "cast_mode",
                "cast_discard",
                "payment",
                "attackers",
                "blockers",
                "combat_damage",
                "cleanup_discard",
            ]
            .contains(&d.kind.as_str())
            || d.count == 0
            || (!matches!(d.kind.as_str(), "bottom" | "cleanup_discard") && d.count != 1)
            || d.candidates.is_empty()
            || d.legal_mask.len() != d.candidates.len()
            || !unique(&d.candidates)
            || !d
                .candidates
                .iter()
                .all(|c| command_valid(o, c) && kind_command(&d.kind, c))
            || combat != d.factored.is_some()
            || (d.kind != "payment" && pending != o.pending.is_some())
            || (d.kind == "payment" && o.pending.as_ref().is_some_and(|p| p.pool.is_none()))
        {
            return false;
        }
        if let Some(f) = &d.factored
            && (!unique(&f.forbidden_blocks)
                || !f
                    .forbidden_blocks
                    .iter()
                    .all(|(b, a)| f.blockers.contains(b) && f.attackers.contains(a))
                || !unique(&f.attackers)
                || !unique(&f.blockers)
                || !unique(&f.selected)
                || !f
                    .attackers
                    .iter()
                    .chain(&f.blockers)
                    .chain(&f.selected)
                    .all(field)
                || !f.selected.iter().all(|r| f.attackers.contains(r))
                || !unique(&f.blocks.iter().map(|(b, _)| b).collect::<Vec<_>>())
                || !f.blocks.iter().all(|(b, a)| {
                    f.blockers.contains(b)
                        && f.attackers.contains(a)
                        && !f.forbidden_blocks.contains(&(*b, *a))
                })
                || !unique(&f.damage.iter().map(|d| d.attacker).collect::<Vec<_>>())
                || !f.damage.iter().all(|d| {
                    field(&d.attacker)
                        && unique(&d.blockers)
                        && d.blockers.iter().all(field)
                        && d.trample_lethal
                            .as_ref()
                            .is_none_or(|l| l.len() == d.blockers.len())
                        && d.amounts.as_ref().is_none_or(|a| allocation(d, a))
                }))
        {
            return false;
        }
    }
    true
}
fn allocation(d: &DamageAllocation, amounts: &[(VisibleRef, u32)]) -> bool {
    let Some(total) = amounts
        .iter()
        .try_fold(0u32, |sum, (_, n)| sum.checked_add(*n))
    else {
        return false;
    };
    unique(&amounts.iter().map(|(b, _)| b).collect::<Vec<_>>())
        && amounts.iter().all(|(b, _)| d.blockers.contains(b))
        && total <= d.power
        && (total == d.power
            || d.trample_lethal.as_ref().is_some_and(|lethal| {
                lethal.len() == d.blockers.len()
                    && d.blockers.iter().zip(lethal).all(|(b, need)| {
                        amounts.iter().find(|(h, _)| h == b).map_or(0, |(_, n)| *n) >= *need
                    })
            }))
}

fn valid_submission(o: &Observation, s: &Submission) -> bool {
    let Some(d) = &o.decision else {
        return false;
    };
    if s.schema_version != 1
        || s.revision != d.revision
        || s.generation != d.generation
        || s.choices.len() != d.count
        || !s
            .choices
            .iter()
            .all(|c| command_valid(o, c) && kind_command(&d.kind, c))
    {
        return false;
    }
    if let (Some(f), [c]) = (&d.factored, s.choices.as_slice()) {
        match c {
            Command::SelectAttackers { cards } if d.kind == "attackers" => {
                return unique(cards) && cards.iter().all(|r| f.attackers.contains(r));
            }
            Command::SelectBlockers { blocks } if d.kind == "blockers" => {
                return unique(&blocks.iter().map(|(b, _)| b).collect::<Vec<_>>())
                    && blocks.iter().all(|(b, a)| {
                        f.blockers.contains(b)
                            && f.attackers.contains(a)
                            && !f.forbidden_blocks.contains(&(*b, *a))
                    });
            }
            Command::AssignDamage { attacker, amounts } if d.kind == "combat_damage" => {
                return f
                    .damage
                    .iter()
                    .find(|d| d.attacker == *attacker)
                    .is_some_and(|d| allocation(d, amounts));
            }
            _ => (),
        }
    }
    unique(&s.choices)
        && s.choices.iter().all(|c| {
            d.candidates
                .iter()
                .enumerate()
                .any(|(i, x)| x == c && d.legal_mask[i])
        })
}
fn action_status(o: &Observation, s: &Submission) -> ActionStatus {
    match s.choices.first() {
        Some(Command::CancelPayment | Command::CancelTargets | Command::CancelActivation) => {
            ActionStatus::Cancelled
        }
        Some(
            Command::Mode { .. }
            | Command::Cast { .. }
            | Command::Activate { .. }
            | Command::Target { .. }
            | Command::FinishTargets
            | Command::Pay { .. }
            | Command::SelectAttackers { .. }
            | Command::SelectBlockers { .. }
            | Command::AssignDamage { .. },
        ) => ActionStatus::Continuing,
        Some(Command::TapMana { .. } | Command::Discard { .. }) if o.pending.is_some() => {
            ActionStatus::Continuing
        }
        _ => ActionStatus::Committed,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub episode: EpisodeKey,
    pub seat: u8,
    pub seat_index: usize,
    pub decision_index: usize,
    pub next_decision: Option<usize>,
    pub observation: Observation,
    pub next_observation: Observation,
    pub choice: Choice,
    pub reward: i8,
    pub decisions_elapsed: usize,
    pub logical_actions_elapsed: usize,
    pub cancelled_actions_elapsed: usize,
    pub terminated: bool,
    pub truncated: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeatSequence {
    pub versions: crate::schema::Versions,
    pub discount_convention: DiscountConvention,
    pub episode: EpisodeKey,
    pub seat: u8,
    pub transitions: Vec<Transition>,
    pub final_observation: Observation,
    /// Sole reward-bearing record if this seat made no decisions; otherwise zero.
    pub unassigned_reward: i8,
    pub total_return: i8,
    pub end: End,
}

impl Episode {
    pub fn seat(&self, seat: u8) -> Result<SeatSequence, Error> {
        validate(self)?;
        if seat > 1 {
            return Err(Error::Invalid);
        }
        let f = self
            .footer
            .as_ref()
            .filter(|f| f.complete && !matches!(f.end, End::Failed(_)))
            .ok_or(Error::Incomplete)?;
        let s = seat as usize;
        let rows: Vec<_> = self
            .decisions
            .iter()
            .filter(|d| d.actor as usize == s)
            .collect();
        let transitions = rows
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let next = rows.get(i + 1);
                let end = next.map_or(self.decisions.len(), |d| d.index);
                let interval = &self.decisions[d.index..end];
                Transition {
                    episode: self.header.id.clone(),
                    seat: s as u8,
                    seat_index: i,
                    decision_index: d.index,
                    next_decision: next.map(|d| d.index),
                    observation: d.observation.clone(),
                    next_observation: next.map_or_else(
                        || f.final_observations[s].clone(),
                        |d| d.observation.clone(),
                    ),
                    choice: d.choice.clone(),
                    reward: interval.iter().map(|d| d.reward[s]).sum::<i8>()
                        + if next.is_none() {
                            f.boundary_reward[s]
                        } else {
                            0
                        },
                    decisions_elapsed: interval.len(),
                    logical_actions_elapsed: action_count(interval),
                    cancelled_actions_elapsed: cancellation_count(interval),
                    terminated: next.is_none() && matches!(f.end, End::Completed),
                    truncated: next.is_none() && matches!(f.end, End::Truncated(_)),
                }
            })
            .collect();
        Ok(SeatSequence {
            versions: self.header.versions.clone(),
            discount_convention: self.discount_convention.clone(),
            episode: self.header.id.clone(),
            seat: s as u8,
            transitions,
            final_observation: f.final_observations[s].clone(),
            unassigned_reward: if rows.is_empty() { f.returns[s] } else { 0 },
            total_return: f.returns[s],
            end: f.end.clone(),
        })
    }
}
fn action_count(decisions: &[Decision]) -> usize {
    decisions
        .iter()
        .enumerate()
        .filter(|(i, d)| {
            *i == 0 || decisions[*i - 1].choice.logical_action != d.choice.logical_action
        })
        .count()
}
fn cancellation_count(decisions: &[Decision]) -> usize {
    decisions
        .iter()
        .filter(|d| d.choice.status == ActionStatus::Cancelled)
        .count()
}

fn kind_command(kind: &str, command: &Command) -> bool {
    matches!(
        (kind, command),
        ("keep_or_mulligan", Command::Keep | Command::Mulligan)
            | ("bottom", Command::Bottom { .. })
            | ("cast_mode", Command::Mode { .. } | Command::CancelPayment)
            | ("cleanup_discard", Command::Discard { .. })
            | (
                "cast_discard",
                Command::Discard { .. } | Command::CancelPayment
            )
            | (
                "priority",
                Command::Pass
                    | Command::PlayLand { .. }
                    | Command::TapMana { .. }
                    | Command::Cast { .. }
                    | Command::Activate { .. }
            )
            | (
                "activation_target",
                Command::Target { .. } | Command::FinishActivation | Command::CancelActivation
            )
            | (
                "growth_target" | "bite_source" | "bite_destination" | "targets_complete",
                Command::Target { .. } | Command::FinishTargets | Command::CancelTargets
            )
            | (
                "activation_payment",
                Command::Pay { .. } | Command::FinishActivation | Command::CancelActivation
            )
            | (
                "payment",
                Command::Pay { .. }
                    | Command::TapMana { .. }
                    | Command::FinishPayment
                    | Command::CancelPayment
            )
            | (
                "attackers",
                Command::SelectAttackers { .. } | Command::FinishCombat
            )
            | (
                "blockers",
                Command::SelectBlockers { .. } | Command::FinishCombat
            )
            | (
                "combat_damage",
                Command::AssignDamage { .. } | Command::FinishCombat
            )
    )
}
