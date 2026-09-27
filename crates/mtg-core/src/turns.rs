//! Turn progression and priority, including supported creature-stack resolution. Inspection is privileged, as in opening.
use super::*;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Upkeep,
    Draw,
    PrecombatMain,
    BeginningCombat,
    DeclareAttackers,
    DeclareBlockers,
    CombatDamage,
    EndCombat,
    PostcombatMain,
    End,
    Cleanup,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnKind {
    Priority,
    Combat(super::combat::CombatKind),
    Discard { count: usize },
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurnDecision {
    pub id: DecisionId,
    pub actor: Seat,
    pub kind: TurnKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnProgress {
    Decision(TurnDecision),
    Terminal(super::terminal::Outcome),
}
impl TurnDecision {
    pub fn candidate(self, index: usize) -> CandidateId {
        CandidateId {
            decision: self.id,
            index,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnSelection {
    Pass(CandidateId),
    Discard(Vec<CandidateId>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnAction {
    pub decision: DecisionId,
    pub selection: TurnSelection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnError {
    NotReady,
    AlreadyStarted,
    Invalid(ApplyError),
    Draw(DrawError),
    UnsupportedStack,
    EffectOverflow,
    UnsupportedCombat,
    TurnExhausted,
    Storage(StorageError),
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub(super) struct TurnState {
    pub(super) decision: Option<TurnDecision>,
    pub(super) position: Option<(u64, Seat, Step)>,
    pub(super) passed: bool,
    pub(super) mana: [[u32; 6]; 2],
    pub(super) land_used: bool,
    pub(super) payment: Option<super::mana::Payment>,
    pub(super) casting: Option<super::casting::PendingCast>,
    pub(super) stack: Vec<Handle>,
    pub(super) targeting: Option<super::targets::Targeting>,
    pub(super) effects: Vec<(Handle, super::targets::Effect)>,
    pub(super) modifications: Vec<super::targets::Modification>,
    pub(super) last_resolution: Option<super::targets::Resolution>,
    pub(super) sick: Vec<Handle>,
    pub(super) combat: super::combat::CombatState,
}
impl Game {
    pub fn turn_position(&self) -> Option<(u64, Seat, Step)> {
        self.turns.position
    }
    pub fn turn_decision(&self) -> Option<TurnDecision> {
        if !self.work.is_empty() || self.turns.payment.is_some() || self.turns.targeting.is_some() {
            None
        } else {
            self.turns.decision
        }
    }
    pub fn mana(&self) -> [[u32; 6]; 2] {
        self.turns.mana
    }
    pub fn discard_cards(&self) -> Option<Vec<Handle>> {
        let d = self.turn_decision()?;
        matches!(d.kind, TurnKind::Discard { .. })
            .then(|| self.objects.in_zone(Zone::Hand(d.actor)).collect())
    }
    /// Enter the first upkeep after untap. The opening API retains its explicit
    /// OpeningComplete boundary; starting turns is not a player action.
    pub fn start_turns(&mut self) -> Result<TurnDecision, TurnError> {
        if self.outcome.is_some() {
            return Err(TurnError::NotReady);
        }
        if self.turns.position.is_some() {
            return Err(TurnError::AlreadyStarted);
        }
        if self.rng.is_none()
            || self.decision.is_some()
            || !self.work.is_empty()
            || self.kept != [true; 2]
        {
            return Err(TurnError::NotReady);
        }
        if self.objects.in_zone(Zone::Stack).next().is_some() {
            return Err(TurnError::UnsupportedStack);
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(TurnError::Invalid(ApplyError::DecisionExhausted))?;
        self.untap(self.starting);
        self.turns.position = Some((1, self.starting, Step::Upkeep));
        self.generation = generation;
        Ok(self.set_turn_decision(self.starting, TurnKind::Priority))
    }
    /// Scalar execution drains the same owned continuation as bounded execution.
    pub fn apply_turn(
        &mut self,
        actor: Seat,
        action: &TurnAction,
    ) -> Result<TurnProgress, TurnError> {
        let mut progress = self.apply_turn_quantum(actor, action, NonZeroUsize::MAX)?;
        while progress == Progress::InternalYield {
            progress = self.resume(NonZeroUsize::MAX);
        }
        match progress {
            Progress::TurnDecision(d) => Ok(TurnProgress::Decision(d)),
            Progress::Terminal(o) => Ok(TurnProgress::Terminal(o)),
            _ => unreachable!("accepted turn command settles at a turn boundary"),
        }
    }
    /// Validate the complete command before mutation. Stack settlement and first
    /// priority passes consume bounded work; empty-stack turn/cleanup progression
    /// remains scalar pending its separately owned continuation.
    pub fn apply_turn_quantum(
        &mut self,
        actor: Seat,
        action: &TurnAction,
        quantum: NonZeroUsize,
    ) -> Result<Progress, TurnError> {
        self.apply_turn_work(actor, action)?;
        Ok(self.resume(quantum))
    }
    fn apply_turn_work(&mut self, actor: Seat, action: &TurnAction) -> Result<(), TurnError> {
        let d = self.turn_decision().ok_or(TurnError::NotReady)?;
        let invalid = TurnError::Invalid;
        if actor != d.actor {
            return Err(invalid(ApplyError::WrongActor));
        }
        if action.decision != d.id {
            return Err(invalid(ApplyError::StaleDecision));
        }
        let mut discard = Vec::new();
        match (&action.selection, d.kind) {
            (TurnSelection::Pass(c), TurnKind::Priority) => {
                validate_candidate(*c, d.id, 1).map_err(invalid)?
            }
            (TurnSelection::Discard(cs), TurnKind::Discard { count }) => {
                if cs.len() != count {
                    return Err(invalid(ApplyError::WrongCardinality));
                }
                let cards = self.discard_cards().expect("discard decision");
                for (i, c) in cs.iter().enumerate() {
                    validate_candidate(*c, d.id, cards.len()).map_err(invalid)?;
                    if cs[..i].contains(c) {
                        return Err(invalid(ApplyError::DuplicateCandidate));
                    }
                    discard.push(cards[c.index]);
                }
            }
            _ => return Err(invalid(ApplyError::WrongKind)),
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(invalid(ApplyError::DecisionExhausted))?;
        if self.objects.in_zone(Zone::Stack).next().is_some() {
            return self.queue_stack_pass(actor, generation);
        }
        let (turn, active, step) = self.turns.position.expect("turn decision position");
        if d.kind == TurnKind::Priority && !self.turns.passed {
            self.work
                .try_reserve(1)
                .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
            self.generation = generation;
            self.turns.decision = None;
            self.work.push_back(Work::Priority {
                actor: opponent(actor),
                passed: true,
                terminal: false,
            });
            return Ok(());
        }
        let mut next_turn = turn;
        let mut next_active = active;
        let next_step = match step {
            Step::Upkeep if turn == 1 => Step::PrecombatMain, // CR 103.8a: whole step skipped.
            Step::Upkeep => Step::Draw,
            Step::Draw => Step::PrecombatMain,
            Step::PrecombatMain => Step::BeginningCombat,
            Step::BeginningCombat => {
                if !self.supported_combat() {
                    return Err(TurnError::UnsupportedCombat);
                }
                Step::DeclareAttackers
            }
            Step::DeclareAttackers if self.turns.combat.attacks.is_empty() => Step::EndCombat,
            Step::DeclareAttackers => Step::DeclareBlockers,
            Step::DeclareBlockers => Step::CombatDamage,
            Step::CombatDamage => Step::EndCombat,
            Step::EndCombat => Step::PostcombatMain,
            Step::PostcombatMain => Step::End,
            Step::End if self.objects.in_zone(Zone::Hand(active)).count() > 7 => Step::Cleanup,
            Step::End | Step::Cleanup => {
                next_turn = turn.checked_add(1).ok_or(TurnError::TurnExhausted)?;
                next_active = opponent(active);
                Step::Upkeep
            }
        };
        // Storage failures remain errors without advancement. An attempted empty
        // draw is a rules result, at the draw step after boundary mana empties.
        if next_step == Step::Draw {
            match self.draw_top(active) {
                Ok(_) => {}
                Err(DrawError::EmptyLibrary) => {
                    self.turns.position = Some((next_turn, next_active, next_step));
                    self.turns.mana = [[0; 6]; 2];
                    self.turns.passed = false;
                    self.generation = generation;
                    return Ok(());
                }
                Err(e) => return Err(TurnError::Draw(e)),
            }
        }
        if !discard.is_empty() {
            self.objects
                .prepare_moves(&discard, Zone::Graveyard(active))
                .map_err(TurnError::Storage)?;
            for h in discard {
                self.objects
                    .move_to(h, Zone::Graveyard(active))
                    .expect("preflighted cleanup discard");
            }
        }
        if next_step == Step::Upkeep {
            // CR 514.2: after cleanup discard, expire boosts and remove damage
            // together, with no intermediate lethal-damage check.
            self.cleanup_effects();
            self.turns.land_used = false;
            self.untap(next_active);
        }
        self.turns.position = Some((next_turn, next_active, next_step));
        self.turns.passed = false;
        self.turns.mana = [[0; 6]; 2]; // CR 106.4 / 500.4, both players, every boundary.
        self.generation = generation;
        if next_step == Step::PostcombatMain {
            self.turns.combat = super::combat::CombatState::default();
        }
        let kind = if next_step == Step::DeclareAttackers && self.has_combat_creature(active) {
            TurnKind::Combat(super::combat::CombatKind::Attackers)
        } else if next_step == Step::DeclareBlockers {
            TurnKind::Combat(super::combat::CombatKind::Blockers)
        } else if next_step == Step::CombatDamage {
            TurnKind::Combat(super::combat::CombatKind::Damage)
        } else if next_step == Step::Cleanup {
            TurnKind::Discard {
                count: self.objects.in_zone(Zone::Hand(active)).count() - 7,
            }
        } else {
            TurnKind::Priority
        };
        let actor = if next_step == Step::DeclareBlockers {
            opponent(active)
        } else {
            next_active
        };
        self.set_turn_decision(actor, kind);
        Ok(())
    }
    pub(super) fn set_turn_decision(&mut self, actor: Seat, kind: TurnKind) -> TurnDecision {
        let d = TurnDecision {
            id: DecisionId {
                scope: self.objects.scope(),
                generation: self.generation,
            },
            actor,
            kind,
        };
        self.turns.decision = Some(d);
        d
    }
    fn untap(&mut self, active: Seat) {
        self.turns.sick.retain(|h| {
            self.objects
                .get(*h)
                .is_ok_and(|o| o.zone == Zone::Battlefield && o.controller != active)
        });
        let handles: Vec<_> = self.objects.in_zone(Zone::Battlefield).collect();
        for h in handles {
            let o = self.objects.get_mut(h).expect("live permanent");
            if o.controller == active {
                o.tapped = false;
            }
        }
    }
}

pub(super) fn opponent(seat: Seat) -> Seat {
    match seat {
        Seat::P0 => Seat::P1,
        Seat::P1 => Seat::P0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ready() -> Game {
        let mut g = Game::new().unwrap();
        g.reset(&Config::default(), 42, 9).unwrap();
        while let Some(d) = g.decision() {
            g.apply(
                d.actor,
                &OpeningAction {
                    decision: d.id,
                    selection: Selection::Choose(d.candidate(0)),
                },
            )
            .unwrap();
        }
        g
    }
    fn pass(g: &mut Game) {
        let d = g.turn_decision().unwrap();
        g.apply_turn(
            d.actor,
            &TurnAction {
                decision: d.id,
                selection: TurnSelection::Pass(d.candidate(0)),
            },
        )
        .unwrap();
    }
    #[test]
    fn turns_untap_controller_and_mana_every_boundary() {
        // Synthetic battlefield, not proof of land play. CR 502.3 uses controller.
        let mut g = ready();
        let forest = CardId::from_key("forest").unwrap();
        let a = g
            .objects
            .allocate(forest, Seat::P1, Zone::Battlefield)
            .unwrap();
        let b = g
            .objects
            .allocate(forest, Seat::P0, Zone::Battlefield)
            .unwrap();
        {
            let o = g.objects.get_mut(a).unwrap();
            o.controller = Seat::P0;
            o.tapped = true;
        }
        {
            let o = g.objects.get_mut(b).unwrap();
            o.controller = Seat::P1;
            o.tapped = true;
        }
        g.start_turns().unwrap();
        assert!(!g.objects.get(a).unwrap().tapped);
        assert!(g.objects.get(b).unwrap().tapped);
        // CR 106.4: all colors, both seats; passing once is NOT a boundary.
        for step in [
            Step::Upkeep,
            Step::PrecombatMain,
            Step::BeginningCombat,
            Step::DeclareAttackers,
            Step::EndCombat,
            Step::PostcombatMain,
            Step::End,
        ] {
            assert_eq!(g.turn_position(), Some((1, Seat::P0, step)));
            g.turns.mana = [[1, 2, 3, 4, 5, 6], [6, 5, 4, 3, 2, 1]];
            let mana = g.mana();
            pass(&mut g);
            assert_eq!(g.mana(), mana);
            pass(&mut g);
            assert_eq!(g.mana(), [[0; 6]; 2]);
        }
        assert_eq!(g.turn_position(), Some((2, Seat::P1, Step::Upkeep)));
        assert!(!g.objects.get(b).unwrap().tapped);
        pass(&mut g);
        pass(&mut g);
        assert_eq!(g.turn_position(), Some((2, Seat::P1, Step::Draw)));
        assert_eq!(g.objects.in_zone(Zone::Hand(Seat::P1)).count(), 8);
        g.turns.mana = [[1; 6]; 2];
        pass(&mut g);
        pass(&mut g);
        assert_eq!(g.mana(), [[0; 6]; 2]);
        assert_eq!(g.objects.in_zone(Zone::Hand(Seat::P1)).count(), 8);
    }
    #[test]
    fn turns_exhaustion_and_unsupported_stack_are_unchanged() {
        let mut g = ready();
        g.start_turns().unwrap();
        let d = g.turn_decision().unwrap();
        let a = TurnAction {
            decision: d.id,
            selection: TurnSelection::Pass(d.candidate(0)),
        };
        g.generation = u64::MAX;
        let before = format!("{g:?}");
        assert_eq!(
            g.apply_turn(d.actor, &a),
            Err(TurnError::Invalid(ApplyError::DecisionExhausted))
        );
        assert_eq!(format!("{g:?}"), before);
        g.generation = d.id.generation;
        g.objects
            .allocate(CardId::from_key("bear-cub").unwrap(), Seat::P0, Zone::Stack)
            .unwrap();
        let before = format!("{g:?}");
        assert_eq!(g.apply_turn(d.actor, &a), Err(TurnError::UnsupportedStack));
        assert_eq!(format!("{g:?}"), before);
    }
    #[test]
    fn turns_empty_library_terminal_and_unsupported_combat_preserve_boundary() {
        let mut g = ready();
        g.start_turns().unwrap();
        // CR 704.5b supersedes the pre-GH-72 unsupported-draw placeholder.
        // Synthetic second-turn upkeep: lose only upon the draw attempt.
        g.turns.position = Some((2, Seat::P0, Step::Upkeep));
        let cards: Vec<_> = g.objects.in_zone(Zone::Library(Seat::P0)).collect();
        for h in cards {
            g.objects.remove(h).unwrap();
        }
        pass(&mut g);
        let d = g.turn_decision().unwrap();
        assert_eq!(
            g.apply_turn(
                d.actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Pass(d.candidate(0))
                }
            ),
            Ok(TurnProgress::Terminal(super::terminal::Outcome {
                winner: Some(Seat::P1),
                losses: [Some(super::terminal::LossReason::EmptyDraw), None]
            }))
        );
        assert!(g.turn_decision().is_none());
        assert_eq!(g.turn_position(), Some((2, Seat::P0, Step::Draw)));
        assert_eq!(g.mana(), [[0; 6]; 2]);
        let mut g = ready();
        g.start_turns().unwrap();
        for _ in 0..4 {
            pass(&mut g);
        }
        g.objects
            .allocate(
                CardId::from_key("magnigoth-sentry").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        pass(&mut g);
        let d = g.turn_decision().unwrap();
        let before = format!("{g:?}");
        assert_eq!(
            g.apply_turn(
                d.actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Pass(d.candidate(0))
                }
            ),
            Err(TurnError::UnsupportedCombat)
        );
        assert_eq!(format!("{g:?}"), before);
    }
    #[test]
    fn turns_cleanup_multiple_cards_foreign_candidates_and_turn_exhaustion() {
        let mut g = ready();
        g.start_turns().unwrap();
        g.draw_top(Seat::P0).unwrap();
        g.draw_top(Seat::P0).unwrap();
        for _ in 0..14 {
            pass(&mut g);
        }
        assert_eq!(g.turn_position(), Some((1, Seat::P0, Step::Cleanup)));
        let d = g.turn_decision().unwrap();
        assert_eq!(d.kind, TurnKind::Discard { count: 2 });
        let mut foreign = ready();
        let f = foreign.start_turns().unwrap();
        for (cs, error) in [
            (
                vec![d.candidate(0), d.candidate(0)],
                ApplyError::DuplicateCandidate,
            ),
            (
                vec![d.candidate(0), f.candidate(1)],
                ApplyError::StaleCandidate,
            ),
        ] {
            let before = format!("{g:?}");
            assert_eq!(
                g.apply_turn(
                    d.actor,
                    &TurnAction {
                        decision: d.id,
                        selection: TurnSelection::Discard(cs)
                    }
                ),
                Err(TurnError::Invalid(error))
            );
            assert_eq!(format!("{g:?}"), before);
        }
        g.turns.position = Some((u64::MAX, Seat::P0, Step::Cleanup));
        let action = TurnAction {
            decision: d.id,
            selection: TurnSelection::Discard(vec![d.candidate(8), d.candidate(0)]),
        };
        let before = format!("{g:?}");
        assert_eq!(
            g.apply_turn(d.actor, &action),
            Err(TurnError::TurnExhausted)
        );
        assert_eq!(format!("{g:?}"), before);
        g.turns.position = Some((1, Seat::P0, Step::Cleanup));
        g.turns.mana = [[3; 6]; 2];
        let cards = g.discard_cards().unwrap();
        let expected = [cards[8], cards[0]].map(|h| g.objects.get(h).unwrap().card);
        g.apply_turn(d.actor, &action).unwrap();
        assert_eq!(g.mana(), [[0; 6]; 2]);
        let actual: Vec<_> = g
            .objects
            .in_zone(Zone::Graveyard(Seat::P0))
            .map(|h| g.objects.get(h).unwrap().card)
            .collect();
        assert_eq!(actual, expected);
    }
    #[test]
    fn turns_same_neutral_priority_fixture_as_reference_bridges() {
        // This narrow decoder supports ONLY the existing smoke's declared shape.
        // It is not a general scenario executor or normal-reset reachability test.
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../fixtures/scenarios/xmage-priority-pass.json"
        ))
        .unwrap();
        let initial = &fixture["setup"]["state"];
        assert_eq!(initial["turn"], 1);
        assert_eq!(initial["active_player"], 0);
        assert_eq!(initial["priority"], 0);
        assert_eq!(initial["phase"], "beginning");
        assert_eq!(initial["step"], "upkeep");
        assert_eq!(initial["stack"], serde_json::json!([]));
        assert_eq!(initial["effects"], serde_json::json!([]));
        let mut g = ready();
        g.objects.reset().unwrap();
        let mut ids = Vec::new();
        for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
            let player = &initial["players"][i];
            assert_eq!(player["seat"], i);
            g.life[i] = player["life"].as_u64().unwrap().try_into().unwrap();
            for (j, key) in ["W", "U", "B", "R", "G", "C"].into_iter().enumerate() {
                g.turns.mana[i][j] = player["mana"][key].as_u64().unwrap().try_into().unwrap();
            }
            for zone in ["hand", "battlefield", "graveyard", "exile"] {
                assert_eq!(player["zones"][zone], serde_json::json!([]));
            }
            for id in player["zones"]["library"].as_array().unwrap() {
                let o = initial["objects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|o| o["id"] == *id)
                    .unwrap();
                assert_eq!(o["owner"], i);
                assert_eq!(o["controller"], i);
                assert_eq!(o["status"]["tapped"], false);
                let h = g
                    .objects
                    .allocate(
                        CardId::from_key(o["card_id"].as_str().unwrap()).unwrap(),
                        seat,
                        Zone::Library(seat),
                    )
                    .unwrap();
                ids.push((h, id.clone()));
            }
        }
        g.start_turns().unwrap();
        let objects_before = format!("{:?}", g.objects);
        let rng_before = format!("{:?}", g.rng);
        assert_eq!(fixture["script"].as_array().unwrap().len(), 1);
        assert_eq!(fixture["script"][0]["kind"], "pass");
        assert_eq!(fixture["script"][0]["actor"], 0);
        pass(&mut g);
        let (turn, active, step) = g.turn_position().unwrap();
        assert_eq!(step, Step::Upkeep);
        let players:Vec<_>=[Seat::P0,Seat::P1].into_iter().enumerate().map(|(i,seat)|{
            let library:Vec<_>=g.objects.in_zone(Zone::Library(seat)).map(|h|ids.iter().find(|(handle,_)|*handle==h).unwrap().1.clone()).collect();
            let mana=g.mana()[i];serde_json::json!({"seat":i,"life":g.life()[i],"mana":{"W":mana[0],"U":mana[1],"B":mana[2],"R":mana[3],"G":mana[4],"C":mana[5]},"zones":{"library":library,"hand":[],"graveyard":[],"battlefield":[],"exile":[]}})
        }).collect();
        // Empty zones are established above and exact entire storage is checked
        // unchanged, not inferred solely from the expected checkpoint.
        assert_eq!(format!("{:?}", g.objects), objects_before);
        assert_eq!(format!("{:?}", g.rng), rng_before);
        let actual = serde_json::json!({"turn":turn,"active_player":seat_index(active),"priority":seat_index(g.turn_decision().unwrap().actor),"phase":"beginning","step":"upkeep","players":players,"stack":[]});
        let mut expected = initial.clone();
        expected["priority"] = serde_json::json!(1);
        expected.as_object_mut().unwrap().remove("objects");
        expected.as_object_mut().unwrap().remove("effects");
        for p in expected["players"].as_array_mut().unwrap() {
            p.as_object_mut().unwrap().remove("land_plays_used");
        }
        assert_eq!(actual, expected); // CR 117.3d, independently derived, no output oracle.
        for a in fixture["checkpoints"][0]["assertions"].as_array().unwrap() {
            assert_eq!(
                actual.pointer(a["path"].as_str().unwrap()).unwrap(),
                &a["expected"]
            );
        }
    }
}
