// SYS-CORE-002, R0002-B015; independent equations and provenance: doc/rng.md.
// These literals were calculated before production implementation, by vectors.py.
use mtg_core::rng::{EpisodeRng, Stream, UnsupportedVersion, VERSION};

fn draw(rng: &mut EpisodeRng) -> [u64; 4] {
    std::array::from_fn(|_| rng.next_u64())
}

#[test]
fn rng_episode_known_answers() {
    let cases = [
        (
            0x0000000000000000,
            0x0000000000000000,
            Stream::Environment,
            [
                0xd2ce134acb3773b6,
                0x5f1b779181118480,
                0xef154eb51f35828c,
                0xad1e9ee9cac09786,
            ],
        ),
        (
            0x0000000000000000,
            0x0000000000000000,
            Stream::PolicySeat0,
            [
                0x98b090a7afc91678,
                0x651e9c3782c41244,
                0xaa2fce28af4dee8c,
                0xd46432ae7a4e62bc,
            ],
        ),
        (
            0x0000000000000000,
            0x0000000000000000,
            Stream::PolicySeat1,
            [
                0x6579c0be8cb53ec3,
                0x7fad6015fefa9879,
                0xaa1d0796cd9ff045,
                0x588b464d274a972f,
            ],
        ),
        (
            0x0000000000000000,
            0x0000000000000001,
            Stream::Environment,
            [
                0x094abdf9ff8162af,
                0x90c92fb3deca1053,
                0x3410310568c69b6f,
                0xdd375165f8620ab8,
            ],
        ),
        (
            0x0000000000000000,
            0x0000000000000001,
            Stream::PolicySeat0,
            [
                0x0843e1efd2f8296f,
                0xbb63054cca970683,
                0xc905f88bf43479c8,
                0x7be5d08559ae9595,
            ],
        ),
        (
            0x0000000000000000,
            0x0000000000000001,
            Stream::PolicySeat1,
            [
                0x972630e44744cd05,
                0x5fa7195287b24d4f,
                0xeff7eb2f09252036,
                0x5e1767a6f6edfbf2,
            ],
        ),
        (
            0x0000000000000001,
            0x0000000000000000,
            Stream::Environment,
            [
                0x4d6fb0c7565eb1e9,
                0x27888ce77fc1a61b,
                0xb387da3a00fbec5a,
                0x72d69401b1e3b852,
            ],
        ),
        (
            0x0000000000000001,
            0x0000000000000000,
            Stream::PolicySeat0,
            [
                0x548b75627a448f6d,
                0x6f86e2d43e795d82,
                0x08b1b88ba7a4a0d8,
                0xaffc399ef5785a5c,
            ],
        ),
        (
            0x0000000000000001,
            0x0000000000000000,
            Stream::PolicySeat1,
            [
                0x0782cbff536c94cf,
                0x1897f1d2ecc64c92,
                0x20d5861904395251,
                0xb5a30a5fb3dd630e,
            ],
        ),
        (
            0x000000000000002a,
            0x0000000000000007,
            Stream::Environment,
            [
                0xac0fccb1e3c9bb1d,
                0xe38557d03b1ef2d5,
                0x22d417ccb3e3e81c,
                0xc1234086f4533f05,
            ],
        ),
        (
            0x000000000000002a,
            0x0000000000000007,
            Stream::PolicySeat0,
            [
                0x54adde746d5abc62,
                0xe038c83f0817fb55,
                0x9d4ddff100cf5271,
                0x28507cbf2ba1539b,
            ],
        ),
        (
            0x000000000000002a,
            0x0000000000000007,
            Stream::PolicySeat1,
            [
                0x9d3d968e3567098d,
                0x76f0345194e14489,
                0x1b39e811c8b6b093,
                0xca98ea22e83882d3,
            ],
        ),
        (
            0xffffffffffffffff,
            0xffffffffffffffff,
            Stream::Environment,
            [
                0x66d3102eb869fda8,
                0x9732e72e34d58ba6,
                0x10462167a1236d06,
                0xd07666515a2da21b,
            ],
        ),
        (
            0xffffffffffffffff,
            0xffffffffffffffff,
            Stream::PolicySeat0,
            [
                0x102ef2e1b4a569c9,
                0xb86907e1944a1c57,
                0xb758a0b870e30632,
                0x265e6048ba84e09f,
            ],
        ),
        (
            0xffffffffffffffff,
            0xffffffffffffffff,
            Stream::PolicySeat1,
            [
                0xdfc359ccf5b3c578,
                0x733ce3d31bd8f692,
                0xa1081f419a166184,
                0xcbb2ba4edf2ea5c5,
            ],
        ),
        (
            0x8000000000000000,
            0x8000000000000000,
            Stream::Environment,
            [
                0xa6b026a45478bbc6,
                0xbbcf8c03b71007f4,
                0x303c72f5a129abb5,
                0xf79857bcb82f1514,
            ],
        ),
        (
            0x8000000000000000,
            0x8000000000000000,
            Stream::PolicySeat0,
            [
                0xf6623a276266c175,
                0xb82d44593b5f2c81,
                0xc686ca90ebfdef7c,
                0xc88d9abb2a9c3631,
            ],
        ),
        (
            0x8000000000000000,
            0x8000000000000000,
            Stream::PolicySeat1,
            [
                0x9b049027d805ddf9,
                0x8e47b0979e3d394f,
                0x156f06631366bf0a,
                0x92f5f215db335d95,
            ],
        ),
    ];
    for (master, episode, stream, expected) in cases {
        let mut rng = EpisodeRng::new(VERSION, master, episode, stream).unwrap();
        assert_eq!(rng.version(), "splitmix64-v1");
        assert_eq!(
            draw(&mut rng),
            expected,
            "master={master} episode={episode}"
        );
    }
}

#[test]
fn rng_unknown_versions_rejected() {
    for version in [
        "",
        "splitmix64",
        "splitmix64-v0",
        "splitmix64-v2",
        "SplitMix64-v1",
        "splitmix64-v1 ",
        "chacha8-v1",
    ] {
        for stream in [
            Stream::Environment,
            Stream::PolicySeat0,
            Stream::PolicySeat1,
        ] {
            assert!(
                matches!(
                    EpisodeRng::new(version, 0, 0, stream),
                    Err(UnsupportedVersion)
                ),
                "{version:?}"
            );
        }
    }
}

#[test]
fn rng_creation_order_and_policy_draws_do_not_change_environment() {
    let ids = [0, 1, 7, 1 << 63, u64::MAX];
    let baseline: Vec<_> = ids
        .iter()
        .map(|&id| {
            let mut r = EpisodeRng::new(VERSION, 42, id, Stream::Environment).unwrap();
            (0..128).map(|_| r.next_u64()).collect::<Vec<_>>()
        })
        .collect();
    // Reverse creation order, then interleave draws; consume unequal policy bursts.
    let mut reordered: Vec<_> = ids
        .iter()
        .rev()
        .map(|&id| EpisodeRng::new(VERSION, 42, id, Stream::Environment).unwrap())
        .collect();
    let mut expected = baseline
        .iter()
        .rev()
        .map(|words| words.iter())
        .collect::<Vec<_>>();
    for step in 0..128 {
        for (index, rng) in reordered.iter_mut().enumerate() {
            for stream in [Stream::PolicySeat0, Stream::PolicySeat1] {
                let mut policy =
                    EpisodeRng::new(VERSION, 42, ids[ids.len() - 1 - index], stream).unwrap();
                for _ in 0..step + index {
                    policy.next_u64();
                }
            }
            assert_eq!(rng.next_u64(), *expected[index].next().unwrap());
        }
    }
    // A fresh same-ID stream restarts independently of all previous consumption.
    for (index, id) in ids.into_iter().enumerate() {
        let mut rng = EpisodeRng::new(VERSION, 42, id, Stream::Environment).unwrap();
        for expected in &baseline[index] {
            assert_eq!(rng.next_u64(), *expected);
        }
    }
}
