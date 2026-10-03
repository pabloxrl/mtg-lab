//! Owned work execution shared by opening, turns, spells and combat.
use super::*;

/// Internal work is never a player choice, terminal result, or reward event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    TurnDecision(turns::TurnDecision),
    Terminal(terminal::Outcome),
    InternalYield,
    Decision(OpeningDecision),
    OpeningComplete,
    NotStarted,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub(super) enum Work {
    Turn(turns::TurnWork),
    CombatLife([i64; 2]),
    FinishCombat,
    Modify(targets::Modification),
    GrantHaste(Handle),
    GrantTrample(Handle),
    RemoveAbility(Handle),
    CreateGoblins {
        controller: Seat,
    },
    SpellMove {
        handle: Handle,
        zone: Zone,
        controller: Option<Seat>,
    },
    FinishSpell {
        spell: Handle,
        resolution: Option<targets::Resolution>,
    },
    Priority {
        actor: Seat,
        passed: bool,
        terminal: bool,
    },
    Reset {
        #[serde(with = "snapshot::decks")]
        decks: Box<[[CardId; 40]; 2]>,
        random: [bool; 2],
        seat: usize,
        size: usize,
        position: usize,
        phase: u8,
    },
    Redraw {
        seat: Seat,
        old: Vec<Handle>,
        shuffled: Vec<Handle>,
        current: Vec<Handle>,
        size: usize,
        position: usize,
        phase: u8,
    },
    Bottom {
        seat: Seat,
        cards: Vec<Handle>,
        position: usize,
    },
    Advance,
}
// Exactly one RNG word per unit, including rejection. A rejected word retains
// the same shuffle position so even the rejection loop can yield.
fn shuffle_step<T>(cards: &mut [T], size: &mut usize, rng: &mut EpisodeRng) {
    shuffle_word(cards, size, rng.next_u64());
}
pub(super) fn shuffle_word<T>(cards: &mut [T], size: &mut usize, word: u64) {
    let limit = (1_u128 << 64) / *size as u128 * *size as u128;
    if u128::from(word) < limit {
        cards.swap(*size - 1, (word % *size as u64) as usize);
        *size -= 1;
    }
}
impl Game {
    pub(super) fn finish_work(&mut self) {
        while self.resume(NonZeroUsize::MAX) == Progress::InternalYield {}
    }
    /// Resume owned work exactly once, stopping at the first player boundary.
    /// At a settled boundary repeated resumes are read-only. Inspection during
    /// a yield is privileged and may see partially completed internal work.
    pub fn resume(&mut self, quantum: NonZeroUsize) -> Progress {
        if let Some(result) = self.outcome {
            return Progress::Terminal(result);
        }
        for _ in 0..quantum.get() {
            let Some(mut work) = self.work.pop_front() else {
                break;
            };
            let done = match &mut work {
                Work::Turn(w) => {
                    self.run_turn_work(w);
                    true
                }
                Work::CombatLife(life) => {
                    self.life = *life;
                    true
                }
                Work::FinishCombat => {
                    self.turns.modifications.retain(|m| {
                        self.objects
                            .get(m.handle)
                            .is_ok_and(|o| o.zone == Zone::Battlefield)
                    });
                    self.turns.combat.assignments.clear();
                    true
                }
                Work::GrantTrample(h) => {
                    self.turns.trample.push(*h);
                    true
                }
                Work::GrantHaste(h) => {
                    self.turns.haste.push(*h);
                    true
                }
                Work::RemoveAbility(h) => {
                    self.objects
                        .remove(*h)
                        .expect("preflighted ability removal");
                    true
                }
                Work::Modify(m) => {
                    if let Some(old) = self
                        .turns
                        .modifications
                        .iter_mut()
                        .find(|x| x.handle == m.handle)
                    {
                        *old = *m;
                    } else {
                        self.turns.modifications.push(*m);
                    }
                    true
                }
                Work::CreateGoblins { controller } => {
                    // Fixed two-object effect; all resources reserved before accepting resolution.
                    for _ in 0..2 {
                        let h = self
                            .objects
                            .allocate(
                                CardId::from_key("goblin-token").unwrap(),
                                *controller,
                                Zone::Battlefield,
                            )
                            .expect("preflighted token batch");
                        self.turns.sick.push(h);
                    }
                    true
                }
                Work::SpellMove {
                    handle,
                    zone,
                    controller,
                } => {
                    let moved = self
                        .objects
                        .move_to(*handle, *zone)
                        .expect("preflighted spell move");
                    if self.objects.get(moved).unwrap().card.identity().key == "goblin-token"
                        && *zone != Zone::Battlefield
                    {
                        // CR 704.5d: no priority is exposed between death and cessation.
                        self.objects
                            .remove(moved)
                            .expect("preflighted token cessation");
                    }
                    if let Some(controller) = controller {
                        self.objects.get_mut(moved).expect("permanent").controller = *controller;
                        self.turns.sick.push(moved);
                    }
                    true
                }
                Work::FinishSpell { spell, resolution } => {
                    assert_eq!(self.turns.stack.pop(), Some(*spell));
                    self.turns.abilities.retain(|a| a.object != *spell);
                    self.turns.effects.retain(|(h, _)| h != spell);
                    self.turns.modes.retain(|(h, _)| h != spell);
                    self.turns.modifications.retain(|m| {
                        self.objects
                            .get(m.handle)
                            .is_ok_and(|o| o.zone == Zone::Battlefield)
                    });
                    if resolution.is_some() {
                        self.turns.last_resolution = *resolution;
                    }
                    true
                }
                Work::Priority {
                    actor,
                    passed,
                    terminal,
                } => {
                    self.turns.passed = *passed;
                    if !*terminal || self.settle_terminal(None).is_none() {
                        self.set_turn_decision(*actor, turns::TurnKind::Priority);
                    }
                    true
                }
                Work::Advance => {
                    self.advance_opening();
                    true
                }
                Work::Bottom {
                    seat,
                    cards,
                    position,
                } => {
                    self.objects
                        .move_to(cards[*position], Zone::Library(*seat))
                        .expect("reserved bottom move");
                    *position += 1;
                    if *position == cards.len() {
                        let i = seat_index(*seat);
                        self.needs_bottom[i] = false;
                        if self.mulligans[i] == 7 {
                            self.kept[i] = true;
                        }
                        true
                    } else {
                        false
                    }
                }
                Work::Reset {
                    decks,
                    random,
                    seat,
                    size,
                    position,
                    phase,
                } => {
                    let actor = [Seat::P0, Seat::P1][*seat];
                    match *phase {
                        0 => {
                            if random[*seat] && *size > 1 {
                                shuffle_step(&mut decks[*seat], size, self.rng.as_mut().unwrap());
                            } else if *seat == 0 {
                                *seat = 1;
                                *size = 40;
                            } else {
                                *seat = 0;
                                *phase = 1;
                            }
                            false
                        }
                        1 => {
                            self.objects
                                .allocate(decks[*seat][*position], actor, Zone::Library(actor))
                                .expect("reserved reset allocation");
                            *position += 1;
                            if *position == 40 {
                                *position = 0;
                                *phase = 2;
                            }
                            false
                        }
                        _ => {
                            self.draw_internal(actor);
                            *position += 1;
                            if *position == 7 {
                                if *seat == 0 {
                                    *seat = 1;
                                    *position = 0;
                                    *phase = 1;
                                    false
                                } else {
                                    self.set_decision(self.starting, OpeningKind::KeepOrMulligan);
                                    true
                                }
                            } else {
                                false
                            }
                        }
                    }
                }
                Work::Redraw {
                    seat,
                    old,
                    shuffled,
                    current,
                    size,
                    position,
                    phase,
                } => {
                    match *phase {
                        0 => {
                            if *size > 1 {
                                shuffle_step(shuffled, size, self.rng.as_mut().unwrap());
                            } else {
                                *phase = 1;
                            }
                            false
                        }
                        1 => {
                            current.push(
                                self.objects
                                    .move_to(old[*position], Zone::Library(*seat))
                                    .expect("reserved redraw move"),
                            );
                            *position += 1;
                            if *position == 40 {
                                *phase = 2;
                                *position = 0;
                            }
                            false
                        }
                        2 => {
                            // Fixed 40-card permutation; no unbounded rules work.
                            let updated: Vec<_> = shuffled
                                .iter()
                                .map(|h| current[old.iter().position(|o| o == h).unwrap()])
                                .collect();
                            self.objects.reorder(Zone::Library(*seat), &updated);
                            *phase = 3;
                            false
                        }
                        _ => {
                            self.draw_internal(*seat);
                            *position += 1;
                            if *position == 7 {
                                self.mulligans[seat_index(*seat)] += 1;
                                self.needs_bottom[seat_index(*seat)] = true;
                                true
                            } else {
                                false
                            }
                        }
                    }
                }
            };
            if !done {
                self.work.push_front(work);
            }
        }
        if !self.work.is_empty() {
            Progress::InternalYield
        } else if let Some(result) = self.outcome {
            Progress::Terminal(result)
        } else if let Some(d) = self.turn_decision() {
            Progress::TurnDecision(d)
        } else if let Some(d) = self.decision {
            Progress::Decision(d)
        } else if self.rng.is_some() {
            Progress::OpeningComplete
        } else {
            Progress::NotStarted
        }
    }
}
