//! Caller-authorized, in-memory replay artifacts for owned captured episodes.
use super::{EpisodeResult, Status};
use crate::trajectory::EpisodeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Availability {
    Available(String),
    Unavailable(Status),
    NotCaptured,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Denied,
    Unknown,
    Identity,
    Binding,
    Corrupt,
}
// No Debug/Serialize implementation: a registry contains both seats' secrets.
#[derive(Default)]
pub struct Registry {
    artifacts: std::collections::BTreeMap<String, Artifact>,
    used: std::collections::BTreeSet<String>,
}
struct Artifact {
    episode: EpisodeKey,
    binding: Vec<u8>,
    bytes: Vec<u8>,
}
impl Registry {
    /// The local owner supplies an opaque ID, unrelated to seeds or policy data.
    /// IDs may never be rebound within this registry, even after removal.
    /// Registration grants no read authority. The caller must authorize each read.
    pub fn register(
        &mut self,
        id: &str,
        result: &mut EpisodeResult,
    ) -> Result<Availability, Error> {
        if !matches!(result.status, Status::Completed(_)) {
            return Ok(Availability::Unavailable(result.status));
        }
        let Some(trajectory) = result.trajectory.as_ref() else {
            return Ok(Availability::NotCaptured);
        };
        if id.trim().is_empty()
            || self.used.contains(id)
            || trajectory
                .header()
                .restricted_replay
                .as_ref()
                .is_some_and(|x| x != id)
        {
            return Err(Error::Identity);
        }
        if trajectory.header().id.ordinal != result.inputs.ordinal
            || !trajectory
                .footer()
                .is_some_and(|f| f.complete && f.end == crate::trajectory::End::Completed)
        {
            return Err(Error::Binding);
        }
        let choices: Vec<crate::game::actions::Record> = result
            .history
            .iter()
            .map(|b| serde_json::from_slice(b).map_err(|_| Error::Corrupt))
            .collect::<Result<_, _>>()?;
        let inputs = &result.inputs;
        let bytes = crate::opening::replay::played::record(
            &inputs.config,
            inputs.master,
            inputs.ordinal,
            &choices,
        )
        .map_err(|_| Error::Binding)?;
        check(&bytes, result)?;
        let episode = trajectory.header().id.clone();
        // Mutate the owned trajectory only after all validation succeeds.
        result.trajectory.as_mut().unwrap().bind_replay(id);
        self.artifacts.insert(
            id.into(),
            Artifact {
                episode,
                binding: binding(result),
                bytes,
            },
        );
        self.used.insert(id.into());
        Ok(Availability::Available(id.into()))
    }
    /// Default denial is explicit: pass a deny-all closure unless the local
    /// application has authorized this exact artifact/episode. ID knowledge is
    /// not authority. Returned bytes borrow the registry; revoke/drop prevents
    /// future resolution, but cannot recall copies already released to a reader.
    pub fn resolve<'a>(
        &'a self,
        id: &str,
        result: &EpisodeResult,
        authorize: impl FnOnce(&str, &EpisodeKey) -> bool,
    ) -> Result<&'a [u8], Error> {
        let artifact = self.artifacts.get(id).ok_or(Error::Unknown)?;
        if !authorize(id, &artifact.episode) {
            return Err(Error::Denied);
        }
        if artifact.binding != binding(result) {
            return Err(Error::Binding);
        }
        check(&artifact.bytes, result)?;
        Ok(&artifact.bytes)
    }
    pub fn remove(&mut self, id: &str) -> Result<(), Error> {
        self.artifacts.remove(id).map(|_| ()).ok_or(Error::Unknown)
    }
}
fn binding(result: &EpisodeResult) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    Sha256::digest(
        serde_json::to_vec(&(
            &result.inputs.config,
            result.inputs.master,
            result.inputs.ordinal,
            &result.history,
            &result.snapshot,
            &result.trajectory,
            format!("{:?}", result.status),
            result.capture_requested,
            &result.final_observations,
            result.accepted_decisions,
            format!("{:?}", result.budget),
        ))
        .expect("owned result is serializable"),
    )
    .to_vec()
}
fn check(bytes: &[u8], result: &EpisodeResult) -> Result<(), Error> {
    let game = crate::opening::replay::played::verify(bytes).map_err(|_| Error::Corrupt)?;
    if Some(result.status) != game.outcome().map(Status::Completed)
        || normalized(&game.snapshot())? != normalized(&result.snapshot)?
    {
        return Err(Error::Binding);
    }
    Ok(())
}
// Snapshots contain process-local object-store and decision namespaces. Compare
// all other state, including identities/incarnations, zones, work and RNG.
fn normalized(bytes: &[u8]) -> Result<serde_json::Value, Error> {
    use serde_json::{Value, json};
    fn scrub(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    if key == "scope" || key == "store" {
                        *value = json!(0);
                    } else {
                        scrub(value);
                    }
                }
            }
            Value::Array(values) => values.iter_mut().for_each(scrub),
            _ => (),
        }
    }
    let envelope: Value = serde_json::from_slice(bytes).map_err(|_| Error::Corrupt)?;
    let mut value: Value =
        serde_json::from_str(envelope["payload"].as_str().ok_or(Error::Corrupt)?)
            .map_err(|_| Error::Corrupt)?;
    value["objects"]["id"] = json!(0);
    scrub(&mut value);
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        episode::{Driver, Failure, Progress},
        game::{Config, policy},
        objects::Seat,
        trajectory::{Header, Limits, Versions},
    };
    use std::num::NonZeroUsize;
    const ID: &str = "45b374c9-a4db-44fe-a681-603c4055708a";
    fn result(run: &str, ordinal: u64, complete: bool) -> EpisodeResult {
        let header = Header {
            id: EpisodeKey {
                run: run.into(),
                ordinal,
            },
            versions: Versions {
                schema: 2,
                engine: "test".into(),
                rules: "cr".into(),
                cards: "pool".into(),
                action: "policy-v1".into(),
                observation: 1,
            },
            deck_hashes: ["a".repeat(64), "b".repeat(64)],
            config_hash: "c".repeat(64),
            policies: ["test".into(), "test".into()],
            starting_seat: 0,
            limits: Limits::default(),
            restricted_replay: None,
        };
        let mut d = Driver::new(256).unwrap();
        d.reset_captured(&Config::default(), 162, ordinal, NonZeroUsize::MAX, &header)
            .unwrap();
        for seat in [Seat::P0, Seat::P1] {
            let decision = d.observe(seat).unwrap().decision.unwrap();
            d.submit(
                seat,
                &policy::Submission {
                    schema_version: 1,
                    revision: decision.revision,
                    generation: decision.generation,
                    choices: vec![policy::Choice::Keep],
                },
            )
            .unwrap();
        }
        while d.advance(NonZeroUsize::MAX).unwrap() == Progress::InternalYield {}
        if complete {
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
        }
        d.finish().unwrap()
    }
    const RUN: &str = "a34c952c-723c-44ef-95f9-dcdb066db576";
    #[test]
    fn default_denial_exact_artifact_authority_and_registry_lifetime() {
        let mut a = result(RUN, 0, true);
        let mut b = result(RUN, 1, true);
        let mut r = Registry::default();
        assert_eq!(
            r.register(ID, &mut a),
            Ok(Availability::Available(ID.into()))
        );
        assert_eq!(
            r.register("second", &mut b),
            Ok(Availability::Available("second".into()))
        );
        assert_eq!(r.resolve(ID, &a, |_, _| false), Err(Error::Denied));
        assert_eq!(
            r.resolve("second", &b, |id, _| id == ID),
            Err(Error::Denied)
        );
        assert_eq!(r.resolve(ID, &b, |_, _| true), Err(Error::Binding));
        assert_eq!(
            r.resolve(
                ID,
                &result("b34c952c-723c-44ef-95f9-dcdb066db576", 0, true),
                |_, _| true
            ),
            Err(Error::Binding)
        );
        assert!(r.register(ID, &mut b).is_err());
        r.remove(ID).unwrap();
        assert_eq!(r.resolve(ID, &a, |_, _| true), Err(Error::Unknown));
        assert!(r.register(ID, &mut a).is_err());
        assert_eq!(
            Registry::default().resolve(ID, &a, |_, _| true),
            Err(Error::Unknown)
        );
        assert!(r.resolve("second", &b, |_, _| true).is_ok());
    }
    #[test]
    fn incomplete_and_injected_failure_statuses_never_create_replay() {
        // Status mutations are explicitly synthetic ownership-boundary faults.
        // Real truncated/failed driver behavior is covered in episode budgets.
        for status in [
            Status::Incomplete,
            Status::Truncated(crate::trajectory::Limit::Decisions),
            Status::Failed(Failure::Recording),
        ] {
            let mut a = result(RUN, 0, false);
            a.status = status;
            let mut r = Registry::default();
            assert_eq!(
                r.register(ID, &mut a),
                Ok(Availability::Unavailable(status))
            );
            assert_eq!(r.resolve(ID, &a, |_, _| true), Err(Error::Unknown));
            assert!(a.trajectory().unwrap().header().restricted_replay.is_none());
        }
        let mut a = result(RUN, 0, true);
        a.trajectory = None;
        assert_eq!(
            Registry::default().register(ID, &mut a),
            Ok(Availability::NotCaptured)
        );
    }
    #[test]
    fn real_truncated_and_failed_captures_have_no_artifact() {
        use crate::episode::{Budget, Clock};
        #[derive(Debug)]
        struct Zero;
        impl Clock for Zero {
            fn now_ms(&self) -> u64 {
                0
            }
        }
        for truncate in [true, false] {
            let mut header = result(RUN, 0, true).trajectory.unwrap().header().clone();
            header.limits.decisions = truncate.then_some(1);
            let mut d = Driver::bounded(
                256,
                Budget {
                    limits: header.limits.clone(),
                    work_quantum: NonZeroUsize::MAX,
                    records: NonZeroUsize::MIN,
                },
                Box::new(Zero),
            )
            .unwrap();
            d.reset_captured(&Config::default(), 162, 0, NonZeroUsize::MAX, &header)
                .unwrap();
            for seat in [Seat::P0, Seat::P1] {
                let decision = d.observe(seat).unwrap().decision.unwrap();
                let submitted = d.submit(
                    seat,
                    &policy::Submission {
                        schema_version: 1,
                        revision: decision.revision,
                        generation: decision.generation,
                        choices: vec![policy::Choice::Keep],
                    },
                );
                if seat == Seat::P0 {
                    submitted.unwrap();
                } else {
                    assert!(submitted.is_err());
                }
            }
            let mut a = d.finish().unwrap();
            let expected = if truncate {
                Status::Truncated(crate::trajectory::Limit::Decisions)
            } else {
                Status::Failed(Failure::RecordCapacity)
            };
            assert_eq!(a.status(), expected);
            let mut registry = Registry::default();
            assert_eq!(
                registry.register(ID, &mut a),
                Ok(Availability::Unavailable(expected))
            );
            assert_eq!(registry.resolve(ID, &a, |_, _| true), Err(Error::Unknown));
        }
    }
    #[test]
    fn corrupt_stored_replay_is_never_released() {
        let mut a = result(RUN, 0, true);
        let mut registry = Registry::default();
        registry.register(ID, &mut a).unwrap();
        // Explicit synthetic corruption of a private registry entry.
        let artifact = registry.artifacts.get_mut(ID).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&artifact.bytes).unwrap();
        value["choices"][0]["after"]["life"] = serde_json::json!([19, 20]);
        artifact.bytes = serde_json::to_vec(&value).unwrap();
        assert_eq!(registry.resolve(ID, &a, |_, _| true), Err(Error::Corrupt));
    }
    #[test]
    fn injected_config_history_snapshot_and_status_mismatches_fail() {
        // Deliberate privileged corruption, never a substitute for real capture.
        let original = result(RUN, 0, true);
        for mutation in 0..5 {
            let mut a = original.clone();
            match mutation {
                0 => a.inputs.master += 1,
                1 => a.inputs.ordinal += 1,
                2 => a.inputs.config.starting_seat = 1,
                3 => {
                    a.history.pop();
                }
                _ => a.snapshot = b"broken".to_vec(),
            }
            assert!(
                Registry::default().register(ID, &mut a).is_err(),
                "mutation {mutation}"
            );
            assert!(a.trajectory().unwrap().header().restricted_replay.is_none());
        }
        let mut a = original;
        let mut r = Registry::default();
        r.register(ID, &mut a).unwrap();
        for mutation in 0..4 {
            let mut b = a.clone();
            match mutation {
                0 => b.inputs.master += 1,
                1 => {
                    b.history.pop();
                }
                2 => b.status = Status::Incomplete,
                _ => b.snapshot[0] ^= 1,
            }
            assert_eq!(r.resolve(ID, &b, |_, _| true), Err(Error::Binding));
        }
    }
}
