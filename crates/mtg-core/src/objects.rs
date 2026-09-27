//! Compact per-game storage; no rules or policy-visible observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardId(u8);
#[derive(Debug)]
pub struct CardIdentity {
    pub key: &'static str,
    pub content_sha256: &'static str,
}
include!("card_identities.rs");
impl CardId {
    pub fn from_key(key: &str) -> Option<Self> {
        IDENTITIES
            .iter()
            .position(|c| c.key == key)
            .map(|i| Self(i as u8))
    }
    pub fn identity(self) -> &'static CardIdentity {
        &IDENTITIES[self.0 as usize]
    }
    pub fn all() -> impl Iterator<Item = Self> {
        (0..IDENTITIES.len()).map(|i| Self(i as u8))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seat {
    P0,
    P1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Library(Seat),
    Hand(Seat),
    Graveyard(Seat),
    Battlefield,
    Stack,
    Exile,
}
impl Zone {
    pub(crate) fn is_public(self) -> bool {
        match self {
            Self::Library(_) | Self::Hand(_) => false,
            Self::Graveyard(_) | Self::Battlefield | Self::Stack | Self::Exile => true,
        }
    }
    pub const ALL: [Self; 9] = [
        Self::Library(Seat::P0),
        Self::Library(Seat::P1),
        Self::Hand(Seat::P0),
        Self::Hand(Seat::P1),
        Self::Graveyard(Seat::P0),
        Self::Graveyard(Seat::P1),
        Self::Battlefield,
        Self::Stack,
        Self::Exile,
    ];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle {
    store: u64,
    epoch: u64,
    generation: u64,
    slot: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Object {
    pub controller: Seat,
    pub tapped: bool,
    pub card: CardId,
    pub owner: Seat,
    pub zone: Zone,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageError {
    InvalidHandle,
    IdentityExhausted,
    CapacityExceeded,
}
/// A fact witnessed at an event, never a live pointer into a hidden zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KnownCard {
    pub card: CardId,
    pub owner: Seat,
    pub zone: Zone,
}
#[derive(Debug, PartialEq, Eq)]
struct Slot {
    generation: u64,
    object: Option<Object>,
}

/// Authoritative storage for one game. Handles are process-local capabilities,
/// not replay IDs. This type deliberately cannot be cloned.
#[derive(Debug, PartialEq, Eq)]
pub struct ObjectStore {
    id: u64,
    epoch: u64,
    slots: Vec<Slot>,
    free: Vec<u32>,
    zones: [Vec<u32>; 9],
    knowledge: [Vec<KnownCard>; 2],
}

impl Zone {
    fn index(self) -> usize {
        match self {
            Self::Library(Seat::P0) => 0,
            Self::Library(Seat::P1) => 1,
            Self::Hand(Seat::P0) => 2,
            Self::Hand(Seat::P1) => 3,
            Self::Graveyard(Seat::P0) => 4,
            Self::Graveyard(Seat::P1) => 5,
            Self::Battlefield => 6,
            Self::Stack => 7,
            Self::Exile => 8,
        }
    }
}

fn next_identity(value: u64) -> Result<u64, StorageError> {
    value.checked_add(1).ok_or(StorageError::IdentityExhausted)
}
fn reserve<T>(buffer: &mut Vec<T>) -> Result<(), StorageError> {
    buffer
        .try_reserve(1)
        .map_err(|_| StorageError::CapacityExceeded)
}
impl ObjectStore {
    pub(crate) fn scope(&self) -> u64 {
        self.id
    }
    pub fn new() -> Result<Self, StorageError> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_STORE: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_STORE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| StorageError::IdentityExhausted)?;
        Ok(Self {
            id,
            epoch: 0,
            slots: Vec::new(),
            free: Vec::new(),
            zones: std::array::from_fn(|_| Vec::new()),
            knowledge: std::array::from_fn(|_| Vec::new()),
        })
    }

    /// Append an object to the destination zone, reusing a vacant slot first.
    /// Storage does not decide which placements are legal game actions.
    pub fn allocate(
        &mut self,
        card: CardId,
        owner: Seat,
        zone: Zone,
    ) -> Result<Handle, StorageError> {
        let (slot, generation) = if let Some(&slot) = self.free.last() {
            (slot, next_identity(self.slots[slot as usize].generation)?)
        } else {
            let slot =
                u32::try_from(self.slots.len()).map_err(|_| StorageError::CapacityExceeded)?;
            reserve(&mut self.slots)?;
            (slot, 0)
        };
        reserve(&mut self.zones[zone.index()])?;
        self.reserve_knowledge(zone, 1)?;
        let entry = Slot {
            generation,
            object: Some(Object {
                card,
                owner,
                zone,
                controller: owner,
                tapped: false,
            }),
        };
        if self.free.pop().is_some() {
            self.slots[slot as usize] = entry;
        } else {
            self.slots.push(entry);
        }
        self.zones[zone.index()].push(slot);
        self.record_public(KnownCard { card, owner, zone });
        Ok(self.handle(slot))
    }

    pub fn get(&self, handle: Handle) -> Result<&Object, StorageError> {
        if handle.store != self.id || handle.epoch != self.epoch {
            return Err(StorageError::InvalidHandle);
        }
        self.slots
            .get(handle.slot as usize)
            .filter(|s| s.generation == handle.generation)
            .and_then(|s| s.object.as_ref())
            .ok_or(StorageError::InvalidHandle)
    }

    /// Reserve a validated batch of distinct moves before any object changes.
    pub(crate) fn prepare_moves(
        &mut self,
        handles: &[Handle],
        zone: Zone,
    ) -> Result<(), StorageError> {
        for (i, &handle) in handles.iter().enumerate() {
            self.get(handle)?;
            if handles[..i].contains(&handle) {
                return Err(StorageError::InvalidHandle);
            }
            next_identity(handle.generation)?;
        }
        self.reserve_knowledge(zone, handles.len())?;
        self.zones[zone.index()]
            .try_reserve(handles.len())
            .map_err(|_| StorageError::CapacityExceeded)
    }

    pub(crate) fn get_mut(&mut self, handle: Handle) -> Result<&mut Object, StorageError> {
        self.get(handle)?;
        Ok(self.slots[handle.slot as usize]
            .object
            .as_mut()
            .expect("validated live slot"))
    }

    /// A different zone creates a new identity (CR 400.7). Same-zone requests
    /// are no-ops; this is not a reordering API. Remaining zone order is stable.
    pub fn move_to(&mut self, handle: Handle, zone: Zone) -> Result<Handle, StorageError> {
        let old_zone = self.get(handle)?.zone;
        if old_zone == zone {
            return Ok(handle);
        }
        let generation = next_identity(handle.generation)?;
        reserve(&mut self.zones[zone.index()])?;
        self.reserve_knowledge(zone, 1)?;
        self.unlink(handle.slot, old_zone);
        let slot = &mut self.slots[handle.slot as usize];
        slot.generation = generation;
        let object = slot.object.as_mut().expect("validated live slot");
        object.zone = zone;
        object.controller = object.owner;
        object.tapped = false;
        let fact = KnownCard {
            card: object.card,
            owner: object.owner,
            zone,
        };
        self.zones[zone.index()].push(handle.slot);
        self.record_public(fact);
        Ok(self.handle(handle.slot))
    }

    /// Remove an object from storage; token rules and effects belong to callers.
    pub fn remove(&mut self, handle: Handle) -> Result<Object, StorageError> {
        let object = *self.get(handle)?;
        reserve(&mut self.free)?;
        self.unlink(handle.slot, object.zone);
        self.slots[handle.slot as usize].object = None;
        self.free.push(handle.slot);
        Ok(object)
    }

    /// Prepare a reset's known capacity bounds without discarding live state.
    pub(crate) fn reserve_reset(
        &mut self,
        slots: usize,
        zones: [usize; 9],
    ) -> Result<(), StorageError> {
        next_identity(self.epoch)?;
        self.slots
            .try_reserve(slots.saturating_sub(self.slots.len()))
            .map_err(|_| StorageError::CapacityExceeded)?;
        for (buffer, capacity) in self.zones.iter_mut().zip(zones) {
            buffer
                .try_reserve(capacity.saturating_sub(buffer.len()))
                .map_err(|_| StorageError::CapacityExceeded)?;
        }
        Ok(())
    }

    /// Clear all live objects while retaining allocations and invalidating every
    /// old handle. Epoch exhaustion leaves the store unchanged.
    pub fn reset(&mut self) -> Result<(), StorageError> {
        self.epoch = next_identity(self.epoch)?;
        self.slots.clear();
        self.free.clear();
        for knowledge in &mut self.knowledge {
            knowledge.clear();
        }
        for zone in &mut self.zones {
            zone.clear();
        }
        Ok(())
    }

    fn reserve_knowledge(&mut self, zone: Zone, count: usize) -> Result<(), StorageError> {
        if zone.is_public() {
            for knowledge in &mut self.knowledge {
                knowledge
                    .try_reserve(count)
                    .map_err(|_| StorageError::CapacityExceeded)?;
            }
        }
        Ok(())
    }
    fn record_public(&mut self, fact: KnownCard) {
        if fact.zone.is_public() {
            for knowledge in &mut self.knowledge {
                knowledge.push(fact);
            }
        }
    }
    pub(crate) fn reveal_to(&mut self, seat: Seat, handle: Handle) -> Result<(), StorageError> {
        let object = *self.get(handle)?;
        let knowledge = &mut self.knowledge[if seat == Seat::P0 { 0 } else { 1 }];
        reserve(knowledge)?;
        knowledge.push(KnownCard {
            card: object.card,
            owner: object.owner,
            zone: object.zone,
        });
        Ok(())
    }
    pub(crate) fn knowledge(&self, seat: Seat) -> &[KnownCard] {
        &self.knowledge[if seat == Seat::P0 { 0 } else { 1 }]
    }

    /// Ordered zone members. Each live slot belongs to exactly one list.
    pub fn in_zone(&self, zone: Zone) -> impl Iterator<Item = Handle> + '_ {
        self.zones[zone.index()]
            .iter()
            .map(|&slot| self.handle(slot))
    }
    /// Slots used since reset, including vacancies (not vector capacity).
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }
    /// Retained capacities: object slots, free list, then `Zone::ALL` order.
    pub fn capacities(&self) -> (usize, usize, [usize; 9]) {
        (
            self.slots.capacity(),
            self.free.capacity(),
            std::array::from_fn(|i| self.zones[i].capacity()),
        )
    }
    fn handle(&self, slot: u32) -> Handle {
        Handle {
            store: self.id,
            epoch: self.epoch,
            generation: self.slots[slot as usize].generation,
            slot,
        }
    }
    /// Internal caller supplies a validated complete permutation of this zone.
    pub(crate) fn reorder(&mut self, zone: Zone, order: &[Handle]) {
        debug_assert_eq!(self.zones[zone.index()].len(), order.len());
        debug_assert!(order.iter().enumerate().all(|(i,h)| self.get(*h).is_ok_and(|o| o.zone == zone) && !order[..i].contains(h)));
        for (slot, handle) in self.zones[zone.index()].iter_mut().zip(order) {
            *slot = handle.slot;
        }
    }
    fn unlink(&mut self, slot: u32, zone: Zone) {
        let members = &mut self.zones[zone.index()];
        let position = members
            .iter()
            .position(|&id| id == slot)
            .expect("live object has a zone");
        members.remove(position);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Finite counters may never wrap and make a historical identity valid again.
    // Inject the boundary directly; 2^64 live operations would be impractical.
    #[test]
    fn objects_identity_exhaustion_is_atomic() {
        assert_eq!(
            next_identity(u64::MAX),
            Err(StorageError::IdentityExhausted)
        );
        let mut s = ObjectStore::new().unwrap();
        let h = s
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        s.slots[h.slot as usize].generation = u64::MAX;
        let last = s.handle(h.slot);
        let before = format!("{s:?}");
        assert_eq!(
            s.move_to(last, Zone::Exile),
            Err(StorageError::IdentityExhausted)
        );
        assert_eq!(format!("{s:?}"), before);
        s.remove(last).unwrap();
        let before = format!("{s:?}");
        assert_eq!(
            s.allocate(
                CardId::from_key("forest").unwrap(),
                Seat::P1,
                Zone::Hand(Seat::P1)
            ),
            Err(StorageError::IdentityExhausted)
        );
        assert_eq!(format!("{s:?}"), before);
        s.epoch = u64::MAX;
        let before = format!("{s:?}");
        assert_eq!(s.reset(), Err(StorageError::IdentityExhausted));
        assert_eq!(format!("{s:?}"), before);
    }

    #[test]
    fn objects_dense_records_and_actual_slot_reuse() {
        assert!(std::mem::size_of::<Object>() <= 8);
        assert!(std::mem::size_of::<Slot>() <= 24);
        let mut s = ObjectStore::new().unwrap();
        let id = CardId::from_key("bear-cub").unwrap();
        let a = s.allocate(id, Seat::P0, Zone::Battlefield).unwrap();
        let b = s.allocate(id, Seat::P1, Zone::Battlefield).unwrap();
        let c = s.allocate(id, Seat::P0, Zone::Battlefield).unwrap();
        s.remove(b).unwrap();
        let d = s.allocate(id, Seat::P1, Zone::Battlefield).unwrap();
        assert_eq!(b.slot, d.slot);
        assert_ne!(b.generation, d.generation);
        assert_eq!(
            s.in_zone(Zone::Battlefield).collect::<Vec<_>>(),
            vec![a, c, d]
        );
        assert_eq!(s.get(b), Err(StorageError::InvalidHandle));
        s.reset().unwrap();
        let e = s.allocate(id, Seat::P0, Zone::Battlefield).unwrap();
        assert_eq!(a.slot, e.slot);
        assert_ne!(a.epoch, e.epoch);
        for h in [a, b, c, d] {
            assert_eq!(s.get(h), Err(StorageError::InvalidHandle));
        }
    }
}
