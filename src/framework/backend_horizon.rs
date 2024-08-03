use std::any::Any;
use std::cell::{RefCell, UnsafeCell};
use std::fmt::format;
use std::mem;
use std::pin::Pin;
use std::ptr::slice_from_raw_parts_mut;

use imgui::{DrawData, TextureId, Ui};
use itertools::all;
use lazy_static::lazy_static;

use deko3d::{
    make_texture_handle, Barrier, BlendFactor, BlendOp, BlendState, CmdBuf, CmdBufMaker, ColorMask, ColorState,
    ColorWriteState, CopyBuf, DepthStencilState, DeviceMaker, Face, Filter, Image, ImageDescriptor, ImageFlags,
    ImageFormat, ImageLayout, ImageLayoutMaker, ImageRect, ImageView, InvalidateFlags, MemBlock, MemBlockFlags,
    MemBlockMaker, MipFilter, Primitive, QueueFlags, QueueMaker, RasterizerState, ResHandle, Sampler,
    SamplerDescriptor, Scissor, Shader, ShaderMaker, Stage, StageFlag, SwapchainMaker, Viewport, VtxAttribSize,
    VtxAttribState, VtxAttribType, VtxBufferState, WrapMode, DK_CMDMEM_ALIGNMENT, DK_MEMBLOCK_ALIGNMENT,
    DK_SHADER_CODE_ALIGNMENT, DK_SHADER_CODE_UNUSABLE_SIZE, DK_UNIFORM_BUF_ALIGNMENT,
};

use crate::common::{Color, Rect};
use crate::framework::backend::{
    Backend, BackendEventLoop, BackendGamepad, BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand,
    VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::gamepad;
use crate::framework::gamepad::{Axis, Button, GamepadType};
use crate::framework::graphics::BlendMode;
use crate::framework::util::field_offset;
use crate::game::shared_game_state::SharedGameState;
use crate::game::Game;

mod nx {
    type NWindow = std::ffi::c_void;

    pub const HID_PAD_NO1: u64 = 1 << 0;
    pub const HID_PAD_NO2: u64 = 1 << 1;
    pub const HID_PAD_NO3: u64 = 1 << 2;
    pub const HID_PAD_NO4: u64 = 1 << 3;
    pub const HID_PAD_NO5: u64 = 1 << 4;
    pub const HID_PAD_NO6: u64 = 1 << 5;
    pub const HID_PAD_NO7: u64 = 1 << 6;
    pub const HID_PAD_NO8: u64 = 1 << 7;
    pub const HID_PAD_OTHER: u64 = 1 << 0x10;
    pub const HID_PAD_HANDHELD: u64 = 1 << 0x20;

    pub const HID_PAD_STYLE_FULL_KEY: u32 = 1 << 0;
    pub const HID_PAD_STYLE_HANDHELD: u32 = 1 << 1;
    pub const HID_PAD_STYLE_JOY_DUAL: u32 = 1 << 2;
    pub const HID_PAD_STYLE_JOY_LEFT: u32 = 1 << 3;
    pub const HID_PAD_STYLE_JOY_RIGHT: u32 = 1 << 4;
    pub const HID_PAD_STYLE_GC: u32 = 1 << 5;
    pub const HID_PAD_STYLE_PALMA: u32 = 1 << 6;
    pub const HID_PAD_STYLE_LARK: u32 = 1 << 7;
    pub const HID_PAD_STYLE_HANDHELD_LARK: u32 = 1 << 8;
    pub const HID_PAD_STYLE_LUCIA: u32 = 1 << 9;
    pub const HID_PAD_STYLE_LAGON: u32 = 1 << 10;
    pub const HID_PAD_STYLE_LAGER: u32 = 1 << 11;
    pub const HID_PAD_STYLE_SYSTEM_EXT: u32 = 1 << 29;
    pub const HID_PAD_STYLE_SYSTEM: u32 = 1 << 30;

    pub const HID_PAD_STYLE_SET_FULL_CTRL: u32 =
        HID_PAD_STYLE_FULL_KEY | HID_PAD_STYLE_HANDHELD | HID_PAD_STYLE_JOY_DUAL;
    pub const HID_PAD_STYLE_SET_STANDARD: u32 =
        HID_PAD_STYLE_SET_FULL_CTRL | HID_PAD_STYLE_JOY_LEFT | HID_PAD_STYLE_JOY_RIGHT;

    extern "C" {
        pub fn nwindowGetDefault() -> *mut NWindow;

        fn padInitializeWithMask(pad_state: *mut PadState, mask: u64);

        fn padConfigureInput(max_players: u32, style_set: u32);

        fn padUpdate(pad_state: *mut PadState);
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct HidAnalogStickState {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct PadState {
        pub id_mask: u8,
        pub active_id_mask: u8,
        pub read_handheld: bool,
        pub active_handheld: bool,
        pub style_set: u32,
        pub attributes: u32,
        pub buttons_cur: u64,
        pub buttons_old: u64,
        pub sticks: [HidAnalogStickState; 2],
        pub gc_triggers: [u32; 2],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct PadRepeater {
        pub button_mask: u64,
        pub counter: i32,
        pub delay: u16,
        pub repeat: u16,
    }

    pub fn pad_configure(max_players: u32, style_set: u32) {
        unsafe {
            padConfigureInput(max_players, style_set);
        }
    }

    impl PadState {
        pub fn initialize(mask: u64) -> Self {
            let mut state = Self {
                id_mask: 0,
                active_id_mask: 0,
                read_handheld: false,
                active_handheld: false,
                style_set: 0,
                attributes: 0,
                buttons_cur: 0,
                buttons_old: 0,
                sticks: [HidAnalogStickState { x: 0, y: 0 }; 2],
                gc_triggers: [0; 2],
            };

            unsafe {
                padInitializeWithMask(&mut state, mask);
            }

            state
        }

        pub fn initialize_any() -> Self {
            Self::initialize(0x1000100FF)
        }

        pub fn initialize_default() -> Self {
            Self::initialize(HID_PAD_NO1 | HID_PAD_HANDHELD)
        }

        pub fn update(&mut self) {
            unsafe {
                padUpdate(self);
            }
        }

        pub fn is_connected(&self) -> bool {
            self.id_mask != 0
        }

        pub fn get_buttons_down(&self) -> u64 {
            self.buttons_cur & !self.buttons_old
        }

        pub fn get_buttons_up(&self) -> u64 {
            !self.buttons_cur & self.buttons_old
        }
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Service {
        session: u32,
        own_handle: u32,
        object_id: u32,
        pointer_buffer_size: u16,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct Event {
        revent: u32,
        wevent: u32,
        auto_clear: bool,
    }

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum AppletId {
        None = 0,
        Application = 1,
    }

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum LibAppletMode {
        AllForeground = 0,
        Background = 1,
        NoUi = 2,
        BackgroundIndirect = 3,
        AllForegroundInitiallyHidden = 4,
    }

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum LibAppletExitReason {
        Normal = 0,
        Canceled = 1,
        Abnormal = 2,
        Unexpected = 10,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct WebCommonTLVStorage {
        data: [u8; 0x2000],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct AppletHolder {
        pub s: Service,
        pub StateChangedEvent: Event,
        pub PopInteractiveOutDataEvent: Event,
        pub mode: LibAppletMode,
        pub layer_handle: u64,
        pub creating_self: bool,
        pub exitreason: LibAppletExitReason,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct WebCommonConfig {
        arg: WebCommonTLVStorage,
        applet_id: AppletId,
        version: u32,
        holder: AppletHolder,
    }

    extern "C" {
        pub fn webPageCreate(config: *mut WebCommonConfig, url: *const std::ffi::c_char) -> u32;

        pub fn webConfigSetWhitelist(config: *mut WebCommonConfig, whitelist: *const std::ffi::c_char) -> u32;

        pub fn webConfigShow(config: *mut WebCommonConfig, out: *mut u32) -> u32;
    }

    impl WebCommonConfig {
        pub fn new() -> WebCommonConfig {
            unsafe { std::mem::zeroed() }
        }
    }

    extern "C" {
        pub fn romfsMountDataStorageFromProgram(program_id: u64, name: *const std::ffi::c_char) -> u32;

        pub fn romfsMountFromCurrentProcess(name: *const std::ffi::c_char) -> u32;
    }
}

pub struct HorizonBackend;

impl HorizonBackend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(HorizonBackend))
    }
}

impl Backend for HorizonBackend {
    fn create_event_loop(&self, _ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>> {
        nx::pad_configure(8, nx::HID_PAD_STYLE_SET_STANDARD);

        let mut gamepads = [
            nx::PadState::initialize_default(),
            nx::PadState::initialize(nx::HID_PAD_NO2),
            nx::PadState::initialize(nx::HID_PAD_NO3),
            nx::PadState::initialize(nx::HID_PAD_NO4),
            nx::PadState::initialize(nx::HID_PAD_NO5),
            nx::PadState::initialize(nx::HID_PAD_NO6),
            nx::PadState::initialize(nx::HID_PAD_NO7),
            nx::PadState::initialize(nx::HID_PAD_NO8),
        ];

        for pad in gamepads.iter_mut() {
            pad.update();
        }

        Ok(Box::new(HorizonEventLoop { gamepads, active: [false; 8] }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct HorizonEventLoop {
    gamepads: [nx::PadState; 8],
    active: [bool; 8],
}

const GAMEPAD_KEYMAP: [Button; 16] = [
    Button::South,
    Button::East,
    Button::West,
    Button::North,
    Button::LeftStick,
    Button::RightStick,
    Button::LeftShoulder,
    Button::RightShoulder,
    Button::LeftShoulder,
    Button::RightShoulder,
    Button::Back,
    Button::Start,
    Button::DPadLeft,
    Button::DPadUp,
    Button::DPadRight,
    Button::DPadDown,
];

const fn align(size: u32, align: u32) -> u32 {
    (size + align - 1) & !(align - 1)
}

impl HorizonEventLoop {
    fn gamepad_update(&mut self, state: &SharedGameState, ctx: &mut Context) {
        for (id, pad) in self.gamepads.iter_mut().enumerate() {
            pad.update();

            let connected = pad.is_connected();
            if connected != self.active[id] {
                if connected {
                    // connected
                    log::info!("Gamepad {} connected", id);

                    let axis_sensitivity = state.settings.get_gamepad_axis_sensitivity(id as u32);
                    ctx.gamepad_context.add_gamepad(HorizonGamepad::new(id as u32), axis_sensitivity);

                    ctx.gamepad_context.set_gamepad_type(id as u32, GamepadType::NintendoSwitchJoyConPair);
                } else {
                    // disconnected
                    log::info!("Gamepad {} disconnected", id);

                    ctx.gamepad_context.remove_gamepad(id as u32);
                }

                self.active[id] = connected;
            }
        }

        for (id, pad) in self.gamepads.iter().enumerate() {
            if !pad.is_connected() {
                continue;
            }

            let buttons_down = pad.get_buttons_down();
            let buttons_up = pad.get_buttons_up();

            for i in 0..GAMEPAD_KEYMAP.len() {
                let button = GAMEPAD_KEYMAP[i];
                let mask = 1 << i;

                if i == 8 {
                    ctx.gamepad_context.set_axis_value(id as u32, Axis::TriggerLeft, if buttons_down & mask != 0 { 1.0 } else { 0.0 });
                    continue;
                } else if i == 9 {
                    ctx.gamepad_context.set_axis_value(id as u32, Axis::TriggerRight, if buttons_down & mask != 0 { 1.0 } else { 0.0 });
                    continue;
                }

                if buttons_down & mask != 0 {
                    ctx.gamepad_context.set_button(id as u32, button, true);
                }

                if buttons_up & mask != 0 {
                    ctx.gamepad_context.set_button(id as u32, button, false);
                }
            }

            let analog_x = pad.sticks[0].x as f64 / 32768.0;
            let analog_y = -pad.sticks[0].y as f64 / 32768.0;

            ctx.gamepad_context.set_axis_value(id as u32, Axis::LeftX, (analog_x).clamp(-1.0, 1.0));
            ctx.gamepad_context.set_axis_value(id as u32, Axis::LeftY, (analog_y).clamp(-1.0, 1.0));
            ctx.gamepad_context.set_axis_value(id as u32, Axis::RightX, (analog_x).clamp(-1.0, 1.0));
            ctx.gamepad_context.set_axis_value(id as u32, Axis::RightY, (analog_y).clamp(-1.0, 1.0));
            ctx.gamepad_context.update_axes(id as u32);
        }
    }
}

impl BackendEventLoop for HorizonEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context) {
        let state_ref = unsafe { &mut *game.state.get() };

        let scale = 1.0;
        ctx.screen_size = (854.0 * scale, 480.0 * scale);
        state_ref.handle_resize(ctx).unwrap();

        loop {
            self.gamepad_update(state_ref, ctx);

            game.update(ctx).unwrap();

            if ctx.shutdown_requested {
                log::info!("Shutting down...");
                break;
            }

            if state_ref.next_scene.is_some() {
                mem::swap(&mut game.scene, &mut state_ref.next_scene);
                state_ref.next_scene = None;
                game.scene.as_mut().unwrap().init(state_ref, ctx).unwrap();
                game.loops = 0;
                state_ref.frame_time = 0.0;
            }

            game.draw(ctx).unwrap();
        }
    }

    fn new_renderer(&self, ctx: *mut Context) -> GameResult<Box<dyn BackendRenderer>> {
        let mut imgui = imgui::Context::create();
        let ctx = unsafe { &mut *ctx };
        imgui.io_mut().display_size = [ctx.screen_size.0, ctx.screen_size.1];
        imgui.fonts().build_alpha8_texture();

        let device = DeviceMaker::new().create();

        Deko3DRenderer::new(device, imgui)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct HorizonGamepad {
    pub id: u32,
}

impl HorizonGamepad {
    fn new(id: u32) -> Box<dyn BackendGamepad> {
        Box::new(HorizonGamepad { id })
    }
}

impl BackendGamepad for HorizonGamepad {
    fn set_rumble(&mut self, low_freq: u16, high_freq: u16, duration_ms: u32) -> GameResult {
        Ok(())
    }

    fn instance_id(&self) -> u32 {
        self.id
    }
}

lazy_static! {
    static ref VERTEX_ATTRIB_STATE: [VtxAttribState; 3] = [
        *VtxAttribState::new()
            .set_offset(field_offset::<VertexData, _, _>(|v| &v.position) as u16)
            .set_size(VtxAttribSize::_2x32)
            .set_type(VtxAttribType::Float),
        *VtxAttribState::new()
            .set_offset(field_offset::<VertexData, _, _>(|v| &v.uv) as u16)
            .set_size(VtxAttribSize::_2x32)
            .set_type(VtxAttribType::Float),
        *VtxAttribState::new()
            .set_offset(field_offset::<VertexData, _, _>(|v| &v.color) as u16)
            .set_size(VtxAttribSize::_4x8)
            .set_type(VtxAttribType::Unorm),
    ];
    static ref VERTEX_BUFFER_STATE: [VtxBufferState; 1] =
        [VtxBufferState { stride: mem::size_of::<VertexData>() as u32, divisor: 0 }];
}

struct Deko3DVertexBuffer {
    buffer: deko3d::MemBlock,
    capacity: usize, // those two are in bytes
    allocated: usize,
}

impl Deko3DVertexBuffer {
    pub fn new(device: &deko3d::Device) -> GameResult<Self> {
        let capacity = 2 * 16 * DK_MEMBLOCK_ALIGNMENT;

        let buffer = MemBlockMaker::new(device, capacity as u32)
            .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached)
            .create();

        Ok(Deko3DVertexBuffer { buffer, capacity: capacity as usize, allocated: 0 })
    }

    pub fn transfer(&mut self, vertices: &[VertexData], device: &deko3d::Device) -> GameResult<()> {
        let allocated = vertices.len() * mem::size_of::<VertexData>();
        let size = allocated.max(16 * DK_MEMBLOCK_ALIGNMENT as usize);
        let size = align(2 * size as u32, DK_MEMBLOCK_ALIGNMENT) as usize;

        if size > u32::MAX as usize {
            return Err(GameError::ResourceLoadError("Vertex buffer too large".to_string()));
        }

        if size > self.capacity {
            self.buffer = MemBlockMaker::new(device, size as u32)
                .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached)
                .create();
            self.capacity = size;
        }

        unsafe {
            (self.buffer.get_cpu_addr() as *mut VertexData).copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
        }

        self.allocated = allocated;

        Ok(())
    }
}

struct Deko3DShader<UBO> {
    ubo_mem_block: deko3d::MemBlock,
    code_mem_block: deko3d::MemBlock,
    vtx_shader: deko3d::Shader,
    frag_shader: deko3d::Shader,
    data: UBO,
}

impl<UBO: Default> Deko3DShader<UBO> {
    pub fn new(
        device: &deko3d::Device,
        vertex_shader_binary: &[u8],
        fragment_shader_binary: &[u8],
    ) -> GameResult<Self> {
        let ubo_size = mem::size_of::<UBO>();
        let ubo_size = align(ubo_size as u32, DK_MEMBLOCK_ALIGNMENT) as usize;

        let ubo_mem_block = MemBlockMaker::new(device, ubo_size as u32)
            .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached)
            .create();

        let vtx_binary_len = align(vertex_shader_binary.len() as u32, DK_SHADER_CODE_ALIGNMENT) as usize;
        let frag_binary_len = align(fragment_shader_binary.len() as u32, DK_SHADER_CODE_ALIGNMENT) as usize;

        let code_size = vtx_binary_len + frag_binary_len + DK_SHADER_CODE_UNUSABLE_SIZE as usize;
        let code_size = align(code_size as u32, DK_MEMBLOCK_ALIGNMENT) as usize;
        let code_mem_block = MemBlockMaker::new(&device, code_size as u32)
            .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached | MemBlockFlags::Code)
            .create();

        unsafe {
            let buf = code_mem_block.get_cpu_addr() as *mut u8;

            let vtx_code = buf;
            let frag_code = buf.add(vtx_binary_len);

            vtx_code.copy_from_nonoverlapping(vertex_shader_binary.as_ptr(), vertex_shader_binary.len());
            frag_code.copy_from_nonoverlapping(fragment_shader_binary.as_ptr(), fragment_shader_binary.len());
        }

        let mut vtx_shader = Shader::new();
        let mut frag_shader = Shader::new();

        ShaderMaker::new(&code_mem_block, 0).initialize(&mut vtx_shader);
        ShaderMaker::new(&code_mem_block, vtx_binary_len as u32).initialize(&mut frag_shader);

        Ok(Deko3DShader { ubo_mem_block, code_mem_block, vtx_shader, frag_shader, data: Default::default() })
    }

    pub fn update_uniforms(&mut self, data: UBO) {
        self.data = data;
    }

    pub fn bind(&self, cmd_buf: &deko3d::CmdBuf) {
        cmd_buf.bind_shaders(StageFlag::GraphicsMask, &[&self.vtx_shader, &self.frag_shader]);
        cmd_buf.bind_uniform_buffer(Stage::Vertex, 0, self.ubo_mem_block.get_gpu_addr(), self.ubo_mem_block.get_size());
        cmd_buf.push_constants(
            self.ubo_mem_block.get_gpu_addr(),
            self.ubo_mem_block.get_size(),
            0,
            mem::size_of::<UBO>() as u32,
            &self.data as *const _ as *const std::ffi::c_void,
        );
        cmd_buf.bind_vtx_attrib_state(&*VERTEX_ATTRIB_STATE);
        cmd_buf.bind_vtx_buffer_state(&*VERTEX_BUFFER_STATE);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VertUBO {
    proj_mtx: [[f32; 4]; 4],
}

impl Default for VertUBO {
    fn default() -> Self {
        VertUBO { proj_mtx: [[0.0; 4]; 4] }
    }
}

struct Deko3DTextureDesc {
    image: deko3d::ImageDescriptor,
    sampler: deko3d::SamplerDescriptor,
}

pub struct Deko3DTexture {
    dimensions: (u16, u16),
    desc_memory: deko3d::MemBlock,
    memory: deko3d::MemBlock,
    image: deko3d::Image,
    vertices: Vec<VertexData>,
    vbo: Deko3DVertexBuffer,
    renderer: *mut Deko3DRenderer,
}

impl Deko3DTexture {
    unsafe fn renderer<'a, 'b: 'a>(&'a self) -> &'b mut Deko3DRenderer {
        unsafe { &mut *self.renderer }
    }
}

impl BackendTexture for Deko3DTexture {
    fn dimensions(&self) -> (u16, u16) {
        self.dimensions
    }

    fn add(&mut self, command: SpriteBatchCommand) {
        let (width, height) = self.dimensions;
        let (tex_scale_x, tex_scale_y) = (1.0 / width as f32, 1.0 / height as f32);

        match command {
            SpriteBatchCommand::DrawRect(src, dest) => {
                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectFlip(mut src, dest, flip_x, flip_y) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }

                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }

                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectTinted(src, dest, color) => {
                let color = color.to_rgba();
                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectFlipTinted(mut src, dest, flip_x, flip_y, color) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }

                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }

                let color = color.to_rgba();

                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
        }
    }

    fn clear(&mut self) {
        self.vertices.clear();
    }

    fn draw(&mut self) -> GameResult<()> {
        let renderer = unsafe { self.renderer() };

        self.vbo.transfer(&self.vertices, &renderer.device)?;

        let cmdbuf = &renderer.cmdbuf[renderer.slot as usize];

        cmdbuf.bind_vtx_buffer(0, self.vbo.buffer.get_gpu_addr(), self.vbo.buffer.get_size());

        let img_offset = field_offset::<Deko3DTextureDesc, _, _>(|d| &d.image);
        let sampler_offset = field_offset::<Deko3DTextureDesc, _, _>(|d| &d.sampler);

        let desc_gpu = self.desc_memory.get_gpu_addr();
        cmdbuf.bind_sampler_descriptor_set(desc_gpu + sampler_offset as u64, 1);
        cmdbuf.bind_image_descriptor_set(desc_gpu + img_offset as u64, 1);
        cmdbuf.bind_textures(Stage::Fragment, 0, &[make_texture_handle(0, 0)]);

        renderer.texture_shader.update_uniforms(VertUBO { proj_mtx: renderer.curr_mtx });
        renderer.texture_shader.bind(cmdbuf);

        cmdbuf.draw(Primitive::Triangles, self.vertices.len() as u32, 1, 0, 0);
        cmdbuf.barrier(Barrier::Fragments, InvalidateFlags::None);

        renderer.queue.submit_commands(cmdbuf.finish_list());
        renderer.queue.wait_idle();
        cmdbuf.clear();

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

const BUFFER_COUNT: u32 = 2; // double buffering

pub struct Deko3DRenderer {
    device: deko3d::Device,
    queue: deko3d::Queue,

    fb_mem_block: deko3d::MemBlock,
    framebuffers: [deko3d::Image; BUFFER_COUNT as usize],
    swapchain: deko3d::Swapchain,

    depth_mem_block: deko3d::MemBlock,
    depthbuffer: deko3d::Image,

    cmdbuf_mem_block: [deko3d::MemBlock; BUFFER_COUNT as usize],
    cmdbuf: [deko3d::CmdBuf; BUFFER_COUNT as usize],

    cmdbuf_ctrl_mem_block: deko3d::MemBlock,
    cmdbuf_ctrl: deko3d::CmdBuf,
    vbo: Deko3DVertexBuffer,

    texture_shader: Deko3DShader<VertUBO>,
    color_shader: Deko3DShader<VertUBO>,

    curr_mtx: [[f32; 4]; 4],
    width: u32,
    height: u32,
    fb_width: u32,
    fb_height: u32,
    slot: i32,

    imgui: UnsafeCell<imgui::Context>,
}

impl Deko3DRenderer {
    fn new(device: deko3d::Device, imgui: imgui::Context) -> GameResult<Box<dyn BackendRenderer>> {
        let fb_width = 854;
        let fb_height = 480;

        let queue = QueueMaker::new(&device).set_flags(QueueFlags::Graphics).create();

        let mut depth_layout = ImageLayout::new();
        ImageLayoutMaker::new(&device)
            .set_flags(ImageFlags::UsageRender | ImageFlags::HwCompression)
            .set_format(ImageFormat::Z24S8)
            .set_dimensions(fb_width, fb_height, 0)
            .initialize(&mut depth_layout);

        let depth_size =
            align(align(depth_layout.get_size() as u32, depth_layout.get_alignment()), DK_MEMBLOCK_ALIGNMENT);

        let depth_mem_block =
            MemBlockMaker::new(&device, depth_size).set_flags(MemBlockFlags::Image | MemBlockFlags::GpuCached).create();

        let mut depthbuffer = Image::new();
        depthbuffer.initialize(&depth_layout, &depth_mem_block, 0);

        let mut fb_layout: ImageLayout = ImageLayout::new();
        ImageLayoutMaker::new(&device)
            .set_flags(ImageFlags::UsageRender | ImageFlags::UsagePresent | ImageFlags::HwCompression)
            .set_format(ImageFormat::RGBA8Unorm)
            .set_dimensions(fb_width, fb_height, 0)
            .initialize(&mut fb_layout);

        let mut framebuffers: [Image; BUFFER_COUNT as usize] = [Image::new(), Image::new()];
        let fb_size = align(align(fb_layout.get_size() as u32, fb_layout.get_alignment()), DK_MEMBLOCK_ALIGNMENT);

        let fb_mem_block = MemBlockMaker::new(&device, framebuffers.len() as u32 * fb_size)
            .set_flags(MemBlockFlags::Image | MemBlockFlags::GpuCached)
            .create();

        for (i, fb) in framebuffers.iter_mut().enumerate() {
            fb.initialize(&fb_layout, &fb_mem_block, i as u32 * fb_size);
        }

        let native_window = unsafe { nx::nwindowGetDefault() };
        let swapchain = SwapchainMaker::new(&device, native_window, &framebuffers).create();

        let cmd_mem_size = 16 * 1024;
        let cmdbuf_size = align(cmd_mem_size, DK_MEMBLOCK_ALIGNMENT);
        let cmdbuf_mem_block: [MemBlock; BUFFER_COUNT as usize] = [
            MemBlockMaker::new(&device, cmdbuf_size)
                .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached)
                .create(),
            MemBlockMaker::new(&device, cmdbuf_size)
                .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached)
                .create(),
        ];

        let cmdbuf: [CmdBuf; BUFFER_COUNT as usize] =
            [CmdBufMaker::new(&device).create(), CmdBufMaker::new(&device).create()];

        for (i, cmdbuf) in cmdbuf.iter().enumerate() {
            cmdbuf.add_memory(&cmdbuf_mem_block[i], 0, cmd_mem_size);
        }

        let cmdbuf_ctrl_mem_block = MemBlockMaker::new(&device, cmdbuf_size)
            .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached)
            .create();

        let cmdbuf_ctrl = CmdBufMaker::new(&device).create();
        cmdbuf_ctrl.add_memory(&cmdbuf_ctrl_mem_block, 0, cmd_mem_size);

        let vbo = Deko3DVertexBuffer::new(&device)?;

        let texture_shader = Deko3DShader::<VertUBO>::new(
            &device,
            include_bytes!("shaders/deko3d/vertex_basic.dksh"),
            include_bytes!("shaders/deko3d/fragment_textured.dksh"),
        )?;
        let color_shader = Deko3DShader::<VertUBO>::new(
            &device,
            include_bytes!("shaders/deko3d/vertex_basic.dksh"),
            include_bytes!("shaders/deko3d/fragment_color.dksh"),
        )?;

        Ok(Box::new(Deko3DRenderer {
            device,
            queue,
            fb_mem_block,
            framebuffers,
            swapchain,
            depth_mem_block,
            depthbuffer,
            cmdbuf_mem_block,
            cmdbuf,
            cmdbuf_ctrl_mem_block,
            cmdbuf_ctrl,
            vbo,
            texture_shader,
            color_shader,
            curr_mtx: [[0.0; 4]; 4],
            width: fb_width,
            height: fb_height,
            fb_width,
            fb_height,
            slot: 0,
            imgui: UnsafeCell::new(imgui),
        }))
    }
}

impl BackendRenderer for Deko3DRenderer {
    fn renderer_name(&self) -> String {
        "deko3d".to_owned()
    }

    fn clear(&mut self, color: Color) {
        let cmdbuf = &self.cmdbuf[self.slot as usize];

        cmdbuf.clear_color_float(0, ColorMask::RGBA, color.r, color.g, color.b, color.a);
    }

    fn present(&mut self) -> GameResult {
        let cmdbuf = &self.cmdbuf[self.slot as usize];

        cmdbuf.barrier(Barrier::Fragments, InvalidateFlags::None);
        cmdbuf.discard_depth_stencil();

        self.queue.submit_commands(cmdbuf.finish_list());
        self.queue.wait_idle();
        self.queue.present_image(&self.swapchain, self.slot);

        Ok(())
    }

    fn prepare_draw(&mut self, width: f32, height: f32) -> GameResult {
        self.slot = self.queue.acquire_image(&self.swapchain);
        let cmdbuf = &self.cmdbuf[self.slot as usize];
        cmdbuf.clear();

        let image_view = ImageView::new(&self.framebuffers[self.slot as usize]);
        let depth_view = ImageView::new(&self.depthbuffer);
        cmdbuf.bind_render_targets(&[&image_view], Some(&depth_view));
        cmdbuf.set_viewports(0, &[Viewport { x: 0.0, y: 0.0, width, height, near: -1000.0, far: 1000.0 }]);
        cmdbuf.set_scissors(0, &[Scissor { x: 0, y: 0, width: width as u32, height: height as u32 }]);
        cmdbuf.clear_color_float(0, ColorMask::RGBA, 0.0, 0.0, 0.0, 1.0);
        cmdbuf.clear_depth_stencil(true, 1.0, 0xff, 0);
        cmdbuf.bind_rasterizer_state(&RasterizerState::new().set_cull_mode(Face::None));
        cmdbuf.bind_color_state(&ColorState::new().set_blend_enable(0, true));
        cmdbuf.bind_color_write_state(&ColorWriteState::new());
        cmdbuf.bind_depth_stencil_state(&DepthStencilState::new().set_depth_test_enable(false));
        cmdbuf.bind_blend_states(0, &[BlendState::new()]);

        self.curr_mtx = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -height, 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        let img_total = width as u32 * height as u32 * 4;
        let desc_memory = MemBlockMaker::new(
            &self.device,
            align(std::mem::size_of::<Deko3DTextureDesc>() as u32, DK_MEMBLOCK_ALIGNMENT),
        )
        .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached)
        .create();

        let mut desc_cpu = unsafe { &mut *(desc_memory.get_cpu_addr() as *mut Deko3DTextureDesc) };
        desc_cpu.sampler = SamplerDescriptor::new();
        desc_cpu.image = ImageDescriptor::new();

        let mut layout = ImageLayout::new();
        ImageLayoutMaker::new(&self.device)
            .set_flags(ImageFlags::UsageRender | ImageFlags::BlockLinear)
            .set_format(ImageFormat::RGBA8Unorm)
            .set_dimensions(width as u32, height as u32, 0)
            .initialize(&mut layout);

        let memory = MemBlockMaker::new(
            &self.device,
            align(layout.get_size() as u32, DK_MEMBLOCK_ALIGNMENT.max(layout.get_alignment())),
        )
        .set_flags(MemBlockFlags::Image | MemBlockFlags::GpuCached)
        .create();

        let mut image = Image::new();
        image.initialize(&layout, &memory, 0);

        desc_cpu.image.initialize(&ImageView::new(&image), false, false);
        desc_cpu.sampler.initialize(
            &Sampler::new().set_filter(Filter::Nearest, Filter::Nearest, MipFilter::None).set_wrap_mode(
                WrapMode::ClampToEdge,
                WrapMode::ClampToEdge,
                WrapMode::ClampToEdge,
            ),
        );

        let vbo = Deko3DVertexBuffer::new(&self.device)?;

        Ok(Box::new(Deko3DTexture {
            dimensions: (width, height),
            desc_memory,
            memory,
            image,
            vertices: Vec::new(),
            vbo,
            renderer: self,
        }))
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        let img_total = width as u32 * height as u32 * 4;
        let desc_memory = MemBlockMaker::new(
            &self.device,
            align(std::mem::size_of::<Deko3DTextureDesc>() as u32, DK_MEMBLOCK_ALIGNMENT),
        )
        .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached)
        .create();

        let scratch_memory = MemBlockMaker::new(&self.device, align(img_total, DK_MEMBLOCK_ALIGNMENT))
            .set_flags(MemBlockFlags::CpuUncached | MemBlockFlags::GpuCached)
            .create();

        unsafe {
            let len = data.len().min(img_total as usize);
            (scratch_memory.get_cpu_addr() as *mut u8).copy_from_nonoverlapping(data.as_ptr(), len);
        }

        let mut desc_cpu = unsafe { &mut *(desc_memory.get_cpu_addr() as *mut Deko3DTextureDesc) };
        desc_cpu.sampler = SamplerDescriptor::new();
        desc_cpu.image = ImageDescriptor::new();

        let desc_gpu = desc_memory.get_gpu_addr();

        let img_offset = field_offset::<Deko3DTextureDesc, _, _>(|d| &d.image);
        let sampler_offset = field_offset::<Deko3DTextureDesc, _, _>(|d| &d.sampler);

        let mut layout = ImageLayout::new();
        ImageLayoutMaker::new(&self.device)
            .set_flags(ImageFlags::UsageRender | ImageFlags::BlockLinear)
            .set_format(ImageFormat::RGBA8Unorm)
            .set_dimensions(width as u32, height as u32, 0)
            .initialize(&mut layout);

        let memory = MemBlockMaker::new(
            &self.device,
            align(layout.get_size() as u32, DK_MEMBLOCK_ALIGNMENT.max(layout.get_alignment())),
        )
        .set_flags(MemBlockFlags::Image | MemBlockFlags::GpuCached)
        .create();

        let mut image = Image::new();
        image.initialize(&layout, &memory, 0);

        desc_cpu.image.initialize(&ImageView::new(&image), false, false);
        desc_cpu.sampler.initialize(
            &Sampler::new().set_filter(Filter::Nearest, Filter::Nearest, MipFilter::None).set_wrap_mode(
                WrapMode::ClampToEdge,
                WrapMode::ClampToEdge,
                WrapMode::ClampToEdge,
            ),
        );

        let cmdbuf = &self.cmdbuf_ctrl;
        // let cmdbuf = &self.cmdbuf[self.slot as usize];
        cmdbuf.bind_sampler_descriptor_set(desc_gpu + sampler_offset as u64, 1);
        cmdbuf.bind_image_descriptor_set(desc_gpu + img_offset as u64, 1);

        cmdbuf.copy_buffer_to_image(
            &CopyBuf { addr: scratch_memory.get_gpu_addr(), rowLength: 0, imageHeight: 0 },
            &ImageView::new(&image),
            &ImageRect { x: 0, y: 0, z: 0, width: width as u32, height: height as u32, depth: 1 },
            0,
        );
        cmdbuf.barrier(Barrier::None, InvalidateFlags::Descriptors);
        self.queue.submit_commands(cmdbuf.finish_list());
        self.queue.wait_idle();
        cmdbuf.clear();

        let vbo = Deko3DVertexBuffer::new(&self.device)?;

        Ok(Box::new(Deko3DTexture {
            dimensions: (width, height),
            desc_memory,
            memory,
            image,
            vertices: Vec::new(),
            vbo,
            renderer: self,
        }))
    }

    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult {
        let cmdbuf = &self.cmdbuf[self.slot as usize];

        match blend {
            BlendMode::None => {
                cmdbuf.bind_blend_states(0, &[BlendState::new()]);
            }
            BlendMode::Add => {
                cmdbuf.bind_blend_states(
                    0,
                    &[*BlendState::new()
                        .set_src_color_blend_factor(BlendFactor::One)
                        .set_dst_color_blend_factor(BlendFactor::One)
                        .set_src_alpha_blend_factor(BlendFactor::One)
                        .set_dst_alpha_blend_factor(BlendFactor::One)],
                );
            }
            BlendMode::Alpha => {
                cmdbuf.bind_blend_states(0, &[BlendState::new()]);
            }
            BlendMode::Multiply => {
                cmdbuf.bind_blend_states(
                    0,
                    &[*BlendState::new()
                        .set_src_color_blend_factor(BlendFactor::Zero)
                        .set_dst_color_blend_factor(BlendFactor::SrcColor)
                        .set_src_alpha_blend_factor(BlendFactor::Zero)
                        .set_dst_alpha_blend_factor(BlendFactor::SrcAlpha)],
                );
            }
        }

        Ok(())
    }

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        if let Some(texture) = texture {
            let deko_texture = texture
                .as_any()
                .downcast_ref::<Deko3DTexture>()
                .ok_or_else(|| GameError::RenderError("This texture was not created by deko3d backend.".to_string()))?;

            let width = deko_texture.dimensions.0 as f32;
            let height = deko_texture.dimensions.1 as f32;

            let cmdbuf = &self.cmdbuf[self.slot as usize];
            let image_view = ImageView::new(&deko_texture.image);
            cmdbuf.bind_render_targets(&[&image_view], None);
            cmdbuf.set_viewports(0, &[Viewport { x: 0.0, y: 0.0, width, height, near: -1000.0, far: 1000.0 }]);
            cmdbuf.set_scissors(0, &[Scissor { x: 0, y: 0, width: width as u32, height: height as u32 }]);

            self.width = width as u32;
            self.height = height as u32;
            self.curr_mtx = [
                [2.0 / width, 0.0, 0.0, 0.0],
                [0.0, 2.0 / -height, 0.0, 0.0],
                [0.0, 0.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ];
        } else {
            let width = self.fb_width as f32;
            let height = self.fb_height as f32;

            let cmdbuf = &self.cmdbuf[self.slot as usize];
            let image_view = ImageView::new(&self.framebuffers[self.slot as usize]);
            let depth_view = ImageView::new(&self.depthbuffer);
            cmdbuf.bind_render_targets(&[&image_view], Some(&depth_view));
            cmdbuf.set_viewports(0, &[Viewport { x: 0.0, y: 0.0, width, height, near: -1000.0, far: 1000.0 }]);
            cmdbuf.set_scissors(0, &[Scissor { x: 0, y: 0, width: width as u32, height: height as u32 }]);

            self.width = self.fb_width;
            self.height = self.fb_height;
            self.curr_mtx = [
                [2.0 / width, 0.0, 0.0, 0.0],
                [0.0, 2.0 / -height, 0.0, 0.0],
                [0.0, 0.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ];
        }
        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect<isize>, color: Color) -> GameResult {
        let cmdbuf = &self.cmdbuf[self.slot as usize];

        let color = color.to_rgba();
        let uv = (0.0, 0.0);

        let vertices = [
            VertexData { position: (rect.left as _, rect.bottom as _), uv, color },
            VertexData { position: (rect.left as _, rect.top as _), uv, color },
            VertexData { position: (rect.right as _, rect.top as _), uv, color },
            VertexData { position: (rect.left as _, rect.bottom as _), uv, color },
            VertexData { position: (rect.right as _, rect.top as _), uv, color },
            VertexData { position: (rect.right as _, rect.bottom as _), uv, color },
        ];

        self.vbo.transfer(&vertices, &self.device);
        cmdbuf.bind_vtx_buffer(0, self.vbo.buffer.get_gpu_addr(), self.vbo.buffer.get_size());

        self.color_shader.update_uniforms(VertUBO { proj_mtx: self.curr_mtx });
        self.color_shader.bind(cmdbuf);

        cmdbuf.draw(Primitive::Triangles, vertices.len() as u32, 1, 0, 0);

        cmdbuf.barrier(Barrier::Fragments, InvalidateFlags::None);
        self.queue.submit_commands(cmdbuf.finish_list());
        self.queue.wait_idle();
        cmdbuf.clear();

        Ok(())
    }

    fn draw_outline_rect(&mut self, _rect: Rect<isize>, _line_width: usize, _color: Color) -> GameResult {
        Ok(())
    }

    fn set_clip_rect(&mut self, rect: Option<Rect>) -> GameResult {
        let width = self.width;
        let height = self.height;
        let cmdbuf = &self.cmdbuf[self.slot as usize];

        if let Some(rect) = rect {
            let x = rect.left.max(0);
            let y = rect.top.max(0);
            let width = (rect.right - x).min(width as isize);
            let height = (rect.bottom - y).min(height as isize);

            let (x, y, width, height) = (x as u32, y as u32, width as u32, height as u32);

            cmdbuf.set_scissors(0, &[Scissor { x, y, width, height }]);
        } else {
            cmdbuf.set_scissors(0, &[Scissor { x: 0, y: 0, width, height }]);
        }
        Ok(())
    }

    fn imgui(&self) -> GameResult<&mut imgui::Context> {
        unsafe { Ok(&mut *self.imgui.get()) }
    }

    fn imgui_texture_id(&self, _texture: &Box<dyn BackendTexture>) -> GameResult<TextureId> {
        Ok(TextureId::from(0))
    }

    fn prepare_imgui(&mut self, _ui: &Ui) -> GameResult {
        Ok(())
    }

    fn render_imgui(&mut self, _draw_data: &DrawData) -> GameResult {
        Ok(())
    }

    fn supports_vertex_draw(&self) -> bool {
        true
    }

    fn draw_triangle_list(
        &mut self,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult<()> {
        let cmdbuf = &self.cmdbuf[self.slot as usize];

        self.vbo.transfer(vertices, &self.device);
        cmdbuf.bind_vtx_buffer(0, self.vbo.buffer.get_gpu_addr(), self.vbo.buffer.get_size());

        match shader {
            BackendShader::Fill | BackendShader::WaterFill(_, _, _) => {
                self.color_shader.update_uniforms(VertUBO { proj_mtx: self.curr_mtx });
                self.color_shader.bind(cmdbuf);
            }
            BackendShader::Texture => {
                self.texture_shader.update_uniforms(VertUBO { proj_mtx: self.curr_mtx });
                self.texture_shader.bind(cmdbuf);
            }
        }

        cmdbuf.draw(Primitive::Triangles, vertices.len() as u32, 1, 0, 0);

        cmdbuf.barrier(Barrier::Fragments, InvalidateFlags::None);
        self.queue.submit_commands(cmdbuf.finish_list());
        self.queue.wait_idle();
        cmdbuf.clear();

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn web_open(url: &str) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};

    let mut config = nx::WebCommonConfig::new();
    unsafe {
        let curl = std::ffi::CString::new(url).unwrap();
        let ret = nx::webPageCreate(&mut config, curl.as_ptr());
        if ret != 0 {
            return Err(Error::new(ErrorKind::Other, "webPageCreate failed"));
        }

        let whitelist = std::ffi::CString::new("^http*").unwrap();
        let ret = nx::webConfigSetWhitelist(&mut config, whitelist.as_ptr());
        if ret != 0 {
            return Err(Error::new(ErrorKind::Other, "webConfigSetWhitelist failed"));
        }

        let ret = nx::webConfigShow(&mut config, std::ptr::null_mut());
        if ret != 0 {
            return Err(Error::new(ErrorKind::Other, "webConfigShow failed"));
        }
    }

    Ok(())
}

pub fn mount_romfs() -> bool {
    // let title_ids = [
    //     (0x01000D9007C28000u64, "Cave Story+ (Japan)"),
    //     (0x0100B7D0022EE000u64, "Cave Story+ (US)"),
    //     (0x0100A55003B5C000u64, "Cave Story+ (EU)"),
    // ];
    // // romfsMountDataStorageFromProgram
    let romfs_partition = std::ffi::CString::new("romfs").unwrap();
    //
    // for &(tid, name) in title_ids.iter() {
    //     log::info!("Trying to mount RomFS for {} [{:016X}]", name, tid);
    //     let ret = unsafe { nx::romfsMountDataStorageFromProgram(tid, romfs_partition.as_ptr()) };
    //     log::info!("romfsMountDataStorageFromProgram returned {:#04x}", ret);
    //     if ret == 0 {
    //         log::info!("RomFS mounted for {} [{:016X}]", name, tid);
    //         return true;
    //     }
    // }
    let ret = unsafe { nx::romfsMountFromCurrentProcess(romfs_partition.as_ptr()) };
    log::info!("romfsMountFromCurrentProcess returned {:#04x}", ret);
    if ret == 0 {
        return true;
    }

    false
}
