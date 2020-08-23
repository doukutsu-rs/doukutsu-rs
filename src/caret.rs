use crate::bitfield;
use crate::common::{Direction, Rect};
use crate::engine_constants::EngineConstants;
use crate::rng::RNG;

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
    LittleParticles,
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

    pub fn tick(&mut self, rng: &RNG, constants: &EngineConstants) {
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
                    Direction::Left => { self.x -= 0x400; } // 2.0fix9
                    Direction::Up => { self.y -= 0x400; }
                    Direction::Right => { self.x += 0x400; }
                    Direction::Bottom => { self.y += 0x400; }
                }
            }
            CaretType::DrownedQuote => {}
            CaretType::QuestionMark => {
                self.anim_wait += 1;
                if self.anim_wait < 5 {
                    self.y -= 0x800; // 4.0fix9
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
            CaretType::LittleParticles => {
                if self.anim_num == 0 {
                    match self.direct {
                        Direction::Left => {
                            self.vel_x = rng.range(-0x300..0x300) as isize; // -1.5fix9..1.5fix9
                            self.vel_y = rng.range(-0x100..0x100) as isize; // -0.5fix9..0.5fix9
                        }
                        Direction::Up => {
                            self.vel_y = rng.range(1..3) as isize * 0x100;
                        }
                        _ => {}
                    }
                }

                self.anim_num += 1;

                if self.direct == Direction::Left {
                    self.vel_x = (self.vel_x * 4) / 5;
                    self.vel_y = (self.vel_y * 4) / 5;
                }

                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.anim_num == 20 {
                    self.cond.set_visible(false);
                    return;
                }

                self.anim_rect = constants.caret.little_particles_rects[self.anim_num / 2 % constants.caret.little_particles_rects.len()];

                if self.direct == Direction::Right {
                    self.x -= 4 * 0x200;
                }
            }
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
