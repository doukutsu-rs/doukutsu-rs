use std::cell::{Cell, Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};

use crate::framework::error::{GameError, GameResult};
use crate::game::npc::NPC;

/// Maximum capacity of NPCList
const NPC_LIST_MAX_CAP: usize = 512;

pub struct NPCCell(RefCell<NPC>);

/// A data structure for storing an NPC list for current stage.
/// Provides multiple mutable references to NPC objects with internal sanity checks and lifetime bounds.
pub struct NPCList {
    npcs: Box<[NPCCell; NPC_LIST_MAX_CAP]>,
    max_npc: Cell<u16>,
    seed: i32,
}

impl NPCCell {
    pub fn borrow(&self) -> Ref<'_, NPC> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, NPC> {
        self.0.borrow_mut()
    }
}

// Makes sure the user puts all the NPC's back before performing certain operations on the NPC list.
pub struct NPCAccessToken {
    // Prevent forging an NPCAccessToken outside this module
    _private: ()
}

pub struct ImmutableNPCCell<'a>(&'a NPCCell);

impl ImmutableNPCCell<'_> {
    pub fn borrow(&self) -> Ref<'_, NPC> {
        self.0.borrow()
    }
}

pub struct NPCRefMut<'a> {
    cell: &'a NPCCell,
    ref_mut: Option<RefMut<'a, NPC>>,
    token: &'a mut NPCAccessToken
}

impl NPCRefMut<'_> {
    pub fn unborrow_and<T>(&mut self, f: impl FnOnce(&mut NPCAccessToken) -> T) -> T {
        let ref_mut = self.ref_mut.take();
        drop(ref_mut);
        let result = f(&mut self.token);
        self.ref_mut = Some(self.cell.borrow_mut());
        result
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

pub struct MutableNPCCell<'a>(&'a NPCCell);

impl MutableNPCCell<'_> {
    pub fn borrow(&self) -> Ref<'_, NPC> {
        self.0.borrow()
    }

    // borrow_mut will hold on to the access token until the NPCRefMut is dropped.
    pub fn borrow_mut<'a>(&'a self, token: &'a mut NPCAccessToken) -> NPCRefMut<'a> {
        NPCRefMut {
            cell: self.0,
            ref_mut: Some(self.0.borrow_mut()),
            token
        }
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

    /// Inserts NPC into list in first available slot after given ID.
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
    // Note that the access token has a separate lifetime from self and the returned iterator.
    // This is because the iterator does not hold the token while it is active, it merely requires
    // a token to be available during creation.
    pub fn get_npc<'a: 'b, 'b>(&'a self, id: usize, _token: &NPCAccessToken) -> Option<ImmutableNPCCell<'b>> {
        self.npcs.get(id).map(|npc| ImmutableNPCCell(npc))
    }

    /// Returns a mutable reference to NPC from this list.
    pub fn get_npc_mut<'a: 'b, 'b>(&'a self, id: usize, _token: &NPCAccessToken) -> Option<MutableNPCCell<'b>> {
        self.npcs.get(id).map(|npc| MutableNPCCell(npc))
    }

    /// Returns a reference to NPC from this list.
    /// This function returns an unmanaged reference, so it doesn't take an access token.
    /// It is the caller's responsibility to avoid borrow conflicts.
    pub fn get_npc_unmanaged(&self, id: usize) -> Option<&NPCCell> {
        self.npcs.get(id)
    }

    /// Returns an iterator that iterates over allocated (not up to it's capacity) NPC slots.
    pub fn iter(&self, _token: &NPCAccessToken) -> NPCListIterator {
        NPCListIterator::new(self)
    }
    
    /// Returns an iterator that iterates over allocated (not up to it's capacity) NPC slots.
    pub fn iter_mut(&self, _token: &NPCAccessToken) -> NPCListMutableIterator {
        NPCListMutableIterator::new(self)
    }

    /// Returns an iterator over alive NPC slots.
    pub fn iter_alive(&self, _token: &NPCAccessToken) -> NPCListAliveIterator {
        NPCListAliveIterator::new(self)
    }

    // 
    pub fn iter_alive_mut(&self, _token: &NPCAccessToken) -> NPCListMutableAliveIterator {
        NPCListMutableAliveIterator::new(self)
    }

    /// Removes all NPCs from this list and resets it's capacity.
    pub fn clear(&self, token: &mut NPCAccessToken) {
        for (idx, npc) in self.iter_alive_mut(token).enumerate() {
            *npc.borrow_mut(token) = NPC::empty();
            npc.borrow_mut(token).id = idx as u16;
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
    type Item = ImmutableNPCCell<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.map.max_npc.get() {
            return None;
        }

        let item = self.map.npcs.get(self.index as usize).unwrap();
        self.index += 1;

        Some(ImmutableNPCCell(&item))
    }
}

pub struct NPCListMutableIterator<'a> {
    index: u16,
    map: &'a NPCList,
}

impl<'a> NPCListMutableIterator<'a> {
    pub fn new(map: &'a NPCList) -> NPCListMutableIterator<'a> {
        NPCListMutableIterator { index: 0, map }
    }
}

impl<'a> Iterator for NPCListMutableIterator<'a> {
    type Item = MutableNPCCell<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.map.max_npc.get() {
            return None;
        }

        let item = self.map.npcs.get(self.index as usize).unwrap();
        self.index += 1;

        Some(MutableNPCCell(&item))
    }
}

pub struct NPCListMutableAliveIterator<'a> {
    index: u16,
    map: &'a NPCList,
}

impl<'a> NPCListMutableAliveIterator<'a> {
    pub fn new(map: &'a NPCList) -> NPCListMutableAliveIterator<'a> {
        NPCListMutableAliveIterator { index: 0, map }
    }
}

impl<'a> Iterator for NPCListMutableAliveIterator<'a> {
    type Item = MutableNPCCell<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.map.max_npc.get() {
                return None;
            }

            let item = self.map.npcs.get(self.index as usize);
            self.index += 1;

            match item {
                None => {
                    return None;
                }
                // XXX: BEWARE, obscure logic bugs might appear if the user expects mutably-borrowed objects to be returned here!
                // try_borrow is required to prevent double-borrowing (i.e. tick_n160_puu_black) - in that case, it is safe because
                // only type 161 NPC's should be manipulated there.
                Some(npc) if npc.borrow().cond.alive() => {
                    return Some(MutableNPCCell(&npc));
                }
                _ => {}
            }
        }
    }
}

pub struct NPCListAliveIterator<'a> {
    index: u16,
    map: &'a NPCList,
}

impl<'a> NPCListAliveIterator<'a> {
    pub fn new(map: &'a NPCList) -> NPCListAliveIterator<'a> {
        NPCListAliveIterator { index: 0, map }
    }
}

impl<'a> Iterator for NPCListAliveIterator<'a> {
    type Item = ImmutableNPCCell<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.map.max_npc.get() {
                return None;
            }

            let item = self.map.npcs.get(self.index as usize);
            self.index += 1;

            match item {
                None => {
                    return None;
                }
                Some(npc) if npc.borrow().cond.alive() => {
                    return Some(ImmutableNPCCell(&npc));
                }
                _ => {}
            }
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

        assert_eq!(map.iter_alive(&mut token).count(), 3);

        for npc_ref in map.iter_mut(&mut token) {
            if ctr > 0 {
                ctr -= 1;
                map.spawn(100, npc.clone())?;
                map.spawn(400, npc.clone())?;
            }

            if npc_ref.borrow().cond.alive() {
                npc_ref.borrow_mut(&mut token).test_tick(&map)?;
            }
        }

        assert_eq!(map.iter_alive(&mut token).count(), 43);

        for npc_ref in map.iter_mut(&mut token).skip(256) {
            if npc_ref.borrow().cond.alive() {
                npc_ref.borrow_mut(&mut token).cond.set_alive(false);
            }
        }

        assert_eq!(map.iter_alive(&mut token).count(), 22);

        assert!(map.spawn((NPC_LIST_MAX_CAP + 1) as u16, npc.clone()).is_err());

        map.clear(&mut token);
        assert_eq!(map.iter_alive(&mut token).count(), 0);

        for i in 0..map.max_capacity() {
            map.spawn(i, npc.clone())?;
        }

        // Test access token functionality
        for npc_ref in map.iter_alive_mut(&token) {
            let mut npc = npc_ref.borrow_mut(&mut token);
            drop(npc); // TEST: compilation should fail is this is commented out.
            for _npc_ref2 in map.iter_alive_mut(&token) {}
        }
        
        for npc_ref in map.iter_alive_mut(&token) {
            let mut npc = npc_ref.borrow_mut(&mut token);
            npc.unborrow_and(|mut token| {
                for npc_ref2 in map.iter_alive_mut(&token) {
                    // If unborrow_and is working correctly, this should never trigger
                    // an "already borrowed" panic.
                    npc_ref2.borrow_mut(&mut token).action_counter = 667;
                }
            });
        }

        assert!(map.spawn(0, npc.clone()).is_err());
    }

    Ok(())
}
