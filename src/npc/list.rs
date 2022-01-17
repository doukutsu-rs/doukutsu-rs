use std::cell::{Cell, UnsafeCell};
use std::mem::MaybeUninit;

use crate::framework::error::{GameResult, GameError};

use crate::npc::NPC;

/// Maximum capacity of NPCList
const NPC_LIST_MAX_CAP: usize = 512;

/// A data structure for storing an NPC list for current stage.
/// Provides multiple mutable references to NPC objects with internal sanity checks and lifetime bounds.
pub struct NPCList {
    // UnsafeCell is required because we do mutable aliasing (ik, discouraged), prevents Rust/LLVM
    // from theoretically performing some optimizations that might break the code.
    npcs: Box<UnsafeCell<[NPC; NPC_LIST_MAX_CAP]>>,
    max_npc: Cell<u16>,
}

#[allow(dead_code)]
impl NPCList {
    pub fn new() -> NPCList {
        let map = NPCList {
            npcs: Box::new(UnsafeCell::new(unsafe {
                let mut parts_uninit: [NPC; NPC_LIST_MAX_CAP] = MaybeUninit::uninit().assume_init();

                for part in &mut parts_uninit {
                    *part = NPC::empty();
                }

                parts_uninit
            })),
            max_npc: Cell::new(0),
        };

        unsafe {
            for (idx, npc_ref) in map.npcs_mut().iter_mut().enumerate() {
                npc_ref.id = idx as u16;
            }
        }

        map
    }

    /// Inserts NPC into list in first available slot after given ID.
    pub fn spawn(&self, min_id: u16, mut npc: NPC) -> GameResult {
        let npc_len = unsafe { self.npcs().len() };

        if min_id as usize >= npc_len {
            return Err(GameError::InvalidValue("NPC ID is out of bounds".to_string()));
        }

        for id in min_id..(npc_len as u16) {
            let npc_ref = unsafe { self.npcs_mut().get_unchecked_mut(id as usize) };

            if !npc_ref.cond.alive() {
                npc.id = id;

                if npc.tsc_direction == 0 {
                    npc.tsc_direction = npc.direction as u16;
                }

                npc.init_rng();

                *npc_ref = npc;

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
        let npc_len = unsafe { self.npcs().len() };

        if id as usize >= npc_len {
            return Err(GameError::InvalidValue("NPC ID is out of bounds".to_string()));
        }

        npc.id = id;

        if npc.tsc_direction == 0 {
            npc.tsc_direction = npc.direction as u16;
        }

        npc.init_rng();

        unsafe {
            let npc_ref = self.npcs_mut().get_unchecked_mut(id as usize);
            *npc_ref = npc;
        }

        if self.max_npc.get() <= id {
            self.max_npc.replace(id + 1);
        }

        Ok(())
    }

    /// Returns a mutable reference to NPC from this list.
    pub fn get_npc<'a: 'b, 'b>(&'a self, id: usize) -> Option<&'b mut NPC> {
        unsafe {
            self.npcs_mut().get_mut(id)
        }
    }

    /// Returns an iterator that iterates over allocated (not up to it's capacity) NPC slots.
    pub fn iter(&self) -> NPCListMutableIterator {
        NPCListMutableIterator::new(self)
    }

    /// Returns an iterator over alive NPC slots.
    pub fn iter_alive(&self) -> NPCListMutableAliveIterator {
        NPCListMutableAliveIterator::new(self)
    }

    /// Removes all NPCs from this list and resets it's capacity.
    pub fn clear(&self) {
        for (idx, npc) in self.iter_alive().enumerate() {
            *npc = NPC::empty();
            npc.id = idx as u16;
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

    unsafe fn npcs<'a: 'b, 'b>(&'a self) -> &'b [NPC; NPC_LIST_MAX_CAP] {
        &*self.npcs.get()
    }

    unsafe fn npcs_mut<'a: 'b, 'b>(&'a self) -> &'b mut [NPC; NPC_LIST_MAX_CAP] {
        &mut *self.npcs.get()
    }
}

pub struct NPCListMutableIterator<'a> {
    index: u16,
    map: &'a NPCList,
}

impl<'a> NPCListMutableIterator<'a> {
    pub fn new(map: &'a NPCList) -> NPCListMutableIterator<'a> {
        NPCListMutableIterator {
            index: 0,
            map,
        }
    }
}

impl<'a> Iterator for NPCListMutableIterator<'a> {
    type Item = &'a mut NPC;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.map.max_npc.get() {
            return None;
        }

        let item = unsafe { self.map.npcs_mut().get_mut(self.index as usize) };
        self.index += 1;

        item
    }
}

pub struct NPCListMutableAliveIterator<'a> {
    index: u16,
    map: &'a NPCList,
}

impl<'a> NPCListMutableAliveIterator<'a> {
    pub fn new(map: &'a NPCList) -> NPCListMutableAliveIterator<'a> {
        NPCListMutableAliveIterator {
            index: 0,
            map,
        }
    }
}

impl<'a> Iterator for NPCListMutableAliveIterator<'a> {
    type Item = &'a mut NPC;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.map.max_npc.get() {
                return None;
            }

            let item = unsafe { self.map.npcs_mut().get_mut(self.index as usize) };
            self.index += 1;

            match item {
                None => {
                    return None;
                }
                Some(ref npc) if npc.cond.alive() => {
                    return item;
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
        let map = Box::new(NPCList::new());
        let mut ctr = 20;

        map.spawn(0, npc.clone())?;
        map.spawn(2, npc.clone())?;
        map.spawn(256, npc.clone())?;

        assert_eq!(map.iter_alive().count(), 3);

        for npc_ref in map.iter() {
            if ctr > 0 {
                ctr -= 1;
                map.spawn(100, npc.clone())?;
                map.spawn(400, npc.clone())?;
            }

            if npc_ref.cond.alive() {
                npc_ref.test_tick(&map)?;
            }
        }

        assert_eq!(map.iter_alive().count(), 43);

        for npc_ref in map.iter().skip(256) {
            if npc_ref.cond.alive() {
                npc_ref.cond.set_alive(false);
            }
        }

        assert_eq!(map.iter_alive().count(), 22);

        assert!(map.spawn((NPC_LIST_MAX_CAP + 1) as u16, npc.clone()).is_err());

        map.clear();
        assert_eq!(map.iter_alive().count(), 0);

        for i in 0..map.max_capacity() {
            map.spawn(i, npc.clone())?;
        }

        assert!(map.spawn(0, npc.clone()).is_err());
    }

    Ok(())
}
