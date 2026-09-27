//! Independent storage ledgers: RFC R0002-B015 / SYS-CORE-004; CR 400.7
//! supplies new identity on zone changes. These are storage operations, not games.
use mtg_core::objects::{
    CardId, Handle, ObjectStore,
    Seat::*,
    StorageError,
    Zone::{self, *},
};

fn card(key: &str) -> CardId {
    CardId::from_key(key).unwrap()
}
fn ledger(s: &ObjectStore) -> Vec<(Zone, &'static str, mtg_core::objects::Seat)> {
    Zone::ALL
        .into_iter()
        .flat_map(|z| {
            s.in_zone(z).map(move |h| {
                let o = s.get(h).unwrap();
                assert_eq!(o.zone, z);
                (z, o.card.identity().key, o.owner)
            })
        })
        .collect()
}
fn reject(s: &mut ObjectStore, h: Handle) {
    let before = ledger(s);
    assert_eq!(s.get(h), Err(StorageError::InvalidHandle));
    assert_eq!(s.move_to(h, Exile), Err(StorageError::InvalidHandle));
    assert_eq!(s.remove(h), Err(StorageError::InvalidHandle));
    assert_eq!(ledger(s), before);
}

#[test]
fn objects_allocation_and_zone_change_literal_ledgers() {
    let mut s = ObjectStore::new().unwrap();
    let a = s.allocate(card("bear-cub"), P0, Hand(P0)).unwrap();
    let b = s.allocate(card("forest"), P1, Library(P1)).unwrap();
    let c = s.allocate(card("mountain"), P0, Hand(P0)).unwrap();
    assert_eq!(
        ledger(&s),
        vec![
            (Library(P1), "forest", P1),
            (Hand(P0), "bear-cub", P0),
            (Hand(P0), "mountain", P0)
        ]
    );
    let original_b = *s.get(b).unwrap();
    let original_c = *s.get(c).unwrap();
    let a2 = s.move_to(a, Battlefield).unwrap();
    assert_ne!(a, a2);
    reject(&mut s, a);
    assert_eq!(
        ledger(&s),
        vec![
            (Library(P1), "forest", P1),
            (Hand(P0), "mountain", P0),
            (Battlefield, "bear-cub", P0)
        ]
    );
    let a3 = s.move_to(a2, Hand(P0)).unwrap();
    reject(&mut s, a);
    reject(&mut s, a2);
    assert_eq!(s.move_to(a3, Hand(P0)), Ok(a3));
    assert_eq!(
        ledger(&s),
        vec![
            (Library(P1), "forest", P1),
            (Hand(P0), "mountain", P0),
            (Hand(P0), "bear-cub", P0)
        ]
    );
    assert_eq!(s.get(b), Ok(&original_b));
    assert_eq!(s.get(c), Ok(&original_c));
}

#[test]
fn objects_reuse_and_reset_never_revive_retained_handles() {
    let mut s = ObjectStore::new().unwrap();
    let mut old = Vec::new();
    for _ in 0..32 {
        let a = s.allocate(card("bear-cub"), P0, Battlefield).unwrap();
        let b = s.allocate(card("forest"), P1, Hand(P1)).unwrap();
        let c = s.move_to(a, Graveyard(P0)).unwrap();
        old.extend([a, b, c]);
        assert_eq!(s.remove(c).unwrap().card, card("bear-cub"));
        reject(&mut s, c);
        let d = s.allocate(card("mountain"), P0, Exile).unwrap();
        assert_eq!(s.slot_count(), 2, "removed slot must be reused");
        reject(&mut s, c);
        assert_eq!(
            ledger(&s),
            vec![(Hand(P1), "forest", P1), (Exile, "mountain", P0)]
        );
        old.push(d);
        let capacities = s.capacities();
        s.reset().unwrap();
        assert!(ledger(&s).is_empty());
        assert_eq!(s.capacities(), capacities, "reset must retain buffers");
        let fresh = s.allocate(card("forest"), P0, Library(P0)).unwrap();
        for &h in &old {
            reject(&mut s, h);
        }
        assert_eq!(ledger(&s), vec![(Library(P0), "forest", P0)]);
        old.push(fresh);
        s.reset().unwrap();
    }
}

#[test]
fn objects_zone_ownership_order_and_foreign_store_rejection() {
    let mut s = ObjectStore::new().unwrap();
    let mut other = ObjectStore::new().unwrap();
    let foreign = other.allocate(card("bear-cub"), P0, Library(P0)).unwrap();
    let mut h = s.allocate(card("bear-cub"), P0, Library(P0)).unwrap();
    reject(&mut s, foreign);
    reject(&mut other, h);
    let mut old = Vec::new();
    for zone in Zone::ALL {
        if s.get(h).unwrap().zone != zone {
            old.push(h);
        }
        h = s.move_to(h, zone).unwrap();
        assert_eq!(ledger(&s), vec![(zone, "bear-cub", P0)]);
        for &stale in &old {
            reject(&mut s, stale);
        }
    }
    assert_eq!(ledger(&other), vec![(Library(P0), "bear-cub", P0)]);
    s.remove(h).unwrap();
    reject(&mut s, h);
    assert!(ledger(&s).is_empty());
}

#[test]
fn objects_frozen_identities_are_shared_and_match_manifest() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let entries = manifest["cards"].as_array().unwrap();
    assert_eq!(CardId::all().count(), entries.len());
    for entry in entries {
        let id = card(entry["id"].as_str().unwrap());
        let identity = id.identity();
        assert_eq!(
            identity.content_sha256,
            entry["content_sha256"].as_str().unwrap()
        );
        assert!(std::ptr::eq(identity, card(identity.key).identity()));
    }
    assert!(CardId::from_key("unsupported-card").is_none());
    let mut a = ObjectStore::new().unwrap();
    let mut b = ObjectStore::new().unwrap();
    let x = a.allocate(card("bear-cub"), P0, Hand(P0)).unwrap();
    let y = b.allocate(card("bear-cub"), P1, Hand(P1)).unwrap();
    assert!(std::ptr::eq(
        a.get(x).unwrap().card.identity(),
        b.get(y).unwrap().card.identity()
    ));
}
