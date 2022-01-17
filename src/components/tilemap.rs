use crate::common::Rect;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::shared_game_state::{SharedGameState, TileSize};
use crate::stage::{BackgroundType, Stage, StageTexturePaths};

pub struct Tilemap;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TileLayer {
    Background,
    Middleground,
    Foreground,
    Snack,
}

impl Tilemap {
    pub fn new() -> Self {
        Tilemap
    }

    pub fn draw(
        &self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        frame: &Frame,
        layer: TileLayer,
        textures: &StageTexturePaths,
        stage: &Stage,
    ) -> GameResult {
        if stage.map.tile_size == TileSize::Tile8x8 && layer == TileLayer::Snack {
            return Ok(());
        }

        let tex = match layer {
            TileLayer::Snack => "Npc/NpcSym",
            TileLayer::Background => &textures.tileset_bg,
            TileLayer::Middleground => &textures.tileset_mg,
            TileLayer::Foreground => &textures.tileset_fg,
        };

        let (layer_offset, layer_width, layer_height, uses_layers) =
            if let Some(pxpack_data) = &stage.data.pxpack_data {
                match layer {
                    TileLayer::Background => {
                        (pxpack_data.offset_bg as usize, pxpack_data.size_bg.0, pxpack_data.size_bg.1, true)
                    }
                    TileLayer::Middleground => {
                        (pxpack_data.offset_mg as usize, pxpack_data.size_mg.0, pxpack_data.size_mg.1, true)
                    }
                    _ => (0, pxpack_data.size_fg.0, pxpack_data.size_fg.1, true),
                }
            } else {
                (0, stage.map.width, stage.map.height, false)
            };

        if !uses_layers && layer == TileLayer::Middleground {
            return Ok(());
        }

        let tile_size = state.tile_size.as_int();
        let tile_sizef = state.tile_size.as_float();
        let halft = tile_size / 2;
        let halftf = tile_sizef / 2.0;

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, tex)?;
        let mut rect = Rect::new(0, 0, tile_size as u16, tile_size as u16);
        let (mut frame_x, mut frame_y) = frame.xy_interpolated(state.frame_time);

        if let Some(pxpack_data) = &stage.data.pxpack_data {
            let (fx, fy) = match layer {
                TileLayer::Background => pxpack_data.scroll_bg.transform_camera_pos(frame_x, frame_y),
                TileLayer::Middleground => pxpack_data.scroll_mg.transform_camera_pos(frame_x, frame_y),
                _ => pxpack_data.scroll_fg.transform_camera_pos(frame_x, frame_y),
            };

            frame_x = fx;
            frame_y = fy;
        }

        let tile_start_x = (frame_x as i32 / tile_size).clamp(0, layer_width as i32) as usize;
        let tile_start_y = (frame_y as i32 / tile_size).clamp(0, layer_height as i32) as usize;
        let tile_end_x =
            ((frame_x as i32 + 8 + state.canvas_size.0 as i32) / tile_size + 1).clamp(0, layer_width as i32) as usize;
        let tile_end_y = ((frame_y as i32 + halft + state.canvas_size.1 as i32) / tile_size + 1)
            .clamp(0, layer_height as i32) as usize;

        if layer == TileLayer::Snack {
            rect = state.constants.world.snack_rect;
        }

        for y in tile_start_y..tile_end_y {
            for x in tile_start_x..tile_end_x {
                let tile = *stage.map.tiles.get((y * layer_width as usize) + x + layer_offset).unwrap();
                match layer {
                    _ if uses_layers => {
                        if tile == 0 {
                            continue;
                        }

                        let tile_size = tile_size as u16;
                        rect.left = (tile as u16 % 16) * tile_size;
                        rect.top = (tile as u16 / 16) * tile_size;
                        rect.right = rect.left + tile_size;
                        rect.bottom = rect.top + tile_size;
                    }
                    TileLayer::Background => {
                        if stage.map.attrib[tile as usize] >= 0x20 {
                            continue;
                        }

                        let tile_size = tile_size as u16;
                        rect.left = (tile as u16 % 16) * tile_size;
                        rect.top = (tile as u16 / 16) * tile_size;
                        rect.right = rect.left + tile_size;
                        rect.bottom = rect.top + tile_size;
                    }
                    TileLayer::Foreground => {
                        let attr = stage.map.attrib[tile as usize];

                        if attr < 0x40 || attr >= 0x80 || attr == 0x43 {
                            continue;
                        }

                        let tile_size = tile_size as u16;
                        rect.left = (tile as u16 % 16) * tile_size;
                        rect.top = (tile as u16 / 16) * tile_size;
                        rect.right = rect.left + tile_size;
                        rect.bottom = rect.top + tile_size;
                    }
                    TileLayer::Snack => {
                        if stage.map.attrib[tile as usize] != 0x43 {
                            continue;
                        }
                    }
                    _ => {}
                }

                batch.add_rect(
                    (x as f32 * tile_sizef - halftf) - frame_x,
                    (y as f32 * tile_sizef - halftf) - frame_y,
                    &rect,
                );
            }
        }

        batch.draw(ctx)?;

        if layer == TileLayer::Foreground && stage.data.background_type == BackgroundType::Water {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &textures.background)?;
            let rect_top = Rect { left: 0, top: 0, right: 32, bottom: 16 };
            let rect_middle = Rect { left: 0, top: 16, right: 32, bottom: 48 };

            let tile_start_x = frame_x as i32 / 32;
            let tile_end_x = (frame_x + 16.0 + state.canvas_size.0) as i32 / 32 + 1;
            let water_y = state.water_level as f32 / 512.0;
            let tile_count_y = (frame_y + 16.0 + state.canvas_size.1 - water_y) as i32 / 32 + 1;

            for x in tile_start_x..tile_end_x {
                batch.add_rect((x as f32 * 32.0) - frame_x, water_y - frame_y, &rect_top);

                for y in 0..tile_count_y {
                    batch.add_rect((x as f32 * 32.0) - frame_x, (y as f32 * 32.0) + water_y - frame_y, &rect_middle);
                }
            }

            batch.draw(ctx)?;
        }

        Ok(())
    }
}
