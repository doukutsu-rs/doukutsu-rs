use lazy_static::lazy_static;


use crate::common::{Color, Direction, Rect};
use crate::framework::context::Context;
use crate::framework::filesystem;
use crate::framework::filesystem::File;
use crate::player::skin::{PlayerAnimationState, PlayerAppearanceState, PlayerSkin};
use crate::shared_game_state::SharedGameState;

#[derive(Default, Clone, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinMeta {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default = "skinmeta_ret0i")]
    pub gun_offset_x: i16,
    #[serde(default = "skinmeta_ret0i")]
    pub gun_offset_y: i16,
    #[serde(default = "skinmeta_ret16")]
    pub frame_size_width: u16,
    #[serde(default = "skinmeta_ret16")]
    pub frame_size_height: u16,
    #[serde(default = "skinmeta_default_hit_box")]
    pub hit_box: Rect<u16>,
    #[serde(default = "skinmeta_default_display_box")]
    pub display_box: Rect<u16>,
    #[serde(default)]
    pub version: u8,
}

const fn skinmeta_ret0i() -> i16 {
    0
}

const fn skinmeta_ret16() -> u16 {
    16
}

const fn skinmeta_default_hit_box() -> Rect<u16> {
    Rect { left: 5, top: 8, right: 5, bottom: 8 }
}

const fn skinmeta_default_display_box() -> Rect<u16> {
    Rect { left: 8, top: 8, right: 8, bottom: 8 }
}

pub static SUPPORTED_SKINMETA_VERSIONS: [u8; 1] = [1];

lazy_static! {
    pub static ref DEFAULT_SKINMETA: SkinMeta = SkinMeta {
        name: "Player".to_string(),
        description: "".to_string(),
        author: "".to_string(),
        gun_offset_x: 0,
        gun_offset_y: 0,
        frame_size_width: 16,
        frame_size_height: 16,
        hit_box: skinmeta_default_hit_box(),
        display_box: skinmeta_default_display_box(),
        version: 1
    };
}

#[derive(Clone)]
pub struct BasicPlayerSkin {
    texture_name: String,
    color: Color,
    state: PlayerAnimationState,
    appearance: PlayerAppearanceState,
    direction: Direction,
    metadata: SkinMeta,
    tick: u16,
}

impl BasicPlayerSkin {
    pub fn new(texture_name: String, state: &SharedGameState, ctx: &mut Context) -> BasicPlayerSkin {
        let metapath = format!("{}/{}.dskinmeta", state.base_path, texture_name);
        let mut metadata = DEFAULT_SKINMETA.clone();

        if let Ok(file) = filesystem::open(ctx, metapath) {
            match serde_json::from_reader::<File, SkinMeta>(file) {
                Ok(meta) if SUPPORTED_SKINMETA_VERSIONS.contains(&meta.version) => {
                    metadata = meta;
                }
                Ok(meta) => {
                    log::warn!("Unsupported skin metadata file version: {}", meta.version);
                }
                Err(err) => {
                    log::warn!("Failed to load skin metadata file: {:?}", err);
                }
            }
        }

        BasicPlayerSkin {
            texture_name,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            state: PlayerAnimationState::Idle,
            appearance: PlayerAppearanceState::Default,
            direction: Direction::Left,
            metadata,
            tick: 0,
        }
    }
}

impl PlayerSkin for BasicPlayerSkin {
    fn animation_frame_for(&self, state: PlayerAnimationState, direction: Direction, tick: u16) -> Rect<u16> {
        let frame_id = match self.state {
            PlayerAnimationState::Idle => 0u16,
            PlayerAnimationState::Walking => {
                const WALK_INDEXES: [u16; 4] = [1, 0, 2, 0];

                WALK_INDEXES[(tick as usize / 5) % 4]
            }
            PlayerAnimationState::WalkingUp => {
                const WALK_UP_INDEXES: [u16; 4] = [4, 3, 5, 3];

                WALK_UP_INDEXES[(tick as usize / 5) % 4]
            }
            PlayerAnimationState::LookingUp => 3,
            PlayerAnimationState::Examining => 7,
            PlayerAnimationState::Sitting => 8,
            PlayerAnimationState::Collapsed => 9,
            PlayerAnimationState::Jumping => 2,
            PlayerAnimationState::Falling => 1,
            PlayerAnimationState::FallingLookingUp => 4,
            PlayerAnimationState::FallingLookingDown => 6,
            PlayerAnimationState::FallingUpsideDown => 10,
        };

        let y_offset = if direction == Direction::Left { 0 } else { self.metadata.frame_size_height }
            + match self.appearance {
                PlayerAppearanceState::Default => 0,
                PlayerAppearanceState::MimigaMask => self.metadata.frame_size_height.saturating_mul(2),
                PlayerAppearanceState::Custom(i) => (i as u16).saturating_mul(self.metadata.frame_size_height),
            };

        Rect::new_size(
            frame_id.saturating_mul(self.metadata.frame_size_width),
            y_offset,
            self.metadata.frame_size_width,
            self.metadata.frame_size_height,
        )
    }

    fn animation_frame(&self) -> Rect<u16> {
        self.animation_frame_for(self.state, self.direction, self.tick)
    }

    fn tick(&mut self) {
        if self.state == PlayerAnimationState::Walking || self.state == PlayerAnimationState::WalkingUp {
            self.tick = self.tick.wrapping_add(1);
        }
    }

    fn set_state(&mut self, state: PlayerAnimationState) {
        if self.state != state {
            self.state = state;
            self.tick = 0;
        }
    }

    fn get_state(&self) -> PlayerAnimationState {
        self.state
    }

    fn set_appearance(&mut self, appearance: PlayerAppearanceState) {
        self.appearance = appearance;
    }

    fn get_appearance(&mut self) -> PlayerAppearanceState {
        self.appearance
    }

    fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    fn get_color(&self) -> Color {
        self.color
    }

    fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    fn get_direction(&self) -> Direction {
        self.direction
    }

    fn get_skin_texture_name(&self) -> &str {
        self.texture_name.as_str()
    }

    fn get_mask_texture_name(&self) -> &str {
        ""
    }

    fn get_hit_bounds(&self) -> Rect<u32> {
        let ubox = &self.metadata.hit_box;

        Rect {
            left: ubox.left as u32 * 0x200,
            top: ubox.top as u32 * 0x200,
            right: ubox.right as u32 * 0x200,
            bottom: ubox.bottom as u32 * 0x200,
        }
    }

    fn get_display_bounds(&self) -> Rect<u32> {
        let ubox = &self.metadata.display_box;

        Rect {
            left: ubox.left as u32 * 0x200,
            top: ubox.top as u32 * 0x200,
            right: ubox.right as u32 * 0x200,
            bottom: ubox.bottom as u32 * 0x200,
        }
    }

    fn get_gun_offset(&self) -> (i32, i32) {
        (self.metadata.gun_offset_x as i32, self.metadata.gun_offset_y as i32)
    }
}
