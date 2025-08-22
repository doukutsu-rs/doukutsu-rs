use std::cell::{Cell, Ref, RefCell, RefMut, UnsafeCell};
use std::mem::{MaybeUninit, transmute};
use std::ops::{ControlFlow, Deref, DerefMut};

use crate::framework::error::{GameError, GameResult};
use crate::game::npc::NPC;

/// Maximum capacity of NPCList
const NPC_LIST_MAX_CAP: usize = 512;

pub struct NPCCell(RefCell<NPC>);

/// A zero-sized token used to control access to the NPC list and prevent borrow
/// panics. Some operations require this token to be provided.
pub struct NPCAccessToken {
    /// Prevent forging an NPCAccessToken outside this module.
    _private: ()
}

pub trait NPCTokenProvider {
    fn unborrow_then<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut NPCAccessToken) -> T;
}

impl NPCTokenProvider for NPCAccessToken {
    fn unborrow_then<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut NPCAccessToken) -> T
    {
        f(self)
    }
}

/// A mutably borrowed NPC from the NPC list. This object can be temporarily
/// unborrowed using `unborrow_then` to retrieve the token.
pub enum BorrowedNPC<'a> {
    Borrowed {
        ref_mut: RefMut<'a, NPC>,
        cell: &'a NPCCell,
        token: &'a mut NPCAccessToken,
    },
    Unborrowed,
}

impl Deref for BorrowedNPC<'_> {
    type Target = NPC;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            BorrowedNPC::Borrowed { ref_mut, .. } => &*ref_mut,
            _ => unreachable!()
        }
    }
}

impl DerefMut for BorrowedNPC<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            BorrowedNPC::Borrowed { ref_mut, .. } => &mut *ref_mut,
            _ => unreachable!()
        }
    }
}

impl NPCTokenProvider for BorrowedNPC<'_> {
    fn unborrow_then<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut NPCAccessToken) -> T
    {
        match self {
            BorrowedNPC::Borrowed { .. } => {
                let old = std::mem::replace(self, BorrowedNPC::Unborrowed);
                let (old_ref_mut, token, cell) = match old {
                    BorrowedNPC::Borrowed { ref_mut, token, cell } => (ref_mut, token, cell),
                    _ => unreachable!()
                };
                drop(old_ref_mut);

                let result = f(token);

                *self = cell.borrow_mut(token);
                result
            }
            _ => unreachable!()
        }
    }
}

impl NPCCell {
    /// Immutably borrows the NPC. The access token is held immutably until the
    /// borrow is dropped.
    pub fn borrow<'a>(&'a self, _token: &'a NPCAccessToken) -> Ref<'a, NPC> {
        // The access token's rules are enforced by this function's signature.
        self.0.borrow()
    }

    /// Borrows the NPC without requiring an access token. The caller must ensure
    /// this does not cause borrow panics.
    /// This should only be used for quick, temporary access.
    pub fn borrow_unmanaged(&self) -> Ref<'_, NPC> {
        self.0.borrow()
    }

    /// Mutably borrows the NPC. The access token is held until the borrow is dropped.
    pub fn borrow_mut<'a>(&'a self, token: &'a mut NPCAccessToken) -> BorrowedNPC<'a> {
        BorrowedNPC::Borrowed {
            ref_mut: self.0.borrow_mut(),
            cell: self,
            token,
        }
    }
    
    /// Borrows the NPC without requiring an access token. The caller must ensure
    /// this does not cause borrow panics.
    /// This should only be used for quick, temporary access.
    pub fn borrow_mut_unmanaged(&self) -> RefMut<'_, NPC> {
        self.0.borrow_mut()
    }
}

/// A data structure for storing an NPC list for current stage.
/// Provides multiple mutable references to NPC objects with internal sanity checks and lifetime bounds.
pub struct NPCList {
    npcs: Box<[NPCCell; NPC_LIST_MAX_CAP]>,
    max_npc: Cell<u16>,
    seed: i32,
}

#[allow(dead_code)]
impl NPCList {
    pub fn new() -> (NPCList, NPCAccessToken) {
        let map = NPCList {
            npcs: Box::new(std::array::from_fn(|_| NPCCell(RefCell::new(NPC::empty())))),
            max_npc: Cell::new(0),
            seed: 0,
        };

        let mut token = NPCAccessToken { _private: () };

        for (idx, npc_ref) in map.npcs.iter().enumerate() {
            npc_ref.borrow_mut(&mut token).id = idx as u16;
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

                *npc_ref.0.borrow_mut() = npc;

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
        *npc_ref.0.borrow_mut() = npc;

        if self.max_npc.get() <= id {
            self.max_npc.replace(id + 1);
        }

        Ok(())
    }

    /// Returns an NPC cell from this list.
    pub fn get_npc(&self, id: usize) -> Option<&NPCCell> {
        self.npcs.get(id)
    }

    /// Returns an iterator that iterates over allocated (not up to it's capacity) NPC slots.
    pub fn iter(&self) -> impl Iterator<Item = &NPCCell> {
        // FIXME: what if max_npc changes during iteration?
        // should we take that into account?
        self.npcs.iter().take(self.max_npc.get() as usize)
    }

    /// Returns an iterator over alive NPC slots.
    /// Unfortunately, due to Rust's borrowing rules, a mutable iterator version isn't possible.
    /// To iterate mutably over alive NPC's, use `for_each_alive_mut`.
    pub fn iter_alive<'a>(&'a self, token: &'a NPCAccessToken) -> NPCListAliveIterator<'a> {
        NPCListAliveIterator::new(self, token)
    }

    /// Calls a closure for each alive NPC.
    /// To allow early exit, use `try_for_each_alive_mut`.
    pub fn for_each_alive_mut<F>(&self, token: &mut impl NPCTokenProvider, mut f: F)
    where
        F: FnMut(BorrowedNPC<'_>)
    {
        token.unborrow_then(|token| {
            for cell in self.iter() {
                if cell.borrow(token).cond.alive() {
                    f(cell.borrow_mut(token));
                }
            }
        });
    }
    
    pub fn try_for_each_alive_mut<F, B>(&self, token: &mut impl NPCTokenProvider, mut f: F) -> Result<(), B>
    where
        F: FnMut(BorrowedNPC<'_>) -> ControlFlow<B, ()>
    {
        token.unborrow_then(|token| {
            for cell in self.iter() {
                if cell.borrow(token).cond.alive() {
                    if let ControlFlow::Break(b) = f(cell.borrow_mut(token)) {
                        return Err(b);
                    }
                }
            }

            Ok(())
        })
    }

    /// Removes all NPCs from this list and resets it's capacity.
    pub fn clear(&self, token: &mut NPCAccessToken) {
        for (idx, npc) in self.iter().enumerate() {
            let mut npc = npc.borrow_mut(token);

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
}

pub struct NPCListAliveIterator<'a> {
    index: u16,
    map: &'a NPCList,
    token: &'a NPCAccessToken,
}

impl<'a> NPCListAliveIterator<'a> {
    pub fn new(map: &'a NPCList, token: &'a NPCAccessToken) -> NPCListAliveIterator<'a> {
        NPCListAliveIterator { index: 0, map, token }
    }
}

impl<'a> Iterator for NPCListAliveIterator<'a> {
    type Item = Ref<'a, NPC>;

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
                Some(ref npc) if (*npc).borrow(self.token).cond.alive() => {
                    return Some(item?.borrow(self.token));
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
        let mut ctr = 20;

        map.spawn(0, npc.clone())?;
        map.spawn(2, npc.clone())?;
        map.spawn(256, npc.clone())?;

        assert_eq!(map.iter_alive(&token).count(), 3);

        for npc_ref in map.iter() {
            let mut npc_ref = npc_ref.borrow_mut(&mut token);

            if ctr > 0 {
                ctr -= 1;
                map.spawn(100, npc.clone())?;
                map.spawn(400, npc.clone())?;
            }

            if npc_ref.cond.alive() {
                npc_ref.test_tick(&map)?;
            }
        }

        assert_eq!(map.iter_alive(&token).count(), 43);

        for npc_ref in map.iter().skip(256) {
            let mut npc_ref = npc_ref.borrow_mut(&mut token);

            if npc_ref.cond.alive() {
                npc_ref.cond.set_alive(false);
            }
        }

        assert_eq!(map.iter_alive(&token).count(), 22);

        assert!(map.spawn((NPC_LIST_MAX_CAP + 1) as u16, npc.clone()).is_err());

        map.clear(&mut token);
        assert_eq!(map.iter_alive(&token).count(), 0);

        for i in 0..map.max_capacity() {
            map.spawn(i, npc.clone())?;
        }

        assert!(map.spawn(0, npc.clone()).is_err());
    }

    Ok(())
}
