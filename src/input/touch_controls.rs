use ggez::{Context, GameResult};
use winit::event::TouchPhase;

use crate::common::Rect;
use crate::engine_constants::EngineConstants;
use crate::texture_set::TextureSet;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TouchControlType {
    None,
    Dialog,
    Controls,
}

#[derive(Copy, Clone)]
pub struct TouchPoint {
    id: u64,
    touch_id: u64,
    position: (f64, f64),
    last_position: (f64, f64),
}

pub struct TouchControls {
    pub control_type: TouchControlType,
    pub points: Vec<TouchPoint>,
    pub interact_icon: bool,
    touch_id_counter: u64,
    clicks: Vec<TouchPoint>,
}

impl TouchControls {
    pub fn new() -> TouchControls {
        TouchControls {
            control_type: TouchControlType::None,
            touch_id_counter: 0,
            interact_icon: false,
            points: Vec::with_capacity(8),
            clicks: Vec::with_capacity(8),
        }
    }

    pub fn process_winit_event(&mut self, scale: f32, touch: winit::event::Touch) {
        match touch.phase {
            TouchPhase::Started | TouchPhase::Moved => {
                if let Some(point) = self.points.iter_mut().find(|p| p.id == touch.id) {
                    point.last_position = point.position;
                    point.position = (touch.location.x / scale as f64, touch.location.y / scale as f64);
                } else {
                    self.touch_id_counter = self.touch_id_counter.wrapping_add(1);

                    let point = TouchPoint {
                        id: touch.id,
                        touch_id: self.touch_id_counter,
                        position: (touch.location.x / scale as f64, touch.location.y / scale as f64),
                        last_position: (0.0, 0.0),
                    };
                    self.points.push(point);

                    if touch.phase == TouchPhase::Started {
                        self.clicks.push(point);
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.points.retain(|p| p.id != touch.id);
                self.clicks.retain(|p| p.id != touch.id);
            }
        }
    }

    pub fn point_in(&self, bounds: Rect) -> Option<u64> {
        for point in self.points.iter() {
            if (point.position.0 as isize) > bounds.left
                && (point.position.0 as isize) < bounds.right
                && (point.position.1 as isize) > bounds.top
                && (point.position.1 as isize) < bounds.bottom {
                return Some(point.touch_id);
            }
        }

        None
    }

    pub fn consume_click_in(&mut self, bounds: Rect) -> bool {
        self.clicks.retain(|p| p.touch_id != 0);

        for point in self.clicks.iter_mut() {
            if (point.position.0 as isize) > bounds.left
                && (point.position.0 as isize) < bounds.right
                && (point.position.1 as isize) > bounds.top
                && (point.position.1 as isize) < bounds.bottom {
                point.touch_id = 0;

                return true;
            }
        }

        false
    }

    pub fn draw(&self, canvas_size: (f32, f32), constants: &EngineConstants, texture_set: &mut TextureSet, ctx: &mut Context) -> GameResult {
        if self.control_type == TouchControlType::Controls {
            let batch = texture_set.get_or_load_batch(ctx, constants, "builtin/touch")?;
            let color = (255, 255, 255, 160);

            for x in 0..3 {
                for y in 0..3 {
                    let mut icon_x = x;
                    let icon_y = y;

                    if self.interact_icon && x == 1 && y == 2 {
                        icon_x = 3;
                    }

                    batch.add_rect_tinted(4.0 + 48.0 * x as f32 + 8.0,
                                   (canvas_size.1 - 4.0 - 48.0 * 3.0) + 48.0 * y as f32 + 8.0,
                                          color,
                                   &Rect::new_size(icon_x * 32, icon_y * 32, 32, 32));
                }
            }


            batch.add_rect_tinted(canvas_size.0 - (4.0 + 48.0) + 8.0, canvas_size.1 - (4.0 + 48.0) + 8.0,
                                  color,
                                  &Rect::new_size(3 * 32, 32, 32, 32));

            batch.add_rect_tinted(canvas_size.0 - (4.0 + 48.0) + 8.0, canvas_size.1 - (4.0 + 48.0) * 2.0 + 8.0,
                                  color,
                                  &Rect::new_size(3 * 32, 0, 32, 32));

            batch.draw(ctx)?;
        }

        Ok(())
    }
}
