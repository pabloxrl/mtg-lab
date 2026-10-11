//! Test/measurement adapter over the real scalar owner, not a product command.
//! Privileged traces are checked before clocks; rules never own clocks.
use crate::{native::PolicyTiming, script, simulate};
use mtg_core::{
    episode::{Driver, Progress, Status},
    game::{
        Game, actions,
        turns::{TurnAction, TurnSelection},
    },
    objects::Seat,
    trajectory::{EpisodeKey, Header, Limits, Versions},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{cell::Cell, collections::BTreeMap, num::NonZeroUsize};

// Diagnostic-only, current-thread allocation traffic. Delegation preserves the
// System allocator contract, including alignment and realloc failure semantics.
struct Allocator;
thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static ALLOCS: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
    static ALLOCATION_OVERFLOW: Cell<bool> = const { Cell::new(false) };
}
fn allocated(size: usize) {
    let _ = ENABLED.try_with(|enabled| {
        if enabled.get() {
            let _ = ALLOCS.try_with(|counts| {
                let (n, bytes) = counts.get();
                if n.checked_add(1).is_none() || bytes.checked_add(size as u64).is_none() {
                    let _ = ALLOCATION_OVERFLOW.try_with(|flag| flag.set(true));
                }
                counts.set((n.saturating_add(1), bytes.saturating_add(size as u64)));
            });
        }
    });
}
unsafe impl std::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let p = unsafe { std::alloc::System.alloc(layout) };
        if !p.is_null() {
            allocated(layout.size());
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: std::alloc::Layout) -> *mut u8 {
        let p = unsafe { std::alloc::System.alloc_zeroed(layout) };
        if !p.is_null() {
            allocated(layout.size());
        }
        p
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, size: usize) -> *mut u8 {
        let p = unsafe { std::alloc::System.realloc(ptr, layout, size) };
        if !p.is_null() {
            allocated(size);
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe { std::alloc::System.dealloc(ptr, layout) };
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
struct Counting;
impl Counting {
    fn begin() -> Self {
        ENABLED.with(|v| assert!(!v.replace(true)));
        Self
    }
}
impl Drop for Counting {
    fn drop(&mut self) {
        ENABLED.with(|v| v.set(false));
    }
}
fn counts() -> (u64, u64) {
    ALLOCS.with(Cell::get)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    life: [i64; 2],
    hand_counts: Option<[usize; 2]>,
    library_counts: Option<[usize; 2]>,
    creatures: Option<Vec<(String, [u32; 3])>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Trace {
    schema_version: u32,
    privacy: String,
    validated: bool,
    oracle: String,
    engine: String,
    config: simulate::Config,
    checkpoints: Vec<Expected>,
    #[serde(rename = "final")]
    final_state: Expected,
    #[serde(default)]
    capture: bool,
}
#[derive(Default, Serialize)]
struct Cost {
    ns: u128,
    calls: u64,
    allocations: u64,
    allocated_bytes: u64,
}
struct Meter<'a> {
    clock: Option<&'a dyn Fn() -> u128>,
    last: Cell<Option<u128>>,
    costs: BTreeMap<&'static str, Cost>,
}
impl<'a> Meter<'a> {
    fn new(clock: Option<&'a dyn Fn() -> u128>) -> Self {
        Self {
            clock,
            last: Cell::new(None),
            costs: [
                "reset",
                "application",
                "legality_and_view",
                "encoding",
                "finalization",
                "shared",
            ]
            .into_iter()
            .map(|s| (s, Cost::default()))
            .collect(),
        }
    }
    fn now(&self) -> Result<u128, String> {
        let n = self.clock.map_or(0, |f| f());
        if self.last.replace(Some(n)).is_some_and(|last| n < last) {
            return Err("nonmonotonic profile clock".into());
        }
        Ok(n)
    }
    fn measure<T>(
        &mut self,
        label: &'static str,
        f: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let before = counts();
        let start = self.now()?;
        let result = f();
        let end = self.now()?;
        let after = counts();
        let cost = self.costs.get_mut(label).ok_or("unknown cost category")?;
        cost.ns = cost
            .ns
            .checked_add(end - start)
            .ok_or("duration overflow")?;
        if cost.ns > u64::MAX as u128 {
            return Err("duration exceeds artifact capacity".into());
        }
        cost.calls += 1;
        cost.allocations += after.0 - before.0;
        cost.allocated_bytes += after.1 - before.1;
        result
    }
}
fn error(e: impl std::fmt::Debug) -> String {
    format!("native execution rejected: {e:?}")
}
fn settle(d: &mut Driver) -> Result<(), String> {
    if d.status().is_some() {
        return Ok(());
    }
    for _ in 0..100_000 {
        if d.advance(NonZeroUsize::new(64).unwrap()).map_err(error)? != Progress::InternalYield {
            return Ok(());
        }
    }
    Err("unfinished bounded work".into())
}
fn check(d: &Driver, expected: &Expected, index: usize) -> Result<(), String> {
    let o = d.observe(Seat::P0).map_err(error)?;
    let v = &o.view;
    let mut creatures: Vec<_> = v
        .public_zones
        .iter()
        .filter(|z| z.zone == "battlefield")
        .flat_map(|z| &z.cards)
        .filter_map(|c| c.creature.map(|state| (c.card.to_string(), state)))
        .collect();
    creatures.sort();
    let creature_match = expected.creatures.as_ref().is_none_or(|wanted| {
        let mut wanted = wanted.clone();
        wanted.sort();
        wanted == creatures
    });
    if v.life != expected.life
        || expected.hand_counts.is_some_and(|e| e != v.hand_counts)
        || expected
            .library_counts
            .is_some_and(|e| e != v.library_counts)
        || !creature_match
    {
        return Err(format!("checkpoint {index} mismatch"));
    }
    Ok(())
}
fn header(t: &Trace) -> Header {
    let hash = simulate::hash(&serde_json::to_vec(&t.config.game).unwrap());
    Header {
        id: EpisodeKey {
            run: "fce8c3c3-1d41-4a21-87ab-ef4a9b605aa3".into(),
            ordinal: t.config.first_episode,
        },
        versions: Versions {
            schema: 2,
            engine: t.engine.clone(),
            rules: simulate::hash(include_bytes!("../../../data/rules/cr-2026-09-25.json")),
            cards: simulate::hash(include_bytes!(
                "../../../data/cards/foundations_micro_v1.json"
            )),
            action: "semantic-v1".into(),
            observation: 1,
        },
        deck_hashes: [0, 1]
            .map(|i| simulate::hash(&serde_json::to_vec(&t.config.game.seats[i]).unwrap())),
        config_hash: hash,
        policies: t.config.policies.clone(),
        starting_seat: t.config.game.starting_seat,
        limits: Limits::default(),
        restricted_replay: None,
    }
}
struct Execution {
    report: Value,
    witnesses: Vec<(Vec<u8>, actions::Record)>,
}
fn execute(t: &Trace, clock: Option<&dyn Fn() -> u128>) -> Result<Execution, String> {
    let mut meter = Meter::new(clock);
    let counting = clock.map(|_| Counting::begin());
    let initial_allocs = counts();
    let start = meter.now()?;
    let mut driver = Driver::instrumented(1024, t.config.native.as_ref().unwrap().instrumentation)
        .map_err(error)?;
    if let Some(config) = t.config.native.as_ref().unwrap().trace {
        driver.set_trace_config(config).map_err(error)?;
    }
    let h = header(t);
    meter.measure("reset", || {
        if t.capture {
            driver
                .reset_captured(
                    &t.config.game,
                    t.config.master_seed,
                    t.config.first_episode,
                    NonZeroUsize::new(64).unwrap(),
                    &h,
                )
                .map_err(error)?;
        } else {
            driver
                .reset(
                    &t.config.game,
                    t.config.master_seed,
                    t.config.first_episode,
                    NonZeroUsize::new(64).unwrap(),
                )
                .map_err(error)?;
        }
        settle(&mut driver)
    })?;
    let script = t.config.script.as_ref().unwrap();
    let encoder = PolicyTiming {
        encode: true,
        ..Default::default()
    };
    let mut observations = Vec::new();
    let mut decision_kinds = BTreeMap::<String, usize>::new();
    let mut legal_candidates = 0;
    let mut witnesses = Vec::new();
    let mut encoded_bytes = 0;
    let mut cursor = 0;
    for (index, entry) in script.records.iter().enumerate() {
        let record: actions::Record = serde_json::from_str(&entry.record).map_err(error)?;
        if clock.is_none() && record.choices == [actions::Choice::Pass {}] {
            witnesses.push((driver.privileged_snapshot(), record.clone()));
        }
        let o = meter.measure("legality_and_view", || {
            driver.observe(entry.seat).map_err(error)
        })?;
        if o.decision.is_none() && record.decision != "concession" {
            return Err("missing real legal observation".into());
        }
        let bytes = meter.measure("encoding", || encoder.encode_observation(&o).map_err(error))?;
        encoded_bytes += bytes.len();
        if let Some(d) = &o.decision {
            *decision_kinds.entry(d.kind.to_string()).or_default() += 1;
            legal_candidates += d.legal_mask.iter().filter(|legal| **legal).count();
        }
        observations.push(o);
        meter.measure("application", || {
            script::submit(script, &mut cursor, t.config.first_episode, &mut driver)
                .map_err(|e| format!("{e} at record {index}"))?;
            settle(&mut driver)
        })?;
        check(&driver, &t.checkpoints[index], index)?;
    }
    check(&driver, &t.final_state, script.records.len())?;
    if cursor != script.records.len() || !matches!(driver.status(), Some(Status::Completed(_))) {
        return Err("unfinished trace".into());
    }
    let (result, replay_bytes) = meter.measure("finalization", || {
        let result = driver.finish().map_err(error)?;
        let replay = result.privileged_replay(67_108_864).map_err(error)?;
        if let Some(bytes) = &replay {
            mtg_core::opening::replay::played::verify(bytes).map_err(error)?;
        }
        Ok((result, replay.map_or(0, |b| b.len())))
    })?;
    let final_observation = result
        .final_observations()
        .ok_or("missing final observations")?;
    let capture_sha256 = result
        .trajectory()
        .map(|episode| serde_json::to_vec(episode).map(|bytes| simulate::hash(&bytes)))
        .transpose()
        .map_err(error)?;
    let mut report = json!({
        "schema_version":1,"workload":"fixed-trace-v1","status":"measured","privacy":"privileged-diagnostic",
        "instrumentation":t.config.native.as_ref().unwrap().instrumentation,"capture":t.capture,
        "action_version":actions::ACTION_VERSION,"observation_version":1,"work_quantum":64,"oracle":t.oracle,
        "validated_checkpoints":t.checkpoints.len(),"consumed":cursor,"observations":observations.len(),
        "decision_kinds":decision_kinds,"legal_candidates":legal_candidates,
        "observations_sha256":simulate::hash(&serde_json::to_vec(&observations).map_err(error)?),
        "history_sha256":simulate::hash(&serde_json::to_vec(result.privileged_history()).map_err(error)?),
        "encoding":{"bytes":encoded_bytes,"format":"existing PolicyTiming.encode authorized Observation JSON"},
        "capture_sha256":capture_sha256,
        "policy_ns":0,"captured_decisions":result.trajectory().map_or(0, |e|e.decisions().len()),
        "replay_bytes":replay_bytes,"final":final_observation[0].view,
        "application_scope":"Driver semantic decode/validate/apply and settlement, including scratch validation and internal legality; shared work cannot be separated into pure rules cost",
        "allocation_scope":"current-thread successful allocation/reallocation requested count/bytes, not live or resident memory",
        "cpu_profile":null,"limitations":["No hardware counters or sampled call stacks; wall-clock isolated dispatch boundary is provided separately","First work item executes during pass submission; excluded from isolated dispatch samples","Diagnostic report must not be used as a player observation"]
    });
    let elapsed = meter.now()?.checked_sub(start).ok_or("clock subtraction")?;
    if elapsed > u64::MAX as u128 {
        return Err("elapsed duration exceeds artifact capacity".into());
    }
    let after = counts();
    drop(counting);
    let sum_ns = meter
        .costs
        .values()
        .try_fold(0_u128, |a, c| a.checked_add(c.ns))
        .ok_or("sum overflow")?;
    let total_n: u64 = meter.costs.values().map(|c| c.allocations).sum();
    let total_bytes: u64 = meter.costs.values().map(|c| c.allocated_bytes).sum();
    let shared = meter.costs.get_mut("shared").unwrap();
    shared.ns = elapsed
        .checked_sub(sum_ns)
        .ok_or("overlapping time categories")?;
    shared.calls = 1;
    shared.allocations = (after.0 - initial_allocs.0)
        .checked_sub(total_n)
        .ok_or("overlapping allocation categories")?;
    shared.allocated_bytes = (after.1 - initial_allocs.1)
        .checked_sub(total_bytes)
        .ok_or("overlapping allocation bytes")?;
    if ALLOCATION_OVERFLOW.with(Cell::get) {
        return Err("allocation counter overflow".into());
    }
    report["elapsed_ns"] = json!(elapsed);
    report["allocations"] = json!(after.0 - initial_allocs.0);
    report["allocated_bytes"] = json!(after.1 - initial_allocs.1);
    report["costs"] = serde_json::to_value(meter.costs).map_err(error)?;
    Ok(Execution { report, witnesses })
}
fn payload(g: &Game) -> Result<Value, String> {
    let e: Value = serde_json::from_slice(&g.snapshot()).map_err(error)?;
    serde_json::from_str(e["payload"].as_str().ok_or("snapshot payload")?).map_err(error)
}
fn effects(
    witnesses: &[(Vec<u8>, actions::Record)],
    clock: &dyn Fn() -> u128,
) -> Result<Value, String> {
    let mut cost = Cost::default();
    let mut samples = Vec::new();
    for (snapshot, record) in witnesses {
        let mut g = Game::new().map_err(error)?;
        g.restore(snapshot).map_err(error)?;
        // The authoritative decoder checks actor, current decision and legality.
        actions::decode(&g, &serde_json::to_vec(record).map_err(error)?, 1024).map_err(error)?;
        let d = g.turn_decision().ok_or("pass without priority")?;
        let mut progress = g
            .apply_turn_quantum(
                record.actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Pass(d.candidate(0)),
                },
                NonZeroUsize::MIN,
            )
            .map_err(error)?;
        let mut steps = 0;
        while progress == mtg_core::game::Progress::InternalYield {
            steps += 1;
            if steps > 100_000 {
                return Err("unfinished effect work".into());
            }
            let state = payload(&g)?;
            if state["work"][0].get("Modify").is_some() {
                let before = simulate::hash(&g.snapshot());
                let counting = Counting::begin();
                let alloc_before = counts();
                let start = clock();
                progress = std::hint::black_box(g.resume(NonZeroUsize::MIN));
                let elapsed = clock()
                    .checked_sub(start)
                    .ok_or("nonmonotonic effect clock")?;
                let alloc_after = counts();
                drop(counting);
                cost.ns = cost
                    .ns
                    .checked_add(elapsed)
                    .ok_or("effect duration overflow")?;
                if cost.ns > u64::MAX as u128 {
                    return Err("effect duration exceeds artifact capacity".into());
                }
                cost.calls += 1;
                cost.allocations += alloc_after.0 - alloc_before.0;
                cost.allocated_bytes += alloc_after.1 - alloc_before.1;
                samples.push(json!({"before_sha256":before,"after_sha256":simulate::hash(&g.snapshot()),"ns":elapsed}));
            } else {
                progress = g.resume(NonZeroUsize::MIN);
            }
        }
    }
    Ok(
        json!({"scope":"isolated pending Work::Modify via Game::resume(1); separate experiment","dispatches":cost.calls,"cost":cost,"samples":samples}),
    )
}
pub(crate) fn run(input: &[u8], clock: &dyn Fn() -> u128) -> Result<Value, String> {
    if input.len() > 1_048_576 {
        return Err("trace byte bound".into());
    }
    let trace: Trace = serde_json::from_slice(input).map_err(error)?;
    if trace.schema_version != 1
        || trace.privacy != "privileged"
        || !trace.validated
        || trace.oracle.trim().is_empty()
        || trace.engine != mtg_core::game::snapshot::engine()
    {
        return Err("unvalidated trace or incompatible pin".into());
    }
    trace.config.validate()?;
    let script = trace
        .config
        .script
        .as_ref()
        .ok_or("typed semantic trace required")?;
    let native = trace
        .config
        .native
        .as_ref()
        .ok_or("native bounds required")?;
    if trace.config.deadline_ms.is_some()
        || trace.config.capture.is_some()
        || script.records.len() > native.max_records.get()
        || script.records.len() as u64 > trace.config.max_decisions.saturating_add(1)
        || native.work_quantum.get() != 64
        || native.max_work_calls != 100_000
    {
        return Err("unsupported profile bound/persistence; frozen quantum 64 and 100000 work limit required".into());
    }
    if trace.config.episodes != 1
        || script.records.is_empty()
        || script.records.len() != trace.checkpoints.len()
    {
        return Err("missing or extra trace suffix/checkpoint".into());
    }
    // No externally supplied performance clock is reachable before this succeeds.
    let checked = execute(&trace, None)?;
    let mut report = execute(&trace, Some(clock))?.report;
    report["effect_probe"] = effects(&checked.witnesses, clock)?;
    report["trace_sha256"] = json!(simulate::hash(input));
    report["source_sha256"] = json!(env!("MTG_BENCH_SOURCE"));
    report["binary_sha256"] = json!(simulate::hash(
        &std::fs::read(std::env::current_exe().map_err(error)?).map_err(error)?
    ));
    report["toolchain_sha256"] = json!(simulate::hash(env!("MTG_BENCH_RUSTC").as_bytes()));
    report["toolchain"] = json!(env!("MTG_BENCH_RUSTC"));
    report["engine"] = json!(trace.engine);
    report["rules_sha256"] = json!(simulate::hash(include_bytes!(
        "../../../data/rules/cr-2026-09-25.json"
    )));
    report["cards_sha256"] = json!(simulate::hash(include_bytes!(
        "../../../data/cards/foundations_micro_v1.json"
    )));
    report["dependency_sha256"] = json!(simulate::hash(include_bytes!("../../../Cargo.lock")));
    report["build"] = json!({"target":env!("MTG_BENCH_TARGET"),"opt_level":env!("MTG_BENCH_OPT_LEVEL"),"rustflags":env!("MTG_BENCH_RUSTFLAGS")});
    Ok(report)
}
#[test]
fn profile_validated_normal_trace_has_separate_nonempty_costs() {
    let report = run(
        include_bytes!("../../../fixtures/profile/combat.json"),
        &|| 0,
    )
    .expect("independently validated fixed trace must execute before profiling");
    assert_eq!(report["validated_checkpoints"], 434);
    assert!(report["observations"].as_u64().unwrap() > 0);
    assert!(report["encoding"]["bytes"].as_u64().unwrap() > 0);
}

#[test]
fn profile_wrong_checkpoint_fails_before_clock() {
    let mut input: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/profile/combat.json")).unwrap();
    input["checkpoints"][0]["life"][1] = 19.into();
    let result = run(&serde_json::to_vec(&input).unwrap(), &|| {
        panic!("correctness gate must precede timing")
    });
    assert!(result.is_err());
}

#[test]
fn profile_negative_controls_never_read_clock() {
    let original: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/profile/combat.json")).unwrap();
    for mutation in 0..7 {
        let mut bad = original.clone();
        match mutation {
            0 => bad["validated"] = false.into(),
            1 => bad["engine"] = "wrong-pin".into(),
            2 => {
                bad["config"]["script"]["records"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            3 => {
                let row = bad["config"]["script"]["records"][0].clone();
                bad["config"]["script"]["records"]
                    .as_array_mut()
                    .unwrap()
                    .push(row);
            }
            4 => bad["config"]["script"]["records"][0]["record"] = "wrong action".into(),
            5 => {
                bad["config"]["script"]["records"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
                bad["checkpoints"].as_array_mut().unwrap().pop();
            }
            _ => {
                let mut target: Value = serde_json::from_str(
                    bad["config"]["script"]["records"][82]["record"]
                        .as_str()
                        .unwrap(),
                )
                .unwrap();
                target["choices"][0]["card"]["incarnation"] = 99.into();
                bad["config"]["script"]["records"][82]["record"] = target.to_string().into();
            }
        }
        assert!(
            run(&serde_json::to_vec(&bad).unwrap(), &|| panic!(
                "invalid trace reached timing"
            ))
            .is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn profile_effects_and_no_effect_control_use_real_engine() {
    let effects = run(
        include_bytes!("../../../fixtures/profile/effects.json"),
        &|| 0,
    )
    .unwrap();
    assert!(effects["effect_probe"]["dispatches"].as_u64().unwrap() >= 2);
    let control = run(
        include_bytes!("../../../fixtures/profile/control.json"),
        &|| 0,
    )
    .unwrap();
    assert_eq!(control["effect_probe"]["dispatches"], 0);
}

#[test]
fn profile_injected_clock_accounts_exact_disjoint_spans() {
    let reads = std::cell::Cell::new(0_u128);
    let report = run(
        include_bytes!("../../../fixtures/profile/control.json"),
        &|| {
            let n = reads.get();
            reads.set(n + 1);
            n * 7
        },
    )
    .unwrap();
    let costs = report["costs"].as_object().unwrap();
    for (name, cost) in costs {
        if name != "shared" {
            assert_eq!(
                cost["ns"].as_u64().unwrap(),
                cost["calls"].as_u64().unwrap() * 7,
                "{name}"
            );
        }
    }
    assert_eq!(report["policy_ns"], 0);
    // Six records, each with view/encode/apply, plus reset and finalization:
    // 20 disjoint spans × two reads, bracketed by two outer reads.
    assert_eq!(report["elapsed_ns"], 287);
    assert_eq!(report["costs"]["shared"]["ns"], 147);
    assert_eq!(report["costs"]["encoding"]["calls"], 6);
    assert_eq!(report["costs"]["legality_and_view"]["calls"], 6);
    assert_eq!(
        costs
            .values()
            .map(|c| c["ns"].as_u64().unwrap())
            .sum::<u64>(),
        report["elapsed_ns"].as_u64().unwrap()
    );
}

#[test]
fn profile_nonmonotonic_clock_is_an_error() {
    let reads = std::cell::Cell::new(10000_u128);
    assert!(
        run(
            include_bytes!("../../../fixtures/profile/control.json"),
            &|| {
                let n = reads.get();
                reads.set(n - 1);
                n
            }
        )
        .is_err()
    );
}

#[test]
fn profile_export() {
    let input = std::env::var_os("MTG_PROFILE_INPUT")
        .map(|p| std::fs::read(p).unwrap())
        .unwrap_or_else(|| include_bytes!("../../../fixtures/profile/control.json").to_vec());
    let start = std::time::Instant::now();
    let result = run(&input, &|| start.elapsed().as_nanos());
    let report = match result {
        Ok(value) => value,
        Err(error) => serde_json::json!({"error":error}),
    };
    if let Some(path) = std::env::var_os("MTG_PROFILE_OUTPUT") {
        std::fs::write(path, serde_json::to_vec(&report).unwrap()).unwrap();
    } else {
        assert!(report.get("error").is_none(), "{report}");
    }
}
