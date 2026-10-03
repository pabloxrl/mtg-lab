//! Explicit core-to-wire copies. Core owns authorized views; schema/structured
//! own durable contracts and validation. No game state or serialization here.
use crate::{schema as s, structured as w};
use mtg_core::{
    game::{mana, policy as p, views as v},
    trajectory as t,
};
use t::v2;
fn s_visible_card(value: &v::VisibleCard) -> s::VisibleCard {
    let v::VisibleCard {
        card,
        owner,
        controller,
        tapped,
        creature,
        summoning_sick,
        haste,
        trample,
    } = value;
    s::VisibleCard {
        card: card.to_string(),
        owner: *owner,
        controller: *controller,
        tapped: *tapped,
        creature: *creature,
        summoning_sick: *summoning_sick,
        haste: *haste,
        trample: *trample,
    }
}
fn s_public_zone(value: &v::PublicZone) -> s::PublicZone {
    let v::PublicZone { zone, cards } = value;
    s::PublicZone {
        zone: zone.to_string(),
        cards: cards.iter().map(s_visible_card).collect(),
    }
}
fn s_revelation(value: &v::Revelation) -> s::Revelation {
    let v::Revelation {
        card,
        owner,
        zone_at_reveal,
    } = value;
    s::Revelation {
        card: card.to_string(),
        owner: *owner,
        zone_at_reveal: zone_at_reveal.to_string(),
    }
}
fn s_opening_view(value: &v::OpeningView) -> s::OpeningView {
    let v::OpeningView {
        generation,
        kind,
        count,
        candidates,
    } = value;
    s::OpeningView {
        generation: *generation,
        kind: kind.to_string(),
        count: *count,
        candidates: candidates.iter().map(|x| x.to_string()).collect(),
    }
}
fn s_player_view(value: &v::PlayerView) -> s::PlayerView {
    let v::PlayerView {
        schema_version,
        seat,
        life,
        hand_counts,
        library_counts,
        hand,
        public_zones,
        remembered,
        starting_seat,
        turn,
        mana,
        acting_seat,
        opening,
        terminal,
    } = value;
    s::PlayerView {
        schema_version: *schema_version,
        seat: *seat,
        life: *life,
        hand_counts: *hand_counts,
        library_counts: *library_counts,
        hand: hand.iter().map(s_visible_card).collect(),
        public_zones: public_zones.iter().map(s_public_zone).collect(),
        remembered: remembered.iter().map(s_revelation).collect(),
        starting_seat: *starting_seat,
        turn: turn.as_ref().map(|x| (x.0, x.1, x.2.to_string())),
        mana: *mana,
        acting_seat: *acting_seat,
        opening: opening.as_ref().map(s_opening_view),
        terminal: terminal.as_ref().map(s_terminal_view),
    }
}
fn s_terminal_view(value: &v::TerminalView) -> s::TerminalView {
    let v::TerminalView { winner, losses } = value;
    s::TerminalView {
        winner: *winner,
        losses: std::array::from_fn(|i| losses[i].as_ref().map(|x| x.to_string())),
    }
}
fn s_versions(value: &t::Versions) -> s::Versions {
    let t::Versions {
        schema,
        engine,
        rules,
        cards,
        action,
        observation,
    } = value;
    s::Versions {
        schema: *schema,
        engine: engine.to_string(),
        rules: rules.to_string(),
        cards: cards.to_string(),
        action: action.to_string(),
        observation: *observation,
    }
}
fn s_episode_key(value: &t::EpisodeKey) -> s::EpisodeKey {
    let t::EpisodeKey { run, ordinal } = value;
    s::EpisodeKey {
        run: run.to_string(),
        ordinal: *ordinal,
    }
}
pub(crate) fn s_header(value: &t::Header) -> s::Header {
    let t::Header {
        id,
        versions,
        deck_hashes,
        config_hash,
        policies,
        starting_seat,
        limits,
        restricted_replay,
    } = value;
    s::Header {
        id: s_episode_key(id),
        versions: s_versions(versions),
        deck_hashes: std::array::from_fn(|i| deck_hashes[i].to_string()),
        config_hash: config_hash.to_string(),
        policies: std::array::from_fn(|i| policies[i].to_string()),
        starting_seat: *starting_seat,
        limits: s_limits(limits),
        restricted_replay: restricted_replay.as_ref().map(|x| x.to_string()),
    }
}
fn s_limits(value: &t::Limits) -> s::Limits {
    let t::Limits {
        decisions,
        turns,
        wall_time_ms,
    } = value;
    s::Limits {
        decisions: *decisions,
        turns: *turns,
        wall_time_ms: *wall_time_ms,
    }
}
fn s_candidate(value: &t::Candidate) -> s::Candidate {
    let t::Candidate { semantic, features } = value;
    s::Candidate {
        semantic: semantic.to_string(),
        features: features.clone(),
    }
}
fn s_policy_info(value: &t::PolicyInfo) -> s::PolicyInfo {
    let t::PolicyInfo {
        checkpoint,
        log_probability,
        value,
        recurrent_state,
        exploration,
    } = value;
    s::PolicyInfo {
        checkpoint: checkpoint.as_ref().map(|x| x.to_string()),
        log_probability: *log_probability,
        value: *value,
        recurrent_state: recurrent_state.as_ref().map(|x| x.to_string()),
        exploration: exploration.as_ref().map(|x| x.to_string()),
    }
}
fn s_choice(value: &t::Choice) -> s::Choice {
    let t::Choice {
        kind,
        logical_action,
        micro_choice,
        candidates,
        legal_mask,
        selected,
        policy,
    } = value;
    s::Choice {
        kind: kind.to_string(),
        logical_action: *logical_action,
        micro_choice: *micro_choice,
        candidates: candidates.iter().map(s_candidate).collect(),
        legal_mask: legal_mask.clone(),
        selected: *selected,
        policy: s_policy_info(policy),
    }
}
fn s_decision(value: &t::Decision) -> s::Decision {
    let t::Decision {
        episode,
        index,
        seat_index,
        actor,
        observation,
        choice,
        action,
        reward,
        next_actor,
        terminated,
        truncated,
    } = value;
    s::Decision {
        episode: s_episode_key(episode),
        index: *index,
        seat_index: *seat_index,
        actor: *actor,
        observation: s_player_view(observation),
        choice: s_choice(choice),
        action: action.to_string(),
        reward: *reward,
        next_actor: *next_actor,
        terminated: *terminated,
        truncated: *truncated,
    }
}
fn s_footer(value: &t::Footer) -> s::Footer {
    let t::Footer {
        end,
        complete,
        returns,
        boundary_reward,
        decisions,
        logical_actions,
        final_observations,
    } = value;
    s::Footer {
        end: s_end(end),
        complete: *complete,
        returns: *returns,
        boundary_reward: *boundary_reward,
        decisions: *decisions,
        logical_actions: *logical_actions,
        final_observations: std::array::from_fn(|i| s_player_view(&final_observations[i])),
    }
}
fn w_visible_ref(value: &p::VisibleRef) -> w::VisibleRef {
    let p::VisibleRef { zone, row } = value;
    w::VisibleRef {
        zone: w_visible_zone(zone),
        row: *row,
    }
}
fn w_submission(value: &p::Submission) -> w::Submission {
    let p::Submission {
        revision,
        schema_version,
        generation,
        choices,
    } = value;
    w::Submission {
        revision: *revision,
        schema_version: *schema_version,
        generation: *generation,
        choices: choices.iter().map(w_command).collect(),
    }
}
pub(crate) fn w_observation(value: &p::Observation) -> w::Observation {
    let p::Observation {
        schema_version,
        view,
        decision,
        pending,
        stack,
        combat,
        unsupported_families,
    } = value;
    w::Observation {
        schema_version: *schema_version,
        view: s_player_view(view),
        decision: decision.as_ref().map(w_domain),
        pending: pending.as_ref().map(w_pending_spell),
        stack: stack.iter().map(w_stack_spell).collect(),
        combat: combat.iter().map(w_combat_attack).collect(),
        unsupported_families: std::array::from_fn(|i| unsupported_families[i].to_string()),
    }
}
fn w_combat_choices(value: &p::CombatChoices) -> w::CombatChoices {
    let p::CombatChoices {
        attackers,
        blockers,
        selected,
        blocks,
        forbidden_blocks,
        damage,
    } = value;
    w::CombatChoices {
        attackers: attackers.iter().map(w_visible_ref).collect(),
        blockers: blockers.iter().map(w_visible_ref).collect(),
        selected: selected.iter().map(w_visible_ref).collect(),
        blocks: blocks
            .iter()
            .map(|x| (w_visible_ref(&x.0), w_visible_ref(&x.1)))
            .collect(),
        forbidden_blocks: forbidden_blocks
            .iter()
            .map(|(b, a)| (w_visible_ref(b), w_visible_ref(a)))
            .collect(),
        damage: damage.iter().map(w_damage_allocation).collect(),
    }
}
fn w_damage_allocation(value: &p::DamageAllocation) -> w::DamageAllocation {
    let p::DamageAllocation {
        attacker,
        power,
        blockers,
        trample_lethal,
        amounts,
    } = value;
    w::DamageAllocation {
        attacker: w_visible_ref(attacker),
        power: *power,
        blockers: blockers.iter().map(w_visible_ref).collect(),
        trample_lethal: trample_lethal.clone(),
        amounts: amounts
            .as_ref()
            .map(|x| x.iter().map(|x| (w_visible_ref(&x.0), x.1)).collect()),
    }
}
fn w_combat_attack(value: &p::CombatAttack) -> w::CombatAttack {
    let p::CombatAttack {
        attacker,
        blocked,
        blockers,
    } = value;
    w::CombatAttack {
        attacker: w_visible_ref(attacker),
        blocked: *blocked,
        blockers: blockers.iter().map(w_visible_ref).collect(),
    }
}
fn w_pending_spell(value: &p::PendingSpell) -> w::PendingSpell {
    let p::PendingSpell {
        card,
        targets,
        sources,
        pool,
        remaining,
    } = value;
    w::PendingSpell {
        card: w_visible_ref(card),
        targets: targets
            .iter()
            .map(|x| x.as_ref().map(w_visible_ref))
            .collect(),
        sources: sources
            .iter()
            .map(|x| x.as_ref().map(w_visible_ref))
            .collect(),
        pool: *pool,
        remaining: remaining.as_ref().map(w_mana_cost),
    }
}
fn w_stack_spell(value: &p::StackSpell) -> w::StackSpell {
    let p::StackSpell {
        row,
        targets,
        ability,
    } = value;
    w::StackSpell {
        row: *row,
        ability: *ability,
        targets: targets
            .iter()
            .map(|x| x.as_ref().map(w_visible_ref))
            .collect(),
    }
}
fn w_mana_cost(value: &mana::ManaCost) -> w::ManaCost {
    let mana::ManaCost { colored, generic } = value;
    w::ManaCost {
        colored: *colored,
        generic: *generic,
    }
}
fn w_choice(value: &v2::Choice) -> w::Choice {
    let v2::Choice {
        submission,
        logical_action,
        micro_choice,
        status,
        policy,
    } = value;
    w::Choice {
        submission: w_submission(submission),
        logical_action: *logical_action,
        micro_choice: *micro_choice,
        status: w_action_status(status),
        policy: w_policy_info(policy),
    }
}
fn w_decision(value: &v2::Decision) -> w::Decision {
    let v2::Decision {
        episode,
        index,
        seat_index,
        actor,
        observation,
        choice,
        reward,
        next_actor,
        terminated,
        truncated,
    } = value;
    w::Decision {
        episode: s_episode_key(episode),
        index: *index,
        seat_index: *seat_index,
        actor: *actor,
        observation: w_observation(observation),
        choice: w_choice(choice),
        reward: *reward,
        next_actor: *next_actor,
        terminated: *terminated,
        truncated: *truncated,
    }
}
fn w_footer(value: &v2::Footer) -> w::Footer {
    let v2::Footer {
        end,
        complete,
        returns,
        boundary_reward,
        decisions,
        logical_actions,
        cancelled_actions,
        final_observations,
    } = value;
    w::Footer {
        end: s_end(end),
        complete: *complete,
        returns: *returns,
        boundary_reward: *boundary_reward,
        decisions: *decisions,
        logical_actions: *logical_actions,
        cancelled_actions: *cancelled_actions,
        final_observations: std::array::from_fn(|i| w_observation(&final_observations[i])),
    }
}
fn w_policy_info(value: &t::PolicyInfo) -> w::PolicyInfo {
    let t::PolicyInfo {
        checkpoint,
        log_probability,
        value,
        recurrent_state,
        exploration,
    } = value;
    w::PolicyInfo {
        checkpoint: checkpoint.as_ref().map(|x| x.to_string()),
        log_probability: *log_probability,
        value: *value,
        recurrent_state: recurrent_state.as_ref().map(|x| x.to_string()),
        exploration: exploration.as_ref().map(|x| x.to_string()),
    }
}
fn w_domain(value: &p::Decision) -> w::Domain {
    let p::Decision {
        revision,
        generation,
        actor,
        kind,
        count,
        candidates,
        legal_mask,
        factored,
    } = value;
    w::Domain {
        revision: *revision,
        generation: *generation,
        actor: *actor,
        kind: kind.to_string(),
        count: *count,
        candidates: candidates.iter().map(w_command).collect(),
        legal_mask: legal_mask.clone(),
        factored: factored.as_ref().map(w_combat_choices),
    }
}
fn s_limit(value: &t::Limit) -> s::Limit {
    match value {
        t::Limit::Decisions => s::Limit::Decisions,
        t::Limit::Turns => s::Limit::Turns,
        t::Limit::WallTime => s::Limit::WallTime,
    }
}
fn s_reward_convention(value: &t::RewardConvention) -> s::RewardConvention {
    match value {
        t::RewardConvention::SparseZeroSumTerminal => s::RewardConvention::SparseZeroSumTerminal,
    }
}
fn s_discount_convention(value: &t::DiscountConvention) -> s::DiscountConvention {
    match value {
        t::DiscountConvention::UndiscountedEpisodic => s::DiscountConvention::UndiscountedEpisodic,
    }
}
fn s_capture_selection(value: &t::CaptureSelection) -> s::CaptureSelection {
    match value {
        t::CaptureSelection::AllDecisions => s::CaptureSelection::AllDecisions,
    }
}
fn w_visible_zone(value: &p::VisibleZone) -> w::VisibleZone {
    match value {
        p::VisibleZone::Hand => w::VisibleZone::Hand,
        p::VisibleZone::Battlefield => w::VisibleZone::Battlefield,
    }
}
fn w_action_status(value: &v2::ActionStatus) -> w::ActionStatus {
    match value {
        v2::ActionStatus::Continuing => w::ActionStatus::Continuing,
        v2::ActionStatus::Committed => w::ActionStatus::Committed,
        v2::ActionStatus::Cancelled => w::ActionStatus::Cancelled,
    }
}
pub(crate) fn s_end(value: &t::End) -> s::End {
    match value {
        t::End::Completed => s::End::Completed,
        t::End::Truncated(limit) => s::End::Truncated(s_limit(limit)),
        t::End::Failed(reason) => s::End::Failed(reason.clone()),
    }
}
fn w_command(value: &p::Choice) -> w::Command {
    match value {
        p::Choice::Keep => w::Command::Keep,
        p::Choice::Mulligan => w::Command::Mulligan,
        p::Choice::Bottom { card } => w::Command::Bottom {
            card: w_visible_ref(card),
        },
        p::Choice::Pass => w::Command::Pass,
        p::Choice::PlayLand { card } => w::Command::PlayLand {
            card: w_visible_ref(card),
        },
        p::Choice::Activate { card } => w::Command::Activate {
            card: w_visible_ref(card),
        },
        p::Choice::FinishActivation => w::Command::FinishActivation,
        p::Choice::CancelActivation => w::Command::CancelActivation,
        p::Choice::TapMana { card } => w::Command::TapMana {
            card: w_visible_ref(card),
        },
        p::Choice::Pay { color } => w::Command::Pay { color: *color },
        p::Choice::FinishPayment => w::Command::FinishPayment,
        p::Choice::CancelPayment => w::Command::CancelPayment,
        p::Choice::Cast { card } => w::Command::Cast {
            card: w_visible_ref(card),
        },
        p::Choice::Target { card } => w::Command::Target {
            card: w_visible_ref(card),
        },
        p::Choice::FinishTargets => w::Command::FinishTargets,
        p::Choice::CancelTargets => w::Command::CancelTargets,
        p::Choice::SelectAttackers { cards } => w::Command::SelectAttackers {
            cards: cards.iter().map(w_visible_ref).collect(),
        },
        p::Choice::SelectBlockers { blocks } => w::Command::SelectBlockers {
            blocks: blocks
                .iter()
                .map(|x| (w_visible_ref(&x.0), w_visible_ref(&x.1)))
                .collect(),
        },
        p::Choice::AssignDamage { attacker, amounts } => w::Command::AssignDamage {
            attacker: w_visible_ref(attacker),
            amounts: amounts.iter().map(|x| (w_visible_ref(&x.0), x.1)).collect(),
        },
        p::Choice::FinishCombat => w::Command::FinishCombat,
        p::Choice::Discard { card } => w::Command::Discard {
            card: w_visible_ref(card),
        },
        p::Choice::Spell => w::Command::Spell,
        p::Choice::Combat => w::Command::Combat,
    }
}
pub(crate) fn s_episode(value: &t::Episode) -> s::Episode {
    s::Episode {
        reward_convention: s_reward_convention(value.reward_convention()),
        discount_convention: s_discount_convention(value.discount_convention()),
        capture_selection: s_capture_selection(value.capture_selection()),
        header: s_header(value.header()),
        decisions: value.decisions().iter().map(s_decision).collect(),
        footer: value.footer().map(s_footer),
    }
}
pub(crate) fn w_episode(value: &v2::Episode) -> w::Episode {
    w::Episode {
        reward_convention: s_reward_convention(value.reward_convention()),
        discount_convention: s_discount_convention(value.discount_convention()),
        capture_selection: s_capture_selection(value.capture_selection()),
        header: s_header(value.header()),
        decisions: value.decisions().iter().map(w_decision).collect(),
        footer: value.footer().map(w_footer),
    }
}

#[cfg(test)]
mod flying_tests {
    use super::*;
    #[test]
    fn typed_forbidden_block_pairs_are_preserved() {
        let r = |row| p::VisibleRef {
            zone: p::VisibleZone::Battlefield,
            row,
        };
        let core = p::CombatChoices {
            attackers: vec![r(0), r(1)],
            blockers: vec![r(2)],
            selected: vec![],
            blocks: vec![],
            forbidden_blocks: vec![(r(2), r(0))],
            damage: vec![],
        };
        let wire = w_combat_choices(&core);
        assert_eq!(
            serde_json::to_value(&wire).unwrap(),
            serde_json::to_value(&core).unwrap()
        );
        assert_eq!(wire.forbidden_blocks.len(), 1);
        assert_eq!(wire.forbidden_blocks[0].0.row, 2);
        assert_eq!(wire.forbidden_blocks[0].1.row, 0);
    }
}
