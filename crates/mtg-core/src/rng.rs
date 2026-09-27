//! Versioned, owned episode random streams. See `doc/rng.md` for the wire contract.

/// Identifier covering both the PRNG and episode seed derivation.
pub const VERSION: &str = "splitmix64-v1";

/// Separate random consumption for the environment and each policy seat.
#[derive(Clone, Copy)]
pub enum Stream {
    Environment,
    PolicySeat0,
    PolicySeat1,
}

/// The requested RNG algorithm/version is not supported.
#[derive(Debug, PartialEq, Eq)]
pub struct UnsupportedVersion;

/// An owned stream; no shared or global mutable state.
#[derive(Debug, PartialEq, Eq)]
pub struct EpisodeRng {
    state: u64,
}

const STEP: u64 = 0x9e37_79b9_7f4a_7c15;

fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

impl std::fmt::Display for UnsupportedVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unsupported RNG algorithm/version")
    }
}

impl std::error::Error for UnsupportedVersion {}

impl EpisodeRng {
    /// Start a reproducible stream without consuming any other RNG.
    ///
    /// All master seeds and episode IDs are valid, including zero. Callers must
    /// assign stable episode IDs independently of worker or creation order.
    ///
    /// # Errors
    /// Returns `UnsupportedVersion` unless `version` exactly equals `VERSION`.
    pub fn new(
        version: &str,
        master: u64,
        episode: u64,
        stream: Stream,
    ) -> Result<Self, UnsupportedVersion> {
        if version != VERSION {
            return Err(UnsupportedVersion);
        }
        let domain: u64 = match stream {
            Stream::Environment => 0,
            Stream::PolicySeat0 => 1,
            Stream::PolicySeat1 => 2,
        };
        let state = mix(mix(master)
            ^ mix(episode.wrapping_add(STEP))
            ^ mix(domain.wrapping_add(STEP.wrapping_mul(2))));
        Ok(Self { state })
    }

    /// Algorithm and seed-derivation identifier for restricted run metadata.
    pub fn version(&self) -> &'static str {
        VERSION
    }

    /// Advance once and return a raw 64-bit word (not a bounded sample).
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(STEP);
        mix(self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_primitive_known_answers() {
        let cases = [
            (
                0x0000000000000000,
                [
                    0xe220a8397b1dcdaf,
                    0x6e789e6aa1b965f4,
                    0x06c45d188009454f,
                    0xf88bb8a8724c81ec,
                ],
            ),
            (
                0x0000000000000001,
                [
                    0x910a2dec89025cc1,
                    0xbeeb8da1658eec67,
                    0xf893a2eefb32555e,
                    0x71c18690ee42c90b,
                ],
            ),
            (
                0xffffffffffffffff,
                [
                    0xe4d971771b652c20,
                    0xe99ff867dbf682c9,
                    0x382ff84cb27281e9,
                    0x6d1db36ccba982d2,
                ],
            ),
        ];
        for (state, expected) in cases {
            let mut rng = EpisodeRng { state };
            for word in expected {
                assert_eq!(rng.next_u64(), word);
            }
        }
    }
}
