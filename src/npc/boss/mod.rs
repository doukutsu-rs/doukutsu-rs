use crate::npc::NPC;

pub struct BossNPCMap {
    pub parts: [NPC; 16]
}

impl BossNPCMap {
    pub fn new() -> BossNPCMap {
        BossNPCMap {
            parts: [NPC::empty(); 16],
        }
    }
}
