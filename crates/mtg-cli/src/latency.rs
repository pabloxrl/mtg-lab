//! Bounded, clock-free aggregation owned by the scalar client, never the rules.
use serde_json::{Value, json};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Config {
    pub interval: u64,
    pub bucket_upper_ns: [u64; 7],
}
impl Default for Config {
    fn default() -> Self {
        Self {
            interval: 64,
            bucket_upper_ns: [0, 10, 100, 1000, 10000, 100000, 1000000],
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.interval == 0
            || self.bucket_upper_ns[0] != 0
            || self.bucket_upper_ns.windows(2).any(|w| w[0] >= w[1])
            || self.bucket_upper_ns[6] == u64::MAX
        {
            return Err("invalid latency interval or buckets");
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Phase {
    Reset,
    Transition,
    LegalityAndView,
    Policy,
    Encoding,
    Finalization,
}
const LABELS: [&str; 6] = [
    "reset",
    "transition",
    "legality_and_view",
    "policy",
    "encoding",
    "finalization",
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Histogram {
    attempts: u64,
    count: u64,
    skipped: u64,
    errors: u64,
    clock_errors: u64,
    sum_ns: u64,
    buckets: [u64; 8],
    overflow: bool,
}
fn add(to: &mut u64, n: u64, overflow: &mut bool) {
    match to.checked_add(n) {
        Some(v) => *to = v,
        None => {
            *to = u64::MAX;
            *overflow = true;
        }
    }
}
impl Histogram {
    fn valid(&self, edges: &[u64; 7]) -> bool {
        let attempts =
            u128::from(self.count) + u128::from(self.skipped) + u128::from(self.clock_errors);
        let buckets: u128 = self.buckets.iter().map(|n| u128::from(*n)).sum();
        let bound = |n: u128| {
            if self.overflow {
                n.min(u128::from(u64::MAX))
            } else {
                n
            }
        };
        if bound(attempts) != u128::from(self.attempts)
            || bound(buckets) != u128::from(self.count)
            || self.errors > self.attempts
            || (self.count == 0 && self.sum_ns != 0)
        {
            return false;
        }
        if !self.overflow {
            let mut low = 0u128;
            let mut high = 0u128;
            for (i, &n) in self.buckets.iter().enumerate() {
                low += u128::from(n) * u128::from(if i == 0 { 0 } else { edges[i - 1] + 1 });
                high += u128::from(n) * u128::from(edges.get(i).copied().unwrap_or(u64::MAX));
            }
            if !(low..=high).contains(&u128::from(self.sum_ns)) {
                return false;
            }
        }
        true
    }
    fn merge(&mut self, other: &Self) {
        self.overflow |= other.overflow;
        for (a, b) in [
            (&mut self.attempts, other.attempts),
            (&mut self.count, other.count),
            (&mut self.skipped, other.skipped),
            (&mut self.errors, other.errors),
            (&mut self.clock_errors, other.clock_errors),
            (&mut self.sum_ns, other.sum_ns),
        ] {
            add(a, b, &mut self.overflow);
        }
        for (a, b) in self.buckets.iter_mut().zip(other.buckets) {
            add(a, b, &mut self.overflow);
        }
    }
    fn percentile(&self, edges: &[u64; 7], p: u128) -> Value {
        if self.count == 0 || self.overflow {
            return Value::Null;
        }
        let rank = (u128::from(self.count) * p).div_ceil(100);
        let mut count = 0u128;
        for (i, n) in self.buckets.iter().enumerate() {
            count += u128::from(*n);
            if count >= rank {
                return json!({"lower_ns": if i == 0 {0} else {edges[i-1]+1}, "upper_ns":edges.get(i)});
            }
        }
        Value::Null
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Sampler {
    config: Config,
    phases: [Histogram; 6],
    next: [u64; 6],
}
impl Sampler {
    pub fn new(config: Config) -> Result<Self, &'static str> {
        config.validate()?;
        Ok(Self {
            config,
            phases: std::array::from_fn(|_| Histogram::default()),
            next: [0; 6],
        })
    }
    pub fn reset(&mut self) {
        self.phases = std::array::from_fn(|_| Histogram::default());
        self.next = [0; 6];
    }
    pub fn begin(&mut self, phase: Phase) -> bool {
        let i = phase as usize;
        let selected = self.next[i] == 0;
        self.next[i] = if self.next[i] == self.config.interval - 1 {
            0
        } else {
            self.next[i] + 1
        };
        let h = &mut self.phases[i];
        add(&mut h.attempts, 1, &mut h.overflow);
        if !selected {
            add(&mut h.skipped, 1, &mut h.overflow);
        }
        selected
    }
    pub fn finish(&mut self, phase: Phase, selected: bool, duration: Option<u128>, failed: bool) {
        let h = &mut self.phases[phase as usize];
        if failed {
            add(&mut h.errors, 1, &mut h.overflow);
        }
        if !selected {
            return;
        }
        match duration {
            None => add(&mut h.clock_errors, 1, &mut h.overflow),
            Some(n) => {
                let n = u64::try_from(n).unwrap_or_else(|_| {
                    h.overflow = true;
                    u64::MAX
                });
                let bucket = self
                    .config
                    .bucket_upper_ns
                    .iter()
                    .position(|edge| n <= *edge)
                    .unwrap_or(7);
                add(&mut h.buckets[bucket], 1, &mut h.overflow);
                add(&mut h.count, 1, &mut h.overflow);
                add(&mut h.sum_ns, n, &mut h.overflow);
            }
        }
    }
    /// Aggregate completed local streams. Sampling cursor remains local to the
    /// destination; merged counts do not pretend to be one contiguous schedule.
    pub fn merge(&mut self, other: &Self) -> Result<(), &'static str> {
        self.config.validate()?;
        other.config.validate()?;
        if self.config != other.config
            || self
                .phases
                .iter()
                .chain(&other.phases)
                .any(|h| !h.valid(&self.config.bucket_upper_ns))
        {
            return Err("incompatible or malformed latency merge");
        }
        for (a, b) in self.phases.iter_mut().zip(&other.phases) {
            a.merge(b);
        }
        Ok(())
    }
    pub fn report(&self, encode: bool, script: bool) -> Value {
        let mut phases = serde_json::Map::new();
        for (i, h) in self.phases.iter().enumerate() {
            let status = if h.clock_errors > 0 || h.overflow {
                "unavailable"
            } else if h.count > 0 {
                "measured"
            } else if script && matches!(i, 2..=4) {
                "not_applicable"
            } else {
                "not_measured"
            };
            phases.insert(
                LABELS[i].into(),
                json!({"status":status,"attempts":h.attempts,
                "count":h.count,"skipped":h.skipped,"errors":h.errors,"clock_errors":h.clock_errors,
                "sum_ns":h.sum_ns,"buckets":h.buckets,"overflow":h.overflow,
                "p50":h.percentile(&self.config.bucket_upper_ns,50),
                "p95":h.percentile(&self.config.bucket_upper_ns,95),
                "p99":h.percentile(&self.config.bucket_upper_ns,99)}),
            );
        }
        json!({"schema_version":1,"interval":self.config.interval,"bucket_upper_ns":self.config.bucket_upper_ns,
            "schedule":"per-phase ordinal 0 then every interval; local streams restart independently",
            "unit":"ns","percentile_method":"nearest-rank bucket bounds","encoding_enabled":encode,
            "phases":phases,"availability":{"queue_wait":"not_applicable","inference":"not_applicable",
                "worker_busy_idle":"not_applicable","memory_high_water":"not_measured"}})
    }
}
#[cfg(test)]
#[path = "latency_tests.rs"]
mod tests;
