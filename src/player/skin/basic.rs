use crate::common::{Color, Direction, Rect};
use crate::player::skin::{PlayerAnimationState, PlayerAppearanceState, PlayerSkin};

#[derive(Clone)]
pub struct BasicPlayerSkin {
    texture_name: String,
    color: Color,
    state: PlayerAnimationState,
    appearance: PlayerAppearanceState,
    direction: Direction,
    tick: u16,
}

impl BasicPlayerSkin {
    pub fn new(texture_name: String) -> BasicPlayerSkin {
        BasicPlayerSkin {
            texture_name,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            state: PlayerAnimationState::Idle,
            appearance: PlayerAppearanceState::Default,
            direction: Direction::Left,
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

        let y_offset = if direction == Direction::Left { 0 } else { 16 }
            + match self.appearance {
                PlayerAppearanceState::Default => 0,
                PlayerAppearanceState::MimigaMask => 32,
                PlayerAppearanceState::Custom(i) => (i as u16).saturating_mul(16),
            };

        Rect::new_size(frame_id * 16, y_offset, 16, 16)
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
}
