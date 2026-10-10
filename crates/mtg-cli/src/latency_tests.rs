use super::*;

#[test]
fn literal_buckets_edges_percentiles_zero_and_overflow_tail() {
    let mut s = Sampler::new(Config {
        interval: 1,
        ..Config::default()
    })
    .unwrap();
    // Inclusive upper bounds: equal edge stays in that bucket, edge+1 advances.
    for n in [
        0, 1, 10, 11, 100, 101, 1000, 1001, 10000, 10001, 100000, 100001, 1000000, 1000001,
    ] {
        let selected = s.begin(Phase::Reset);
        s.finish(Phase::Reset, selected, Some(n), false);
    }
    let r = s.report(false, false);
    let p = &r["phases"]["reset"];
    assert_eq!(p["buckets"], json!([1, 2, 2, 2, 2, 2, 2, 1]));
    assert_eq!(p["count"], 14);
    assert_eq!(p["sum_ns"], 2222227);
    assert_eq!(p["p50"], json!({"lower_ns":101,"upper_ns":1000}));
    for key in ["p95", "p99"] {
        assert_eq!(p[key], json!({"lower_ns":1000001,"upper_ns":null}));
    }
    assert_eq!(r["phases"]["encoding"]["status"], "not_measured");
    assert_eq!(r["availability"]["queue_wait"], "not_applicable");
    assert_eq!(r["availability"]["memory_high_water"], "not_measured");
}

#[test]
fn schedule_reset_merge_and_malformed_merge_are_explicit() {
    let config = Config {
        interval: 2,
        ..Config::default()
    };
    let mut a = Sampler::new(config.clone()).unwrap();
    let mut schedule = Vec::new();
    for _ in 0..5 {
        let selected = a.begin(Phase::Policy);
        schedule.push(selected);
        a.finish(Phase::Policy, selected, Some(7), false);
    }
    assert_eq!(schedule, [true, false, true, false, true]);
    let mut total = a.clone();
    total.merge(&a).unwrap();
    let p = &total.report(true, false)["phases"]["policy"];
    assert_eq!(p["attempts"], 10);
    assert_eq!(p["count"], 6);
    assert_eq!(p["skipped"], 4);
    assert_eq!(p["sum_ns"], 42);
    let before = total.clone();
    let mut bad = a.clone();
    bad.phases[Phase::Policy as usize].count += 1;
    assert!(total.merge(&bad).is_err());
    assert_eq!(total, before, "merge rejection must be atomic");
    bad = a.clone();
    bad.config.interval = 3;
    assert!(total.merge(&bad).is_err());
    assert_eq!(total, before);
    bad = a.clone();
    bad.phases[Phase::Policy as usize].sum_ns = 999;
    assert!(total.merge(&bad).is_err());
    assert_eq!(total, before);
    a.reset();
    assert_eq!(a, Sampler::new(config).unwrap());
    assert!(a.begin(Phase::Policy));
}

#[test]
fn invalid_configuration_errors_and_saturation_never_wrap() {
    let mut c = Config {
        interval: 0,
        ..Config::default()
    };
    assert!(Sampler::new(c.clone()).is_err());
    c.interval = 1;
    c.bucket_upper_ns[1] = 0;
    assert!(Sampler::new(c.clone()).is_err());
    c.bucket_upper_ns[1] = 101;
    assert!(Sampler::new(c).is_err());
    let mut s = Sampler::new(Config {
        interval: 1,
        ..Config::default()
    })
    .unwrap();
    let selected = s.begin(Phase::Reset);
    s.finish(Phase::Reset, selected, None, true);
    let r = s.report(false, false);
    let p = &r["phases"]["reset"];
    assert_eq!(p["status"], "unavailable");
    assert_eq!(p["clock_errors"], 1);
    assert_eq!(p["errors"], 1);
    assert_eq!(p["count"], 0);
    assert_eq!(p["p50"], Value::Null);
    let selected = s.begin(Phase::Encoding);
    s.finish(Phase::Encoding, selected, Some(u128::MAX), false);
    let p = &s.report(true, false)["phases"]["encoding"];
    assert_eq!(p["sum_ns"], u64::MAX);
    assert_eq!(p["overflow"], true);
    // Reach saturation with a valid imported local summary, without billions
    // of game operations; retain a literal independently checked count table.
    let h = &mut s.phases[Phase::Policy as usize];
    h.attempts = u64::MAX;
    h.count = u64::MAX;
    h.buckets[0] = u64::MAX;
    let copy = s.clone();
    s.merge(&copy).unwrap();
    assert!(s.phases[Phase::Policy as usize].overflow);
    assert_eq!(s.phases[Phase::Policy as usize].count, u64::MAX);
}
