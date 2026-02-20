use std::any::Any;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::c_void;
use std::io::Read;
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;

use glutin::event::{ElementState, Event, TouchPhase, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest, PossiblyCurrent, WindowedContext};
use imgui::{DrawCmdParams, DrawData, DrawIdx, DrawVert};
use winit::window::Icon;

use crate::common::Rect;
use crate::framework::backend::{Backend, BackendEventLoop, BackendGamepad, BackendRenderer, BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
#[cfg(target_os = "android")]
use crate::framework::gamepad::{Axis, Button};
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::framework::gl;
use crate::framework::keyboard::ScanCode;
use crate::framework::render_opengl::{GLContext, OpenGLRenderer};
use crate::game::Game;
use crate::game::GAME_SUSPENDED;
use crate::input::touch_controls::TouchPoint;

pub struct GlutinBackend;

impl GlutinBackend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(GlutinBackend))
    }
}

impl Backend for GlutinBackend {
    fn create_event_loop(&self, _ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>> {
        #[cfg(target_os = "android")]
        loop {
            match ndk_glue::native_window().as_ref() {
                Some(_) => {
                    log::info!("NativeWindow Found: {:?}", ndk_glue::native_window());
                    break;
                }
                None => (),
            }
        }

        Ok(Box::new(GlutinEventLoop { refs: Rc::new(UnsafeCell::new(None)) }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct GlutinEventLoop {
    refs: Rc<UnsafeCell<Option<WindowedContext<PossiblyCurrent>>>>,
}

impl GlutinEventLoop {
    fn get_context(&self, ctx: &Context, event_loop: &EventLoop<()>) -> &mut WindowedContext<PossiblyCurrent> {
        let mut refs = unsafe { &mut *self.refs.get() };

        if refs.is_none() {
            let mut window = WindowBuilder::new();
            
            let windowed_context = ContextBuilder::new();
            let windowed_context = windowed_context.with_gl(GlRequest::Specific(Api::OpenGl, (3, 0)));
            #[cfg(target_os = "android")]
            let windowed_context = windowed_context.with_gl(GlRequest::Specific(Api::OpenGlEs, (2, 0)));

            let windowed_context = windowed_context
                .with_gl_profile(GlProfile::Core)
                .with_gl_debug_flag(false)
                .with_pixel_format(24, 8)
                .with_vsync(true);

            #[cfg(target_os = "windows")]
            {
                use glutin::platform::windows::WindowBuilderExtWindows;
                window = window.with_drag_and_drop(false);
            }

            window = window.with_title("doukutsu-rs");
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "android", target_os = "horizon")))]
            {
                let mut file = filesystem::open(&ctx, "/builtin/icon.bmp").unwrap();
                let mut buf: Vec<u8> = Vec::new();
                file.read_to_end(&mut buf);
                
                let mut img = match image::load_from_memory_with_format(buf.as_slice(), image::ImageFormat::Bmp) {
                    Ok(image) => image.into_rgba8(),
                    Err(e) => panic!("Cannot set window icon")
                };
                
                let (width, height) = img.dimensions();
                let icon = Icon::from_rgba(img.into_raw(), width, height).unwrap();
                
                window = window.with_window_icon(Some(icon));
            }
            
            let windowed_context = windowed_context.build_windowed(window, event_loop).unwrap();

            let windowed_context = unsafe { windowed_context.make_current().unwrap() };

            #[cfg(target_os = "android")]
            if let Some(nwin) = ndk_glue::native_window().as_ref() {
                unsafe {
                    windowed_context.surface_created(nwin.ptr().as_ptr() as *mut std::ffi::c_void);
                }
            }

            refs.replace(windowed_context);
        }

        refs.as_mut().unwrap()
    }
}

#[cfg(target_os = "android")]
fn request_android_redraw() {
    match ndk_glue::native_window().as_ref() {
        Some(native_window) => {
            let a_native_window: *mut ndk_sys::ANativeWindow = native_window.ptr().as_ptr();
            let a_native_activity: *mut ndk_sys::ANativeActivity = ndk_glue::native_activity().ptr().as_ptr();
            unsafe {
                match (*(*a_native_activity).callbacks).onNativeWindowRedrawNeeded {
                    Some(callback) => callback(a_native_activity, a_native_window),
                    None => (),
                };
            };
        }
        None => (),
    }
}

#[cfg(target_os = "android")]
fn get_insets() -> GameResult<(f32, f32, f32, f32)> {
    unsafe {
        use jni::objects::JObject;
        use jni::JavaVM;

        let vm_ptr = ndk_glue::native_activity().vm();
        let vm = JavaVM::from_raw(vm_ptr)?;
        let vm_env = vm.attach_current_thread()?;

        let class = vm_env.new_global_ref(JObject::from_raw(ndk_glue::native_activity().activity()))?;
        let field = vm_env.get_field(class.as_obj(), "displayInsets", "[I")?.to_jni().l as jni::sys::jintArray;

        let mut elements = [0; 4];
        vm_env.get_int_array_region(field, 0, &mut elements)?;

        vm_env.delete_local_ref(JObject::from_raw(field));

        //Game always runs with horizontal orientation so top and bottom cutouts not needed and only wastes piece of the screen
        elements[1] = 0;
        elements[3] = 0;

        Ok((elements[0] as f32, elements[1] as f32, elements[2] as f32, elements[3] as f32))
    }
}

// Android gamepad support via JNI
#[cfg(target_os = "android")]
const GAMEPAD_DATA_SIZE: usize = 8;
#[cfg(target_os = "android")]
const MAX_GAMEPADS: usize = 4;

#[cfg(target_os = "android")]
pub struct AndroidGamepad {
    device_id: u32,
}

#[cfg(target_os = "android")]
impl BackendGamepad for AndroidGamepad {
    fn set_rumble(&mut self, _low_freq: u16, _high_freq: u16, _duration_ms: u32) -> GameResult {
        // Android rumble requires API 31+ and is complex to implement via JNI
        // For now, rumble is not supported
        Ok(())
    }

    fn instance_id(&self) -> u32 {
        self.device_id
    }
}

#[cfg(target_os = "android")]
struct AndroidGamepadState {
    device_id: i32,
    buttons: i32,
    left_x: i32,
    left_y: i32,
    right_x: i32,
    right_y: i32,
    trigger_left: i32,
    trigger_right: i32,
}

#[cfg(target_os = "android")]
fn get_gamepad_data() -> GameResult<(i32, Vec<AndroidGamepadState>)> {
    unsafe {
        use jni::objects::JObject;
        use jni::JavaVM;

        let vm_ptr = ndk_glue::native_activity().vm();
        let vm = JavaVM::from_raw(vm_ptr)?;
        let vm_env = vm.attach_current_thread()?;

        let class = vm_env.new_global_ref(JObject::from_raw(ndk_glue::native_activity().activity()))?;

        // Get gamepad count
        let count_field = vm_env.get_field(class.as_obj(), "gamepadCount", "I")?;
        let gamepad_count = count_field.i().unwrap_or(0);

        if gamepad_count == 0 {
            return Ok((0, Vec::new()));
        }

        // Get gamepad data array
        let data_field = vm_env.get_field(class.as_obj(), "gamepadData", "[I")?.to_jni().l as jni::sys::jintArray;

        let total_elements = (gamepad_count as usize) * GAMEPAD_DATA_SIZE;
        let mut elements = vec![0i32; MAX_GAMEPADS * GAMEPAD_DATA_SIZE];
        vm_env.get_int_array_region(data_field, 0, &mut elements[..total_elements])?;

        vm_env.delete_local_ref(JObject::from_raw(data_field));

        let mut gamepads = Vec::with_capacity(gamepad_count as usize);
        for i in 0..gamepad_count as usize {
            let base = i * GAMEPAD_DATA_SIZE;
            gamepads.push(AndroidGamepadState {
                device_id: elements[base],
                buttons: elements[base + 1],
                left_x: elements[base + 2],
                left_y: elements[base + 3],
                right_x: elements[base + 4],
                right_y: elements[base + 5],
                trigger_left: elements[base + 6],
                trigger_right: elements[base + 7],
            });
        }

        Ok((gamepad_count, gamepads))
    }
}

#[cfg(target_os = "android")]
fn update_android_gamepads(ctx: &mut Context) {
    use crate::framework::gamepad;

    match get_gamepad_data() {
        Ok((_count, gamepads)) => {
            let existing_gamepads: Vec<u32> = ctx.gamepad_context.get_gamepads()
                .iter()
                .map(|g| g.instance_id())
                .collect();

            // Track which device IDs we see
            let mut seen_ids: Vec<u32> = Vec::new();

            for gp_state in &gamepads {
                let device_id = gp_state.device_id as u32;
                seen_ids.push(device_id);

                // Add new gamepads
                if !existing_gamepads.contains(&device_id) {
                    let android_gamepad = Box::new(AndroidGamepad { device_id });
                    gamepad::add_gamepad(ctx, android_gamepad, 0.3);
                    log::info!("Android gamepad connected: device_id={}", device_id);
                }

                // NOTE: Button states are NOT read from Java because NativeActivity
                // routes gamepad button events directly to native code, bypassing Java.
                // Buttons are handled via winit KeyboardInput events with Linux scancodes
                // in the event loop's keyboard input handler.

                // Update axis values only (convert from -32767..32767 to -1.0..1.0)
                ctx.gamepad_context.set_axis_value(device_id, Axis::LeftX, gp_state.left_x as f64 / 32767.0);
                ctx.gamepad_context.set_axis_value(device_id, Axis::LeftY, gp_state.left_y as f64 / 32767.0);
                ctx.gamepad_context.set_axis_value(device_id, Axis::RightX, gp_state.right_x as f64 / 32767.0);
                ctx.gamepad_context.set_axis_value(device_id, Axis::RightY, gp_state.right_y as f64 / 32767.0);
                ctx.gamepad_context.set_axis_value(device_id, Axis::TriggerLeft, gp_state.trigger_left as f64 / 32767.0);
                ctx.gamepad_context.set_axis_value(device_id, Axis::TriggerRight, gp_state.trigger_right as f64 / 32767.0);

                // Update axes in gamepad data
                ctx.gamepad_context.update_axes(device_id);
            }

            // Remove disconnected gamepads
            for existing_id in existing_gamepads {
                if !seen_ids.contains(&existing_id) {
                    gamepad::remove_gamepad(ctx, existing_id);
                    log::info!("Android gamepad disconnected: device_id={}", existing_id);
                }
            }
        }
        Err(_e) => {
            // Silently ignore errors - gamepad data may not be available yet
        }
    }
}

// Convert Linux input scancode to gamepad button
// Scancodes are Linux BTN_* values from input-event-codes.h
#[cfg(target_os = "android")]
fn android_scancode_to_button(scancode: u32) -> Option<Button> {
    match scancode {
        // Face buttons (BTN_SOUTH, BTN_EAST, BTN_NORTH, BTN_WEST)
        304 => Some(Button::South),      // BTN_SOUTH (BTN_A) = 0x130
        305 => Some(Button::East),       // BTN_EAST (BTN_B) = 0x131
        307 => Some(Button::North),      // BTN_NORTH (BTN_X) = 0x133
        308 => Some(Button::West),       // BTN_WEST (BTN_Y) = 0x134
        // Shoulder buttons
        310 => Some(Button::LeftShoulder),   // BTN_TL = 0x136
        311 => Some(Button::RightShoulder),  // BTN_TR = 0x137
        // Triggers (as buttons)
        312 => Some(Button::LeftShoulder),   // BTN_TL2 = 0x138 (fallback)
        313 => Some(Button::RightShoulder),  // BTN_TR2 = 0x139 (fallback)
        // Menu buttons
        314 => Some(Button::Back),       // BTN_SELECT = 0x13a
        315 => Some(Button::Start),      // BTN_START = 0x13b
        316 => Some(Button::Guide),      // BTN_MODE = 0x13c
        // Thumbstick clicks
        317 => Some(Button::LeftStick),  // BTN_THUMBL = 0x13d
        318 => Some(Button::RightStick), // BTN_THUMBR = 0x13e
        // D-pad
        544 => Some(Button::DPadUp),     // BTN_DPAD_UP = 0x220
        545 => Some(Button::DPadDown),   // BTN_DPAD_DOWN = 0x221
        546 => Some(Button::DPadLeft),   // BTN_DPAD_LEFT = 0x222
        547 => Some(Button::DPadRight),  // BTN_DPAD_RIGHT = 0x223
        _ => None,
    }
}

fn get_scaled_size(width: u32, height: u32) -> (f32, f32) {
    let scaled_height = ((height / 480).max(1) * 480) as f32;
    let scaled_width = (width as f32 * (scaled_height as f32 / height as f32)).floor();

    (scaled_width, scaled_height)
}

impl BackendEventLoop for GlutinEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context) {
        let event_loop = EventLoop::new();
        let state_ref = unsafe { &mut *game.state.get() };
        let window: &'static mut WindowedContext<PossiblyCurrent> =
            unsafe { std::mem::transmute(self.get_context(&ctx, &event_loop)) };        
        {
            let size = window.window().inner_size();
            ctx.real_screen_size = (size.width, size.height);
            ctx.screen_size = get_scaled_size(size.width.max(1), size.height.max(1));
            state_ref.handle_resize(ctx).unwrap();
        }

        // it won't ever return
        let (game, ctx): (&'static mut Game, &'static mut Context) =
            unsafe { (std::mem::transmute(game), std::mem::transmute(ctx)) };

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, window_id }
                    if window_id == window.window().id() =>
                {
                    state_ref.shutdown();
                }
                Event::Resumed => {
                    {
                        let mut mutex = GAME_SUSPENDED.lock().unwrap();
                        *mutex = false;
                    }

                    #[cfg(target_os = "android")]
                    if let Some(nwin) = ndk_glue::native_window().as_ref() {
                        state_ref.graphics_reset();
                        unsafe {
                            window.surface_created(nwin.ptr().as_ptr() as *mut std::ffi::c_void);
                            request_android_redraw();
                        }
                    }

                    state_ref.sound_manager.resume();
                }
                Event::Suspended => {
                    {
                        let mut mutex = GAME_SUSPENDED.lock().unwrap();
                        *mutex = true;
                    }

                    #[cfg(target_os = "android")]
                    unsafe {
                        window.surface_destroyed();
                    }

                    state_ref.sound_manager.pause();
                }
                Event::WindowEvent { event: WindowEvent::Resized(size), window_id }
                    if window_id == window.window().id() =>
                {
                    if let Some(renderer) = &ctx.renderer {
                        if let Ok(imgui) = renderer.imgui() {
                            imgui.io_mut().display_size = [size.width as f32, size.height as f32];
                        }

                        ctx.real_screen_size = (size.width, size.height);
                        ctx.screen_size = get_scaled_size(size.width.max(1), size.height.max(1));
                        state_ref.handle_resize(ctx).unwrap();
                    }
                }
                Event::WindowEvent { event: WindowEvent::Touch(touch), window_id }
                    if window_id == window.window().id() =>
                {
                    let mut controls = &mut state_ref.touch_controls;
                    let scale = state_ref.scale as f64;
                    let loc_x = (touch.location.x * ctx.screen_size.0 as f64 / ctx.real_screen_size.0 as f64) / scale;
                    let loc_y = (touch.location.y * ctx.screen_size.1 as f64 / ctx.real_screen_size.1 as f64) / scale;

                    match touch.phase {
                        TouchPhase::Started | TouchPhase::Moved => {
                            if let Some(point) = controls.points.iter_mut().find(|p| p.id == touch.id) {
                                point.last_position = point.position;
                                point.position = (loc_x, loc_y);
                            } else {
                                controls.touch_id_counter = controls.touch_id_counter.wrapping_add(1);

                                let point = TouchPoint {
                                    id: touch.id,
                                    touch_id: controls.touch_id_counter,
                                    position: (loc_x, loc_y),
                                    last_position: (0.0, 0.0),
                                };
                                controls.points.push(point);

                                if touch.phase == TouchPhase::Started {
                                    controls.clicks.push(point);
                                }
                            }
                        }
                        TouchPhase::Ended | TouchPhase::Cancelled => {
                            controls.points.retain(|p| p.id != touch.id);
                            controls.clicks.retain(|p| p.id != touch.id);
                        }
                    }
                }
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, window_id }
                    if window_id == window.window().id() =>
                {
                    let key_state = match input.state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    // On Android, handle gamepad button scancodes directly
                    // since winit doesn't convert them to virtual keycodes
                    #[cfg(target_os = "android")]
                    {
                        if let Some(button) = android_scancode_to_button(input.scancode) {
                            // Collect gamepad IDs first to avoid borrow conflict
                            let gamepad_ids: Vec<u32> = ctx.gamepad_context.get_gamepads()
                                .iter()
                                .map(|gp| gp.instance_id())
                                .collect();
                            // Apply button state to all connected gamepads
                            for device_id in gamepad_ids {
                                ctx.gamepad_context.set_button(device_id, button, key_state);
                            }
                        }
                    }

                    if let Some(keycode) = input.virtual_keycode {
                        if let Some(drs_scan) = conv_keycode(keycode) {
                            ctx.keyboard_context.set_key(drs_scan, key_state);
                        }
                    }
                }
                Event::RedrawRequested(id) if id == window.window().id() => {
                    {
                        let mutex = GAME_SUSPENDED.lock().unwrap();
                        if *mutex {
                            return;
                        }
                    }

                    #[cfg(not(target_os = "android"))]
                    {
                        if let Err(err) = game.draw(ctx) {
                            log::error!("Failed to draw frame: {}", err);
                        }

                        window.window().request_redraw();
                    }

                    #[cfg(target_os = "android")]
                    request_android_redraw();
                }
                Event::MainEventsCleared => {
                    if state_ref.shutdown {
                        log::info!("Shutting down...");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    {
                        let mutex = GAME_SUSPENDED.lock().unwrap();
                        if *mutex {
                            return;
                        }
                    }

                    #[cfg(target_os = "android")]
                    update_android_gamepads(ctx);

                    #[cfg(not(any(target_os = "android", target_os = "horizon")))]
                    {
                        if ctx.window.mode.get_glutin_fullscreen_type() != window.window().fullscreen() {
                            let fullscreen_type = ctx.window.mode.get_glutin_fullscreen_type();
                            let cursor_visible = ctx.window.mode.should_display_mouse_cursor();

                            window.window().set_fullscreen(fullscreen_type);
                            window.window().set_cursor_visible(cursor_visible);
                        }
                    }

                    game.update(ctx).unwrap();

                    #[cfg(target_os = "android")]
                    {
                        match get_insets() {
                            Ok(insets) => {
                                ctx.screen_insets = insets;
                            }
                            Err(e) => {
                                log::error!("Failed to update insets: {}", e);
                            }
                        }

                        if let Err(err) = game.draw(ctx) {
                            log::error!("Failed to draw frame: {}", err);
                        }
                    }

                    if state_ref.next_scene.is_some() {
                        mem::swap(&mut game.scene, &mut state_ref.next_scene);
                        state_ref.next_scene = None;
                        game.scene.as_mut().unwrap().init(state_ref, ctx).unwrap();
                        game.loops = 0;
                        state_ref.frame_time = 0.0;
                    }
                }
                _ => (),
            }
        });
    }

    fn new_renderer(&self, ctx: *mut Context) -> GameResult<Box<dyn BackendRenderer>> {
        let mut imgui = imgui::Context::create();
        imgui.io_mut().display_size = [640.0, 480.0];

        let refs = self.refs.clone();
        let user_data = Rc::into_raw(refs) as *mut c_void;

        unsafe fn get_proc_address(user_data: &mut *mut c_void, name: &str) -> *const c_void {
            let refs = Rc::from_raw(*user_data as *mut UnsafeCell<Option<WindowedContext<PossiblyCurrent>>>);

            let result = {
                let refs = &mut *refs.get();

                if let Some(refs) = refs {
                    refs.get_proc_address(name)
                } else {
                    std::ptr::null()
                }
            };

            *user_data = Rc::into_raw(refs) as *mut c_void;

            result
        }

        unsafe fn swap_buffers(user_data: &mut *mut c_void) {
            let refs = Rc::from_raw(*user_data as *mut UnsafeCell<Option<WindowedContext<PossiblyCurrent>>>);

            {
                let refs = &mut *refs.get();

                if let Some(refs) = refs {
                    refs.swap_buffers();
                }
            }

            *user_data = Rc::into_raw(refs) as *mut c_void;
        }

        let gl_context = GLContext { gles2_mode: true, is_sdl: false, get_proc_address, swap_buffers, user_data, ctx };

        Ok(Box::new(OpenGLRenderer::new(gl_context, UnsafeCell::new(imgui))))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn conv_keycode(code: VirtualKeyCode) -> Option<ScanCode> {
    match code {
        VirtualKeyCode::Key1 => Some(ScanCode::Key1),
        VirtualKeyCode::Key2 => Some(ScanCode::Key2),
        VirtualKeyCode::Key3 => Some(ScanCode::Key3),
        VirtualKeyCode::Key4 => Some(ScanCode::Key4),
        VirtualKeyCode::Key5 => Some(ScanCode::Key5),
        VirtualKeyCode::Key6 => Some(ScanCode::Key6),
        VirtualKeyCode::Key7 => Some(ScanCode::Key7),
        VirtualKeyCode::Key8 => Some(ScanCode::Key8),
        VirtualKeyCode::Key9 => Some(ScanCode::Key9),
        VirtualKeyCode::Key0 => Some(ScanCode::Key0),
        VirtualKeyCode::A => Some(ScanCode::A),
        VirtualKeyCode::B => Some(ScanCode::B),
        VirtualKeyCode::C => Some(ScanCode::C),
        VirtualKeyCode::D => Some(ScanCode::D),
        VirtualKeyCode::E => Some(ScanCode::E),
        VirtualKeyCode::F => Some(ScanCode::F),
        VirtualKeyCode::G => Some(ScanCode::G),
        VirtualKeyCode::H => Some(ScanCode::H),
        VirtualKeyCode::I => Some(ScanCode::I),
        VirtualKeyCode::J => Some(ScanCode::J),
        VirtualKeyCode::K => Some(ScanCode::K),
        VirtualKeyCode::L => Some(ScanCode::L),
        VirtualKeyCode::M => Some(ScanCode::M),
        VirtualKeyCode::N => Some(ScanCode::N),
        VirtualKeyCode::O => Some(ScanCode::O),
        VirtualKeyCode::P => Some(ScanCode::P),
        VirtualKeyCode::Q => Some(ScanCode::Q),
        VirtualKeyCode::R => Some(ScanCode::R),
        VirtualKeyCode::S => Some(ScanCode::S),
        VirtualKeyCode::T => Some(ScanCode::T),
        VirtualKeyCode::U => Some(ScanCode::U),
        VirtualKeyCode::V => Some(ScanCode::V),
        VirtualKeyCode::W => Some(ScanCode::W),
        VirtualKeyCode::X => Some(ScanCode::X),
        VirtualKeyCode::Y => Some(ScanCode::Y),
        VirtualKeyCode::Z => Some(ScanCode::Z),
        VirtualKeyCode::Escape => Some(ScanCode::Escape),
        VirtualKeyCode::F1 => Some(ScanCode::F1),
        VirtualKeyCode::F2 => Some(ScanCode::F2),
        VirtualKeyCode::F3 => Some(ScanCode::F3),
        VirtualKeyCode::F4 => Some(ScanCode::F4),
        VirtualKeyCode::F5 => Some(ScanCode::F5),
        VirtualKeyCode::F6 => Some(ScanCode::F6),
        VirtualKeyCode::F7 => Some(ScanCode::F7),
        VirtualKeyCode::F8 => Some(ScanCode::F8),
        VirtualKeyCode::F9 => Some(ScanCode::F9),
        VirtualKeyCode::F10 => Some(ScanCode::F10),
        VirtualKeyCode::F11 => Some(ScanCode::F11),
        VirtualKeyCode::F12 => Some(ScanCode::F12),
        VirtualKeyCode::F13 => Some(ScanCode::F13),
        VirtualKeyCode::F14 => Some(ScanCode::F14),
        VirtualKeyCode::F15 => Some(ScanCode::F15),
        VirtualKeyCode::F16 => Some(ScanCode::F16),
        VirtualKeyCode::F17 => Some(ScanCode::F17),
        VirtualKeyCode::F18 => Some(ScanCode::F18),
        VirtualKeyCode::F19 => Some(ScanCode::F19),
        VirtualKeyCode::F20 => Some(ScanCode::F20),
        VirtualKeyCode::F21 => Some(ScanCode::F21),
        VirtualKeyCode::F22 => Some(ScanCode::F22),
        VirtualKeyCode::F23 => Some(ScanCode::F23),
        VirtualKeyCode::F24 => Some(ScanCode::F24),
        VirtualKeyCode::Snapshot => Some(ScanCode::Snapshot),
        VirtualKeyCode::Scroll => Some(ScanCode::Scroll),
        VirtualKeyCode::Pause => Some(ScanCode::Pause),
        VirtualKeyCode::Insert => Some(ScanCode::Insert),
        VirtualKeyCode::Home => Some(ScanCode::Home),
        VirtualKeyCode::Delete => Some(ScanCode::Delete),
        VirtualKeyCode::End => Some(ScanCode::End),
        VirtualKeyCode::PageDown => Some(ScanCode::PageDown),
        VirtualKeyCode::PageUp => Some(ScanCode::PageUp),
        VirtualKeyCode::Left => Some(ScanCode::Left),
        VirtualKeyCode::Up => Some(ScanCode::Up),
        VirtualKeyCode::Right => Some(ScanCode::Right),
        VirtualKeyCode::Down => Some(ScanCode::Down),
        VirtualKeyCode::Back => Some(ScanCode::Back),
        VirtualKeyCode::Return => Some(ScanCode::Return),
        VirtualKeyCode::Space => Some(ScanCode::Space),
        VirtualKeyCode::Compose => Some(ScanCode::Compose),
        VirtualKeyCode::Caret => Some(ScanCode::Caret),
        VirtualKeyCode::Numlock => Some(ScanCode::Numlock),
        VirtualKeyCode::Numpad0 => Some(ScanCode::Numpad0),
        VirtualKeyCode::Numpad1 => Some(ScanCode::Numpad1),
        VirtualKeyCode::Numpad2 => Some(ScanCode::Numpad2),
        VirtualKeyCode::Numpad3 => Some(ScanCode::Numpad3),
        VirtualKeyCode::Numpad4 => Some(ScanCode::Numpad4),
        VirtualKeyCode::Numpad5 => Some(ScanCode::Numpad5),
        VirtualKeyCode::Numpad6 => Some(ScanCode::Numpad6),
        VirtualKeyCode::Numpad7 => Some(ScanCode::Numpad7),
        VirtualKeyCode::Numpad8 => Some(ScanCode::Numpad8),
        VirtualKeyCode::Numpad9 => Some(ScanCode::Numpad9),
        VirtualKeyCode::NumpadAdd => Some(ScanCode::NumpadAdd),
        VirtualKeyCode::NumpadDivide => Some(ScanCode::NumpadDivide),
        VirtualKeyCode::NumpadDecimal => Some(ScanCode::NumpadDecimal),
        VirtualKeyCode::NumpadComma => Some(ScanCode::NumpadComma),
        VirtualKeyCode::NumpadEnter => Some(ScanCode::NumpadEnter),
        VirtualKeyCode::NumpadEquals => Some(ScanCode::NumpadEquals),
        VirtualKeyCode::NumpadMultiply => Some(ScanCode::NumpadMultiply),
        VirtualKeyCode::NumpadSubtract => Some(ScanCode::NumpadSubtract),
        VirtualKeyCode::AbntC1 => Some(ScanCode::AbntC1),
        VirtualKeyCode::AbntC2 => Some(ScanCode::AbntC2),
        VirtualKeyCode::Apostrophe => Some(ScanCode::Apostrophe),
        VirtualKeyCode::Apps => Some(ScanCode::Apps),
        VirtualKeyCode::Asterisk => Some(ScanCode::Asterisk),
        VirtualKeyCode::At => Some(ScanCode::At),
        VirtualKeyCode::Ax => Some(ScanCode::Ax),
        VirtualKeyCode::Backslash => Some(ScanCode::Backslash),
        VirtualKeyCode::Calculator => Some(ScanCode::Calculator),
        VirtualKeyCode::Capital => Some(ScanCode::Capital),
        VirtualKeyCode::Colon => Some(ScanCode::Colon),
        VirtualKeyCode::Comma => Some(ScanCode::Comma),
        VirtualKeyCode::Convert => Some(ScanCode::Convert),
        VirtualKeyCode::Equals => Some(ScanCode::Equals),
        VirtualKeyCode::Grave => Some(ScanCode::Grave),
        VirtualKeyCode::Kana => Some(ScanCode::Kana),
        VirtualKeyCode::Kanji => Some(ScanCode::Kanji),
        VirtualKeyCode::LAlt => Some(ScanCode::LAlt),
        VirtualKeyCode::LBracket => Some(ScanCode::LBracket),
        VirtualKeyCode::LControl => Some(ScanCode::LControl),
        VirtualKeyCode::LShift => Some(ScanCode::LShift),
        VirtualKeyCode::LWin => Some(ScanCode::LWin),
        VirtualKeyCode::Mail => Some(ScanCode::Mail),
        VirtualKeyCode::MediaSelect => Some(ScanCode::MediaSelect),
        VirtualKeyCode::MediaStop => Some(ScanCode::MediaStop),
        VirtualKeyCode::Minus => Some(ScanCode::Minus),
        VirtualKeyCode::Mute => Some(ScanCode::Mute),
        VirtualKeyCode::MyComputer => Some(ScanCode::MyComputer),
        VirtualKeyCode::NavigateForward => Some(ScanCode::NavigateForward),
        VirtualKeyCode::NavigateBackward => Some(ScanCode::NavigateBackward),
        VirtualKeyCode::NextTrack => Some(ScanCode::NextTrack),
        VirtualKeyCode::NoConvert => Some(ScanCode::NoConvert),
        VirtualKeyCode::OEM102 => Some(ScanCode::OEM102),
        VirtualKeyCode::Period => Some(ScanCode::Period),
        VirtualKeyCode::PlayPause => Some(ScanCode::PlayPause),
        VirtualKeyCode::Plus => Some(ScanCode::Plus),
        VirtualKeyCode::Power => Some(ScanCode::Power),
        VirtualKeyCode::PrevTrack => Some(ScanCode::PrevTrack),
        VirtualKeyCode::RAlt => Some(ScanCode::RAlt),
        VirtualKeyCode::RBracket => Some(ScanCode::RBracket),
        VirtualKeyCode::RControl => Some(ScanCode::RControl),
        VirtualKeyCode::RShift => Some(ScanCode::RShift),
        VirtualKeyCode::RWin => Some(ScanCode::RWin),
        VirtualKeyCode::Semicolon => Some(ScanCode::Semicolon),
        VirtualKeyCode::Slash => Some(ScanCode::Slash),
        VirtualKeyCode::Sleep => Some(ScanCode::Sleep),
        VirtualKeyCode::Stop => Some(ScanCode::Stop),
        VirtualKeyCode::Sysrq => Some(ScanCode::Sysrq),
        VirtualKeyCode::Tab => Some(ScanCode::Tab),
        VirtualKeyCode::Underline => Some(ScanCode::Underline),
        VirtualKeyCode::Unlabeled => Some(ScanCode::Unlabeled),
        VirtualKeyCode::VolumeDown => Some(ScanCode::VolumeDown),
        VirtualKeyCode::VolumeUp => Some(ScanCode::VolumeUp),
        VirtualKeyCode::Wake => Some(ScanCode::Wake),
        VirtualKeyCode::WebBack => Some(ScanCode::WebBack),
        VirtualKeyCode::WebFavorites => Some(ScanCode::WebFavorites),
        VirtualKeyCode::WebForward => Some(ScanCode::WebForward),
        VirtualKeyCode::WebHome => Some(ScanCode::WebHome),
        VirtualKeyCode::WebRefresh => Some(ScanCode::WebRefresh),
        VirtualKeyCode::WebSearch => Some(ScanCode::WebSearch),
        VirtualKeyCode::WebStop => Some(ScanCode::WebStop),
        VirtualKeyCode::Yen => Some(ScanCode::Yen),
        VirtualKeyCode::Copy => Some(ScanCode::Copy),
        VirtualKeyCode::Paste => Some(ScanCode::Paste),
        VirtualKeyCode::Cut => Some(ScanCode::Cut),
    }
}
