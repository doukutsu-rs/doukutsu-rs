use winit::event::TouchPhase;
use crate::texture_set::TextureSet;
use ggez::{Context, GameResult};
use crate::common::{KeyState, Rect};
use crate::engine_constants::EngineConstants;

struct TouchPoint {
    id: u64,
    position: (f64, f64),
    last_position: (f64, f64),
}

pub struct TouchControls {
    points: Vec<TouchPoint>,
}

impl TouchControls {
    pub fn new() -> TouchControls {
        TouchControls {
            points: Vec::with_capacity(8),
        }
    }
    
    pub fn process_winit_event(&mut self, scale: f32, key_state: &mut KeyState, touch: winit::event::Touch) {
        match touch.phase {
            TouchPhase::Started | TouchPhase::Moved => {
                if let Some(point) = self.points.iter_mut().find(|p| p.id == touch.id) {
                    point.last_position = point.position;
                    point.position = (touch.location.x, touch.location.y);
                } else {
                    self.points.push(TouchPoint {
                        id: touch.id,
                        position: (touch.location.x, touch.location.y),
                        last_position: (0.0, 0.0)
                    });
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.points.retain(|p| p.id != touch.id);
            }
        }
    }

    pub fn draw(&self, constants: &EngineConstants, texture_set: &mut TextureSet, ctx: &mut Context) -> GameResult {
        let batch = texture_set.get_or_load_batch(ctx, constants, "Caret")?;
        let rect = Rect::new_size(104, 120, 24, 24);
        for point in self.points.iter() {
            batch.add_rect(point.position.0 as f32 - 12.0, point.position.1 as f32 - 12.0, &rect);
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
