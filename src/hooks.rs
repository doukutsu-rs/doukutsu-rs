use crate::caret::CaretType;
use crate::common::{Direction, Rect};
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;
use crate::weapon::bullet::BulletManager;

pub struct CHooks {
    handle_npc: unsafe extern "C" fn(callbacks: *const Callbacks, ctx: *const CtxData),
}

pub struct Callbacks {
    random: unsafe extern "C" fn(ctx: *mut CtxData, min: i32, max: i32) -> i32,
    play_sfx: unsafe extern "C" fn(ctx: *mut CtxData, id: u8),
    set_quake: unsafe extern "C" fn(ctx: *mut CtxData, ticks: u16),
    set_caret: unsafe extern "C" fn(ctx: *mut CtxData, x: i32, y: i32, id: u16, direction: u8),
    get_flag: unsafe extern "C" fn(ctx: *mut CtxData, id: u16) -> bool,
    get_map_data: unsafe extern "C" fn(ctx: *mut CtxData) -> MapData,
    get_player_info: unsafe extern "C" fn(ctx: *mut CtxData) -> PlayerInfo,
    update_player_info: unsafe extern "C" fn(ctx: *mut CtxData, player_info: *const PlayerInfo),
    delete_npc_by_type: unsafe extern "C" fn(ctx: *mut CtxData, id: u16, smoke: bool),
    destroy_npc: unsafe extern "C" fn(ctx: *mut CtxData, npc: *mut NPC),
    vanish_npc: unsafe extern "C" fn(ctx: *mut CtxData, npc: *mut NPC),
    create_npc: unsafe extern "C" fn(
        ctx: *mut CtxData,
        npc_id: u16,
        x: i32,
        y: i32,
        vel_x: i32,
        vel_y: i32,
        direction: u16,
        parent: u16,
        min_id: u16,
    ),
    get_npc: unsafe extern "C" fn(ctx: *mut CtxData, npc_id: u16) -> *mut NPC,
    current_npc: unsafe extern "C" fn(ctx: *mut CtxData) -> *mut NPC,
}

#[repr(C)]
pub struct PlayerInfo {
    x: i32,
    y: i32,
    vel_x: i32,
    vel_y: i32,
    flags: u32,
    equip: u16,
    anim_num: u16,
    cond: u16,
    shock: u8,
    direct: u8,
    up: bool,
    down: bool,
    hit: Rect<u32>
}

#[repr(C)]
pub struct MapData {
    tiles: *const u8,
    attrib: *const u8,
    width: u16,
    height: u16,
}

static mut HOOKS: *mut CHooks = std::ptr::null_mut();
struct CtxData<'a, 'b, 'c, 'd, 'e, 'f>(
    &'a mut NPC,
    &'b mut SharedGameState,
    &'c mut Player,
    &'d NPCList,
    &'e mut Stage,
    &'f BulletManager,
);

pub fn init_hooks() {
    #[cfg(target_os = "linux")]
    unsafe {
        let module: *mut libc::c_void = libc::dlopen(b"./libdrshooks.so\0".as_ptr() as *const _, libc::RTLD_NOW);
        if module.is_null() {
            let error = libc::dlerror();
            let message = std::ffi::CString::from_raw(error).to_string_lossy().to_string();

            log::warn!("Cannot initialize hooks?: {}", message);
            return;
        }

        log::info!("Loaded libhooks...");

        let symbol: *mut libc::c_void = libc::dlsym(module, b"drs_hooks_init\0".as_ptr() as *const _);

        if symbol.is_null() {
            log::warn!("initialization function hasn't been found in libhooks.");
            return;
        }

        let init: unsafe extern "C" fn() -> *mut CHooks = std::mem::transmute(symbol);
        HOOKS = (init)();
    }

    #[cfg(target_os = "windows")]
        unsafe {
        use winapi::um::libloaderapi::LoadLibraryA;
        use winapi::um::libloaderapi::GetProcAddress;
        use winapi::um::errhandlingapi::GetLastError;

        let module = LoadLibraryA(b"drshooks.dll\0".as_ptr() as *const _);

        if module.is_null() {
            let error = GetLastError();

            log::warn!("Cannot initialize hooks?: {:#x}", error);
            return;
        }

        log::info!("Loaded libhooks...");

        let symbol = GetProcAddress(module, b"drs_hooks_init\0".as_ptr() as *const _);

        if symbol.is_null() {
            log::warn!("initialization function hasn't been found in libhooks.");
            return;
        }

        let init: unsafe extern "C" fn() -> *mut CHooks = std::mem::transmute(symbol);
        HOOKS = (init)();
    }
}

pub fn reload_hooks() {}

pub fn run_npc_hook(
    npc: &mut NPC,
    state: &mut SharedGameState,
    players: [&mut Player; 2],
    npc_list: &NPCList,
    stage: &mut Stage,
    bullet_manager: &BulletManager,
) {
    unsafe {
        let mut ctx_data = CtxData(npc, state, players[0], npc_list, stage, bullet_manager);

        unsafe extern "C" fn random(ctx: *mut CtxData, min: i32, max: i32) -> i32 {
            let ctx = &*ctx;

            ctx.0.rng.range(min..max)
        };

        unsafe extern "C" fn play_sfx(ctx: *mut CtxData, id: u8) {
            (*ctx).1.sound_manager.play_sfx(id);
        }

        unsafe extern "C" fn set_quake(ctx: *mut CtxData, ticks: u16) {
            (*ctx).1.quake_counter = ticks;
        }

        unsafe extern "C" fn set_caret(ctx: *mut CtxData, x: i32, y: i32, id: u16, direction: u8) {
            (*ctx).1.create_caret(
                x,
                y,
                CaretType::from_int(id as usize).unwrap_or(CaretType::None),
                Direction::from_int_facing(direction as usize).unwrap_or(Direction::Left),
            );
        }

        unsafe extern "C" fn get_flag(ctx: *mut CtxData, id: u16) -> bool {
            (*ctx).1.get_flag(id as usize)
        }

        unsafe extern "C" fn get_map_data(ctx: *mut CtxData) -> MapData {
            let stage = &(*ctx).4;

            MapData {
                tiles: stage.map.tiles.as_ptr(),
                attrib: stage.map.attrib.as_ptr(),
                width: stage.map.width,
                height: stage.map.height,
            }
        }

        unsafe extern "C" fn get_player_info(ctx: *mut CtxData) -> PlayerInfo {
            let player = &(*ctx).2;

            PlayerInfo {
                x: player.x,
                y: player.y,
                vel_x: player.vel_x,
                vel_y: player.vel_y,
                flags: player.flags.0,
                equip: player.equip.0,
                anim_num: player.anim_num,
                cond: player.cond.0,
                shock: player.shock_counter,
                direct: player.direction as u8,
                up: player.up,
                down: player.down,
                hit: player.hit_bounds,
            }
        }

        unsafe extern "C" fn update_player_info(ctx: *mut CtxData, player_info: *const PlayerInfo) {
            let mut player = &mut (*ctx).2;
            let player_info = &(*player_info);

            player.x = player_info.x;
            player.y = player_info.y;
            player.vel_x = player_info.vel_x;
            player.vel_y = player_info.vel_y;
            player.flags.0 = player_info.flags;
            player.equip.0 = player_info.equip;
            player.cond.0 = player_info.cond;
            player.direction = Direction::from_int(player_info.direct as usize).unwrap_or(Direction::Left);
        }

        unsafe extern "C" fn create_npc(
            ctx: *mut CtxData,
            npc_id: u16,
            x: i32,
            y: i32,
            vel_x: i32,
            vel_y: i32,
            direction: u16,
            parent: u16,
            min_id: u16,
        ) {
            let ctx = &*ctx;

            let mut npc = NPC::create(npc_id, &ctx.1.npc_table);
            npc.cond.set_alive(true);
            npc.x = x;
            npc.y = y;
            npc.vel_x = vel_x;
            npc.vel_y = vel_y;
            npc.direction = Direction::from_int(direction as usize).unwrap_or(Direction::Left);
            npc.tsc_direction = direction;
            npc.parent_id = parent;

            let _ = ctx.3.spawn(min_id, npc);
        };

        unsafe extern "C" fn get_npc(ctx: *mut CtxData, npc_id: u16) -> *mut NPC {
            (*ctx).3.get_npc(npc_id as usize).unwrap() as *mut NPC
        }

        unsafe extern "C" fn delete_npc_by_type(ctx: *mut CtxData, id: u16, smoke: bool) {
            (*ctx).3.kill_npcs_by_type(id, smoke, (*ctx).1);
        }

        unsafe extern "C" fn destroy_npc(ctx: *mut CtxData, npc: *mut NPC) {
            let npc = &mut (*npc);

            npc.cond.set_explode_die(true);
        }

        unsafe extern "C" fn vanish_npc(ctx: *mut CtxData, npc: *mut NPC) {
            let npc = &mut (*npc);

            npc.vanish((*ctx).1);
        }

        unsafe extern "C" fn current_npc(ctx: *mut CtxData) -> *mut NPC {
            (*ctx).0 as *mut NPC
        }

        let callbacks = Callbacks {
            random,
            play_sfx,
            set_quake,
            set_caret,
            get_flag,
            get_map_data,
            get_player_info,
            update_player_info,
            delete_npc_by_type,
            destroy_npc,
            vanish_npc,
            create_npc,
            get_npc,
            current_npc,
        };

        if let Some(hook) = HOOKS.as_ref() {
            (hook.handle_npc)(&callbacks as *const Callbacks, &mut ctx_data as *mut CtxData);
        }
    }
}
