use crate::bitfield;
use crate::common::{Direction, Rect};
use crate::engine_constants::EngineConstants;

bitfield! {
  pub struct Cond(u16);
  impl Debug;

  pub visible, set_visible: 7;
}

#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Copy, Clone)]
pub enum CaretType {
    None,
    Bubble,
    ProjectileDissipation,
    Shoot,
    SnakeAfterimage,
    Zzz,
    SnakeAfterimage2,
    Exhaust,
    DrownedQuote,
    QuestionMark,
    LevelUp,
    HurtParticles,
    Explosion,
    SmallParticles,
    Unknown,
    SmallProjectileDissipation,
    Empty,
    PushJumpKey,
}

pub struct Caret {
    pub ctype: CaretType,
    pub x: isize,
    pub y: isize,
    pub vel_x: isize,
    pub vel_y: isize,
    pub offset_x: isize,
    pub offset_y: isize,
    pub cond: Cond,
    pub direct: Direction,
    pub anim_rect: Rect<usize>,
    anim_num: usize,
    anim_wait: isize,
}

impl Caret {
    pub fn new(x: isize, y: isize, ctype: CaretType, direct: Direction, constants: &EngineConstants) -> Self {
        let (offset_x, offset_y) = constants.caret.offsets[ctype as usize];
        Self {
            ctype,
            x,
            y,
            vel_x: 0,
            vel_y: 0,
            offset_x,
            offset_y,
            cond: Cond(0x80),
            direct,
            anim_rect: Rect::<usize>::new(0, 0, 0, 0),
            anim_num: 0,
            anim_wait: 0,
        }
    }

    pub fn tick(&mut self, constants: &EngineConstants) {
        match self.ctype {
            CaretType::None => {}
            CaretType::Bubble => {}
            CaretType::ProjectileDissipation => {}
            CaretType::Shoot => {}
            CaretType::SnakeAfterimage | CaretType::SnakeAfterimage2 => { // dupe, unused
            }
            CaretType::Zzz => {}
            CaretType::Exhaust => {
                self.anim_wait += 1;
                if self.anim_wait > 1 {
                    self.anim_wait = 0;
                    self.anim_num += 1;
                }

                if self.anim_num >= constants.caret.exhaust_rects.len() {
                    self.cond.set_visible(false);
                    return;
                }

                self.anim_rect = constants.caret.exhaust_rects[self.anim_num];

                match self.direct {
                    Direction::Left => { self.x -= 0x400; }
                    Direction::Up => { self.y -= 0x400; }
                    Direction::Right => { self.x += 0x400; }
                    Direction::Bottom => { self.y += 0x400; }
                }
            }
            CaretType::DrownedQuote => {}
            CaretType::QuestionMark => {
                self.anim_wait += 1;
                if self.anim_wait < 5 {
                    self.y -= 0x800;
                }

                if self.anim_wait == 32 {
                    self.cond.set_visible(false);
                }

                self.anim_rect = match self.direct {
                    Direction::Left => { constants.caret.question_left_rect }
                    Direction::Right => { constants.caret.question_right_rect }
                    _ => { self.anim_rect }
                }
            }
            CaretType::LevelUp => {}
            CaretType::HurtParticles => {}
            CaretType::Explosion => {}
            CaretType::SmallParticles => {}
            CaretType::Unknown => {
                // not implemented because it was apparently broken in og game?
                self.cond.set_visible(false);
            }
            CaretType::SmallProjectileDissipation => {}
            CaretType::Empty => {}
            CaretType::PushJumpKey => {}
        }
    }

    pub fn is_dead(&self) -> bool {
        !self.cond.visible()
    }
}
