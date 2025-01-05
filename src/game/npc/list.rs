use std::cell::{Cell, Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};

use streaming_iterator::{StreamingIterator, StreamingIteratorMut};

use crate::framework::error::{GameError, GameResult};
use crate::game::npc::NPC;

/// Maximum capacity of NPCList
const NPC_LIST_MAX_CAP: usize = 512;
pub struct NPCCell(RefCell<NPC>);

// Enforces the rules of the NPC list: All borrows must be returned before performing certain operations.
pub struct NPCAccessToken {
    // Prevent forging an NPCAccessToken outside this module
    _private: ()
}

pub trait NPCAccessTokenProvider {
    type TokenContainer<'a>: DerefMut<Target = NPCAccessToken> where Self : 'a;

    fn provide<'b>(&'b mut self) -> Self::TokenContainer<'b>;
}

impl NPCAccessTokenProvider for NPCAccessToken {
    type TokenContainer<'a> = &'a mut NPCAccessToken where Self : 'a;

    fn provide<'a>(&'a mut self) -> Self::TokenContainer<'a> {
        self
    }
}

/// A data structure for storing an NPC list for current stage.
/// Provides multiple mutable references to NPC objects with internal sanity checks and lifetime bounds.
pub struct NPCList {
    npcs: Box<[NPCCell; NPC_LIST_MAX_CAP]>,
    max_npc: Cell<u16>,
    seed: i32,
}

impl NPCCell {
    pub fn borrow<'a>(&'a self, _token: &'a NPCAccessToken) -> Ref<'a, NPC> {
        self.0.borrow()
    }

    /// Borrows the NPC without an access token. The caller is responsible for preventing borrow panics.
    pub fn borrow_unmanaged(&self) -> Ref<'_, NPC> {
        self.0.borrow()
    }

    /// Mutably borrows the NPC. The access token is taken and held until the borrow is dropped.
    pub fn borrow_mut<'a>(&'a self, token: &'a mut NPCAccessToken) -> NPCRefMut<'a> {
        NPCRefMut {
            cell: self,
            ref_mut: Some(self.0.borrow_mut()),
            token
        }
    }
    
    /// Borrows the NPC without an access token. The caller is responsible for preventing borrow panics.
    pub fn borrow_mut_unmanaged(&self) -> RefMut<'_, NPC> {
        self.0.borrow_mut()
    }
}

pub struct NPCRefMut<'a> {
    cell: &'a NPCCell,
    ref_mut: Option<RefMut<'a, NPC>>,
    token: &'a mut NPCAccessToken,
}

pub struct ExtractedNPCRefMut<'a, 'b> {
    original: &'b mut NPCRefMut<'a>,
}

impl Deref for ExtractedNPCRefMut<'_, '_> {
    type Target = NPCAccessToken;

    fn deref(&self) -> &Self::Target {
        self.original.token
    }
}

impl DerefMut for ExtractedNPCRefMut<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.original.token
    }
}

impl Drop for ExtractedNPCRefMut<'_, '_> {
    fn drop(&mut self) {
        self.original.ref_mut = Some(self.original.cell.0.borrow_mut());
    }
}

impl<'a> NPCAccessTokenProvider for NPCRefMut<'a> {
    type TokenContainer<'b> = ExtractedNPCRefMut<'a, 'b> where Self : 'b;

    fn provide<'b>(&'b mut self) -> Self::TokenContainer<'b> {
        let ref_mut = self.ref_mut.take();
        drop(ref_mut);
        ExtractedNPCRefMut { original: self }
    }
}

impl Deref for NPCRefMut<'_> {
    type Target = NPC;

    fn deref(&self) -> &Self::Target {
        self.ref_mut.as_deref().unwrap()
    }
}

impl DerefMut for NPCRefMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ref_mut.as_deref_mut().unwrap()
    }
}

#[allow(dead_code)]
impl NPCList {
    pub fn new() -> (NPCList, NPCAccessToken) {
        let map = NPCList {
            npcs: Box::new(std::array::from_fn(|_| NPCCell(RefCell::new(NPC::empty())))),
            max_npc: Cell::new(0),
            seed: 0,
        };

        let token = NPCAccessToken { _private: () };

        for (idx, npc_ref) in map.npcs.iter().enumerate() {
            npc_ref.0.borrow_mut().id = idx as u16;
        }

        (map, token)
    }

    pub fn set_rng_seed(&mut self, seed: i32) {
        self.seed = seed;
    }

    // TODO: require an access token for spawn? might not be worth it.
    /// Inserts NPC into list in first available slot after given ID.
    /// No access token is required since borrowed slots are skipped when searching for an empty slot.
    pub fn spawn(&self, min_id: u16, mut npc: NPC) -> GameResult {
        let npc_len = self.npcs.len();

        if min_id as usize >= npc_len {
            return Err(GameError::InvalidValue("NPC ID is out of bounds".to_string()));
        }

        for id in min_id..(npc_len as u16) {
            let npc_ref = self.npcs.get(id as usize).unwrap();

            if npc_ref.0.try_borrow().is_ok_and(|npc_ref| !npc_ref.cond.alive()) {
                npc.id = id;

                if npc.tsc_direction == 0 {
                    npc.tsc_direction = npc.direction as u16;
                }

                npc.init_rng(self.seed);

                npc_ref.0.replace(npc);

                if self.max_npc.get() <= id {
                    self.max_npc.replace(id + 1);
                }

                return Ok(());
            }
        }

        Err(GameError::InvalidValue("No free NPC slot found!".to_string()))
    }

    /// Inserts the NPC at specified slot.
    pub fn spawn_at_slot(&self, id: u16, mut npc: NPC) -> GameResult {
        let npc_len = self.npcs.len();

        if id as usize >= npc_len {
            return Err(GameError::InvalidValue("NPC ID is out of bounds".to_string()));
        }

        npc.id = id;

        if npc.tsc_direction == 0 {
            npc.tsc_direction = npc.direction as u16;
        }

        npc.init_rng(self.seed);

        let npc_ref = self.npcs.get(id as usize).unwrap();
        npc_ref.0.replace(npc);

        if self.max_npc.get() <= id {
            self.max_npc.replace(id + 1);
        }

        Ok(())
    }

    /// Returns a reference to NPC from this list.
    pub fn get_npc(&self, id: usize) -> Option<&NPCCell> {
        self.npcs.get(id)
    }

    /// Returns an iterator that iterates over allocated (not up to it's capacity) NPC slots.
    pub fn iter(&self) -> NPCListIterator {
        NPCListIterator::new(self)
    }
    
    /// Returns an iterator over alive NPC slots.
    pub fn iter_alive<'a>(&'a self, token: &'a NPCAccessToken) -> impl Iterator<Item = &'a NPCCell> {
        self.iter()
            .filter(|npc| npc.borrow(token).cond.alive())
    }

    /// Returns an iterator over alive NPC slots.
    pub fn iter_alive_with_token<'a>(&'a self, token: &'a NPCAccessToken) -> impl Iterator<Item = (&'a NPCCell, &'a NPCAccessToken)> {
        self.iter()
            .filter(|npc| npc.borrow(token).cond.alive())
            .map(move |npc| (npc, token))
    }

    /// Returns an iterator over alive NPC slots.
    pub fn iter_alive_mut<'a>(&'a self, token: &'a mut NPCAccessToken) -> impl StreamingIteratorMut<Item = (&'a NPCCell, &'a mut NPCAccessToken)> {
        NPCAddTokenIterator::new(streaming_iterator::convert(self.iter()), token)
            .filter(|(npc, token)| npc.borrow(token).cond.alive())
        // XXX: Oh, how I wish this worked.
        // streaming_iterator::convert(self.iter_alive(token))
        //     .map(|npc| (npc, token))
    }

    /// Removes all NPCs from this list and resets it's capacity.
    pub fn clear(&self, token: &mut NPCAccessToken) {
        let mut npc_iter = self.iter_alive_mut(token);
        let mut idx = 0;
        while let Some((npc, token)) = npc_iter.next_mut() {
            *npc.borrow_mut(token) = NPC::empty();
            npc.borrow_mut(token).id = idx as u16;
            idx += 1;
        }

        self.max_npc.replace(0);
    }

    /// Returns current capacity of this NPC list.
    pub fn current_capacity(&self) -> u16 {
        self.max_npc.get()
    }

    /// Returns maximum capacity of this NPC list.
    pub fn max_capacity(&self) -> u16 {
        NPC_LIST_MAX_CAP as u16
    }
}

pub struct NPCListIterator<'a> {
    index: u16,
    map: &'a NPCList,
}

impl<'a> NPCListIterator<'a> {
    pub fn new(map: &'a NPCList) -> NPCListIterator<'a> {
        NPCListIterator { index: 0, map }
    }
}

impl<'a> Iterator for NPCListIterator<'a> {
    type Item = &'a NPCCell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.map.max_npc.get() {
            return None;
        }

        let item = self.map.get_npc(self.index as usize).unwrap();
        self.index += 1;

        Some(item)
    }
}

// Streaming iterator to add an access token to a stream of NPC's
enum NPCAddTokenIteratorHolder<'a> {
    Dummy,
    Inactive(&'a mut NPCAccessToken),
    Active((&'a NPCCell, &'a mut NPCAccessToken)),
}

impl<'a> NPCAddTokenIteratorHolder<'a> {
    fn activate(&mut self, cell: &'a NPCCell) {
        match std::mem::replace(self, NPCAddTokenIteratorHolder::Dummy) {
            Self::Inactive(token) => {
                *self = NPCAddTokenIteratorHolder::Active((cell, token));
            },
            Self::Active((_, token)) => {
                *self = NPCAddTokenIteratorHolder::Active((cell, token));
            },
            _ => unreachable!()
        }
    }

    fn deactivate(&mut self) {
        match std::mem::replace(self, NPCAddTokenIteratorHolder::Dummy) {
            Self::Inactive(_) => { },
            Self::Active((_, token)) => {
                *self = NPCAddTokenIteratorHolder::Inactive(token);
            },
            _ => unreachable!()
        }
    }
}

pub struct NPCAddTokenIterator<'a, I> {
    it: I,
    holder: NPCAddTokenIteratorHolder<'a>,
}

impl<'a, I> NPCAddTokenIterator<'a, I> {
    fn new(it: I, token: &'a mut NPCAccessToken) -> Self {
        Self { it, holder: NPCAddTokenIteratorHolder::Inactive(token) }
    }
}

impl<'a, I> StreamingIterator for NPCAddTokenIterator<'a, I>
where
    I: StreamingIterator<Item = &'a NPCCell>
{
    type Item = (&'a NPCCell, &'a mut NPCAccessToken);

    fn advance(&mut self) {
        self.it.advance();
        match self.it.get() {
            Some(item) => {
                self.holder.activate(item);
            }
            None => {
                self.holder.deactivate();
            }
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if let NPCAddTokenIteratorHolder::Active(item) = &self.holder {
            Some(&item)
        } else {
            None
        }
    }
}

impl<'a, I> StreamingIteratorMut for NPCAddTokenIterator<'a, I>
where
    I: StreamingIteratorMut<Item = &'a NPCCell>
{
    fn get_mut(&mut self) -> Option<&mut Self::Item> {
        if let NPCAddTokenIteratorHolder::Active(ref mut item) = &mut self.holder {
            Some(item)
        } else {
            None
        }
    }
}

#[test]
pub fn test_npc_list() -> GameResult {
    impl NPC {
        fn test_tick(&mut self, _map: &NPCList) -> GameResult {
            self.action_counter += 1;

            Ok(())
        }
    }

    let mut npc = NPC::empty();
    npc.cond.set_alive(true);

    {
        let (map, mut token) = NPCList::new();
        let map = Box::new(map);
        let mut ctr = 20;

        map.spawn(0, npc.clone())?;
        map.spawn(2, npc.clone())?;
        map.spawn(256, npc.clone())?;

        assert_eq!(map.iter_alive(&token).count(), 3);

        for npc_ref in map.iter() {
            if ctr > 0 {
                ctr -= 1;
                map.spawn(100, npc.clone())?;
                map.spawn(400, npc.clone())?;
            }

            if npc_ref.borrow(&token).cond.alive() {
                npc_ref.borrow_mut(&mut token).test_tick(&map)?;
            }
        }

        assert_eq!(map.iter_alive(&token).count(), 43);

        for npc_ref in map.iter().skip(256) {
            if npc_ref.borrow(&token).cond.alive() {
                npc_ref.borrow_mut(&mut token).cond.set_alive(false);
            }
        }

        assert_eq!(map.iter_alive(&token).count(), 22);

        assert!(map.spawn((NPC_LIST_MAX_CAP + 1) as u16, npc.clone()).is_err());

        map.clear(&mut token);
        assert_eq!(map.iter_alive(&token).count(), 0);

        for i in 0..map.max_capacity() {
            map.spawn(i, npc.clone())?;
        }

        // Test access token functionality
        let mut npc_iter = map.iter_alive_mut(&mut token);
        while let Some((npc, token)) = npc_iter.next_mut() {
            let npc = npc.borrow_mut(token);
            drop(npc); // TEST: compilation should fail is this is commented out.
            for _npc_ref2 in map.iter_alive(&token) {}
        }
        drop(npc_iter);

        let mut npc_iter = map.iter_alive_mut(&mut token);
        while let Some((npc_ref, token)) = npc_iter.next_mut() {
            let mut npc = npc_ref.borrow_mut(token);
            let mut token = npc.provide();
            let mut npc_iter = map.iter_alive_mut(&mut token);
            while let Some((npc_ref2, token)) = npc_iter.next_mut() {
                // If provide is working correctly, this should never trigger
                // an "already borrowed" panic.
                npc_ref2.borrow_mut(token).action_counter = 667;
            }
        }
        drop(npc_iter);

        assert!(map.spawn(0, npc.clone()).is_err());
    }

    Ok(())
}
