//! Standalone privileged semantic action records. See `doc/actions.md`.
use super::*;
use policy::{PolicyError, Submission, VisibleRef, VisibleZone};
use serde::{Deserialize, Serialize};
pub const ACTION_VERSION: u32 = 1;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectRef {
    pub birth: u64,
    pub card: String,
    pub owner: Seat,
    pub zone: Zone,
    pub incarnation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Choice {
    Concede {},
    Keep {},
    Mulligan {},
    Bottom {
        card: ObjectRef,
    },
    Pass {},
    PlayLand {
        card: ObjectRef,
    },
    TapMana {
        card: ObjectRef,
    },
    Pay {
        color: u8,
    },
    FinishPayment {},
    CancelPayment {},
    Cast {
        card: ObjectRef,
    },
    Target {
        card: ObjectRef,
    },
    FinishTargets {},
    CancelTargets {},
    SelectAttackers {
        cards: Vec<ObjectRef>,
    },
    SelectBlockers {
        blocks: Vec<(ObjectRef, ObjectRef)>,
    },
    AssignDamage {
        attacker: ObjectRef,
        amounts: Vec<(ObjectRef, u32)>,
    },
    FinishCombat {},
    Discard {
        card: ObjectRef,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub version: u32,
    pub actor: Seat,
    pub decision: String,
    pub choices: Vec<Choice>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ActionError {
    Malformed,
    UnsupportedVersion,
    WrongKind,
    InvalidReference,
    Policy(PolicyError),
    Concession(terminal::ConcedeError),
    InvalidState,
}
#[derive(Debug)]
pub enum Decoded {
    Decision {
        actor: Seat,
        submission: Submission,
    },
    Concession {
        actor: Seat,
        episode: terminal::EpisodeId,
    },
}
fn concession(g: &Game, actor: Seat) -> Result<terminal::EpisodeId, ActionError> {
    let episode = g.episode_id().ok_or(ActionError::InvalidState)?;
    let mut scratch = Game::new().map_err(|_| ActionError::InvalidState)?;
    scratch
        .restore(&g.snapshot())
        .map_err(|_| ActionError::InvalidState)?;
    scratch
        .concede(
            actor,
            scratch.episode_id().ok_or(ActionError::InvalidState)?,
        )
        .map_err(ActionError::Concession)?;
    Ok(episode)
}
/// Concession is an out-of-band command available to either seat, without priority.
pub fn encode_concession(g: &Game, actor: Seat) -> Result<Vec<u8>, ActionError> {
    concession(g, actor)?;
    serde_json::to_vec(&Record {
        version: ACTION_VERSION,
        actor,
        decision: "concession".into(),
        choices: vec![Choice::Concede {}],
    })
    .map_err(|_| ActionError::Malformed)
}
fn decision(g: &Game, actor: Seat, capacity: usize) -> Result<policy::Decision, ActionError> {
    g.policy_observe(actor, capacity)
        .map_err(ActionError::Policy)?
        .decision
        .ok_or(ActionError::Policy(PolicyError::WrongActor))
}
fn visible(g: &Game, actor: Seat, r: &VisibleRef) -> Result<Handle, ActionError> {
    match r.zone {
        VisibleZone::Hand => g.view_hand(actor).get(r.row).copied(),
        VisibleZone::Battlefield => g.objects.in_zone(Zone::Battlefield).nth(r.row),
    }
    .ok_or(ActionError::InvalidReference)
}
fn reference(g: &Game, actor: Seat, r: &VisibleRef) -> Result<ObjectRef, ActionError> {
    let h = visible(g, actor, r)?;
    let o = g
        .objects
        .get(h)
        .map_err(|_| ActionError::InvalidReference)?;
    let (birth, incarnation) = g
        .objects
        .semantic_identity(h)
        .map_err(|_| ActionError::InvalidReference)?;
    Ok(ObjectRef {
        birth,
        incarnation,
        card: o.card.identity().key.into(),
        owner: o.owner,
        zone: o.zone,
    })
}
fn resolve(g: &Game, actor: Seat, r: &ObjectRef) -> Result<VisibleRef, ActionError> {
    let (zone, handles) = match r.zone {
        Zone::Hand(s) if s == actor => (VisibleZone::Hand, g.view_hand(actor)),
        Zone::Battlefield => (
            VisibleZone::Battlefield,
            g.objects.in_zone(Zone::Battlefield).collect(),
        ),
        _ => return Err(ActionError::InvalidReference),
    };
    let row = handles
        .iter()
        .position(|h| {
            let o = g.objects.get(*h).expect("live zone");
            o.owner == r.owner
                && o.card.identity().key == r.card
                && g.objects.semantic_identity(*h) == Ok((r.birth, r.incarnation))
        })
        .ok_or(ActionError::InvalidReference)?;
    Ok(VisibleRef { zone, row })
}
// Validate using the authoritative policy/core path in an isolated snapshot.
// This diagnostic/persistence API prioritizes full transactional validation over
// throughput. It never changes the caller's Game, RNG, or pending selections.
fn validate(g: &Game, actor: Seat, s: &Submission, capacity: usize) -> Result<(), ActionError> {
    let d = decision(g, actor, capacity)?;
    if s.schema_version != policy::SCHEMA_VERSION {
        return Err(ActionError::Policy(PolicyError::UnsupportedVersion));
    }
    if s.revision != d.revision || s.generation != d.generation {
        return Err(ActionError::Policy(PolicyError::StaleDecision));
    }
    let mut scratch = Game::new().map_err(|_| ActionError::InvalidState)?;
    scratch
        .restore(&g.snapshot())
        .map_err(|_| ActionError::InvalidState)?;
    let fresh = decision(&scratch, actor, capacity)?;
    let mut submission = s.clone();
    submission.revision = fresh.revision;
    submission.generation = fresh.generation;
    scratch
        .apply_policy(actor, &submission, capacity)
        .map_err(ActionError::Policy)
}
/// Encode a currently legal submission. Saved bytes contain no local handles,
/// decision revisions/generations or candidate/observation rows.
pub fn encode(
    g: &Game,
    actor: Seat,
    s: &Submission,
    capacity: usize,
) -> Result<Vec<u8>, ActionError> {
    validate(g, actor, s, capacity)?;
    let d = decision(g, actor, capacity)?;
    let choices = s
        .choices
        .iter()
        .map(|c| encode_choice(g, actor, c))
        .collect::<Result<_, _>>()?;
    serde_json::to_vec(&Record {
        version: ACTION_VERSION,
        actor,
        decision: d.kind.into(),
        choices,
    })
    .map_err(|_| ActionError::Malformed)
}
/// Resolve and validate a record without changing the game. The returned live
/// submission expires under the normal policy revision/generation contract.
pub fn decode(g: &Game, bytes: &[u8], capacity: usize) -> Result<Decoded, ActionError> {
    let r: Record = serde_json::from_slice(bytes).map_err(|_| ActionError::Malformed)?;
    if r.version != ACTION_VERSION {
        return Err(ActionError::UnsupportedVersion);
    }
    if r.decision == "concession" {
        if r.choices != [Choice::Concede {}] {
            return Err(ActionError::WrongKind);
        }
        return Ok(Decoded::Concession {
            actor: r.actor,
            episode: concession(g, r.actor)?,
        });
    }
    let d = decision(g, r.actor, capacity)?;
    if r.decision != d.kind {
        return Err(ActionError::WrongKind);
    }
    let choices = r
        .choices
        .iter()
        .map(|c| decode_choice(g, r.actor, c))
        .collect::<Result<_, _>>()?;
    let submission = Submission {
        schema_version: policy::SCHEMA_VERSION,
        revision: d.revision,
        generation: d.generation,
        choices,
    };
    validate(g, r.actor, &submission, capacity)?;
    Ok(Decoded::Decision {
        actor: r.actor,
        submission,
    })
}
/// Decode, validate and apply through the existing policy/core path.
pub fn apply(g: &mut Game, bytes: &[u8], capacity: usize) -> Result<(), ActionError> {
    match decode(g, bytes, capacity)? {
        Decoded::Decision { actor, submission } => g
            .apply_policy(actor, &submission, capacity)
            .map_err(ActionError::Policy),
        Decoded::Concession { actor, episode } => g
            .concede(actor, episode)
            .map(|_| ())
            .map_err(ActionError::Concession),
    }
}
fn encode_choice(g: &Game, actor: Seat, c: &policy::Choice) -> Result<Choice, ActionError> {
    let map = |r| reference(g, actor, r);
    Ok(match c {
        policy::Choice::Keep => Choice::Keep {},
        policy::Choice::Mulligan => Choice::Mulligan {},
        policy::Choice::Pass => Choice::Pass {},
        policy::Choice::FinishPayment => Choice::FinishPayment {},
        policy::Choice::CancelPayment => Choice::CancelPayment {},
        policy::Choice::FinishTargets => Choice::FinishTargets {},
        policy::Choice::CancelTargets => Choice::CancelTargets {},
        policy::Choice::FinishCombat => Choice::FinishCombat {},
        policy::Choice::Bottom { card } => Choice::Bottom { card: map(card)? },
        policy::Choice::PlayLand { card } => Choice::PlayLand { card: map(card)? },
        policy::Choice::TapMana { card } => Choice::TapMana { card: map(card)? },
        policy::Choice::Cast { card } => Choice::Cast { card: map(card)? },
        policy::Choice::Target { card } => Choice::Target { card: map(card)? },
        policy::Choice::Discard { card } => Choice::Discard { card: map(card)? },
        policy::Choice::Pay { color } => Choice::Pay { color: *color },
        policy::Choice::SelectAttackers { cards } => Choice::SelectAttackers {
            cards: cards.iter().map(map).collect::<Result<_, _>>()?,
        },
        policy::Choice::SelectBlockers { blocks } => Choice::SelectBlockers {
            blocks: blocks
                .iter()
                .map(|(b, a)| Ok((map(b)?, map(a)?)))
                .collect::<Result<_, ActionError>>()?,
        },
        policy::Choice::AssignDamage { attacker, amounts } => Choice::AssignDamage {
            attacker: map(attacker)?,
            amounts: amounts
                .iter()
                .map(|(b, n)| Ok((map(b)?, *n)))
                .collect::<Result<_, ActionError>>()?,
        },
        policy::Choice::Spell | policy::Choice::Combat => return Err(ActionError::WrongKind),
    })
}
fn decode_choice(g: &Game, actor: Seat, c: &Choice) -> Result<policy::Choice, ActionError> {
    let map = |r| resolve(g, actor, r);
    Ok(match c {
        Choice::Concede {} => return Err(ActionError::WrongKind),
        Choice::Keep {} => policy::Choice::Keep,
        Choice::Mulligan {} => policy::Choice::Mulligan,
        Choice::Pass {} => policy::Choice::Pass,
        Choice::FinishPayment {} => policy::Choice::FinishPayment,
        Choice::CancelPayment {} => policy::Choice::CancelPayment,
        Choice::FinishTargets {} => policy::Choice::FinishTargets,
        Choice::CancelTargets {} => policy::Choice::CancelTargets,
        Choice::FinishCombat {} => policy::Choice::FinishCombat,
        Choice::Bottom { card } => policy::Choice::Bottom { card: map(card)? },
        Choice::PlayLand { card } => policy::Choice::PlayLand { card: map(card)? },
        Choice::TapMana { card } => policy::Choice::TapMana { card: map(card)? },
        Choice::Cast { card } => policy::Choice::Cast { card: map(card)? },
        Choice::Target { card } => policy::Choice::Target { card: map(card)? },
        Choice::Discard { card } => policy::Choice::Discard { card: map(card)? },
        Choice::Pay { color } => policy::Choice::Pay { color: *color },
        Choice::SelectAttackers { cards } => policy::Choice::SelectAttackers {
            cards: cards.iter().map(map).collect::<Result<_, _>>()?,
        },
        Choice::SelectBlockers { blocks } => policy::Choice::SelectBlockers {
            blocks: blocks
                .iter()
                .map(|(b, a)| Ok((map(b)?, map(a)?)))
                .collect::<Result<_, ActionError>>()?,
        },
        Choice::AssignDamage { attacker, amounts } => policy::Choice::AssignDamage {
            attacker: map(attacker)?,
            amounts: amounts
                .iter()
                .map(|(b, n)| Ok((map(b)?, *n)))
                .collect::<Result<_, ActionError>>()?,
        },
    })
}
#[cfg(test)]
#[path = "actions_tests.rs"]
mod tests;
