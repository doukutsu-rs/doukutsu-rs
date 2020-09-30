use std::fs::read_to_string;

use crate::bitfield;
use crate::common::{Condition, Direction, Rect};
use crate::engine_constants::EngineConstants;
use crate::rng::RNG;

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
    pub cond: Condition,
    pub direction: Direction,
    pub anim_rect: Rect<usize>,
    anim_num: u16,
    anim_counter: u16,
}

impl Caret {
    pub fn new(x: isize, y: isize, ctype: CaretType, direct: Direction, constants: &EngineConstants) -> Caret {
        let (offset_x, offset_y) = constants.caret.offsets[ctype as usize];

        Caret {
            ctype,
            x,
            y,
            vel_x: 0,
            vel_y: 0,
            offset_x,
            offset_y,
            cond: Condition(0x80),
            direction: direct,
            anim_rect: Rect::new(0, 0, 0, 0),
            anim_num: 0,
            anim_counter: 0,
        }
    }

    pub fn tick(&mut self, rng: &RNG, constants: &EngineConstants) {
        match self.ctype {
            CaretType::None => {}
            CaretType::Bubble => {}
            CaretType::ProjectileDissipation => {
                match self.direction {
                    Direction::Left => {
                        self.vel_y -= 0x10;
                        self.y += self.vel_y;

                        self.anim_counter += 1;
                        if self.anim_counter > 5 {
                            self.anim_counter = 0;
                            self.anim_num += 1;

                            if self.anim_num >= constants.caret.projectile_dissipation_left_rects.len() as u16 {
                                self.cond.set_alive(false);
                                return;
                            }

                            self.anim_rect = constants.caret.projectile_dissipation_left_rects[self.anim_num as usize];
                        }
                    }
                    Direction::Up => {
                        self.anim_counter += 1;

                        if self.anim_counter > 24 {
                            self.cond.set_alive(false);
                        }

                        let len = constants.caret.projectile_dissipation_up_rects.len();
                        self.anim_rect = constants.caret.projectile_dissipation_up_rects[(self.anim_num as usize / 2) % len];
                    }
                    Direction::Right => {
                        self.anim_counter += 1;
                        if self.anim_counter > 2 {
                            self.anim_counter = 0;
                            self.anim_num += 1;

                            if self.anim_num >= constants.caret.projectile_dissipation_right_rects.len() as u16 {
                                self.cond.set_alive(false);
                                return;
                            }

                            self.anim_rect = constants.caret.projectile_dissipation_right_rects[self.anim_num as usize];
                        }
                    }
                    Direction::Bottom => {
                        self.cond.set_alive(false);
                    }
                    Direction::FacingPlayer => unreachable!(),
                }
            }
            CaretType::Shoot => {
                if self.anim_counter == 0 {
                    self.anim_rect = constants.caret.shoot_rects[self.anim_num as usize];
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num >= constants.caret.shoot_rects.len() as u16 {
                        self.cond.set_alive(false);
                    }
                }
            }
            CaretType::SnakeAfterimage | CaretType::SnakeAfterimage2 => {} // dupe, unused
            CaretType::Zzz => {
                if self.anim_counter == 0 {
                    self.anim_rect = constants.caret.zzz_rects[self.anim_num as usize];
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num >= constants.caret.zzz_rects.len() as u16 {
                        self.cond.set_alive(false);
                        return;
                    }
                }

                self.x += 0x80; // 0.4fix9
                self.y -= 0x80;
            }
            CaretType::Exhaust => {
                if self.anim_counter == 0 {
                    self.anim_rect = constants.caret.exhaust_rects[self.anim_num as usize];
                }

                self.anim_counter += 1;
                if self.anim_counter > 1 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num >= constants.caret.exhaust_rects.len() as u16 {
                        self.cond.set_alive(false);
                        return;
                    }
                }

                match self.direction {
                    Direction::Left =>  self.x -= 0x400,  // 2.0fix9
                    Direction::Up =>  self.y -= 0x400,
                    Direction::Right =>  self.x += 0x400,
                    Direction::Bottom =>  self.y += 0x400,
                    Direction::FacingPlayer => unreachable!(),
                }
            }
            CaretType::DrownedQuote => {
                if self.anim_counter == 0 {
                    self.anim_counter = 1;

                    match self.direction {
                        Direction::Left =>  self.anim_rect = constants.caret.drowned_quote_left_rect,
                        Direction::Right =>  self.anim_rect = constants.caret.drowned_quote_right_rect,
                        Direction::FacingPlayer => unreachable!(),
                        _ => {}
                    }
                }
            }
            CaretType::QuestionMark => {
                self.anim_counter += 1;
                if self.anim_counter < 5 {
                    self.y -= 0x800; // 4.0fix9
                }

                if self.anim_counter == 32 {
                    self.cond.set_alive(false);
                }

                self.anim_rect = match self.direction {
                    Direction::Left => { constants.caret.question_left_rect }
                    Direction::Right => { constants.caret.question_right_rect }
                    _ => { self.anim_rect }
                }
            }
            CaretType::LevelUp => {
                self.anim_counter += 1;

                if self.anim_counter == 80 {
                    self.cond.set_alive(false);
                }

                match self.direction {
                    Direction::Left => {
                        if self.anim_counter < 20 {
                            self.y -= 0x400; // 2.0fix9
                        }

                        let count = constants.caret.level_up_rects.len();
                        self.anim_rect = constants.caret.level_up_rects[self.anim_counter as usize / 2 % count]
                    }
                    Direction::Right => {
                        if self.anim_counter < 20 {
                            self.y -= 0x200; // 2.0fix9
                        }

                        let count = constants.caret.level_down_rects.len();
                        self.anim_rect = constants.caret.level_down_rects[self.anim_counter as usize / 2 % count]
                    }
                    _ => {}
                }
            }
            CaretType::HurtParticles => {}
            CaretType::Explosion => {
                if self.anim_counter == 0 {
                    self.anim_rect = constants.caret.explosion_rects[self.anim_num as usize];
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num >= constants.caret.explosion_rects.len() as u16 {
                        self.cond.set_alive(false);
                    }
                }
            }
            CaretType::LittleParticles => {
                if self.anim_num == 0 {
                    match self.direction {
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

                if self.direction == Direction::Left {
                    self.vel_x = (self.vel_x * 4) / 5;
                    self.vel_y = (self.vel_y * 4) / 5;
                }

                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.anim_num == 20 {
                    self.cond.set_alive(false);
                    return;
                }

                let len = constants.caret.little_particles_rects.len();
                self.anim_rect = constants.caret.little_particles_rects[self.anim_num as usize / 2 % len];

                if self.direction == Direction::Right {
                    self.x -= 4 * 0x200;
                }
            }
            CaretType::Unknown => {
                // not implemented because it was apparently broken in og game?
                self.cond.set_alive(false);
            }
            CaretType::SmallProjectileDissipation => {}
            CaretType::Empty => {}
            CaretType::PushJumpKey => {}
        }
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        !self.cond.alive()
    }
}
