use ggez::{self, graphics};
use nalgebra::{Point2};
use patchwork::{TileSet, TileParams};
use std::path::{PathBuf};
use tiled_json_rs as tiled;

use crate::input::{Move};

const EXPLORE_WIDTH: f32 = 512.0;
const EXPLORE_HEIGHT: f32 = 288.0;
const EXPLORE_SPEED: f32 = 80.0;
const DIAGONAL_FACTOR: f32 = 0.7071067811865475;

pub struct ExploreState {
    tiles: TileSet<u32>,
    tile_scale: f32,
    tiles_offset: f32,
    x: f32,
    y: f32,
    tile_x: i32,
    tile_y: i32,
    map: tiled::Map
}

impl ExploreState {
    pub fn new(ctx: &mut ggez::Context, screen_width: f32, screen_height: f32) -> ggez::GameResult<Self> {

        // Currently requires a symlink in the project root to the tileset
        // as the tiled library file paths are relative to the project root,
        // not the map file.
        let map = tiled::Map::load_from_file(&PathBuf::from("resources/test_map.json"))
            .expect("Failed to load map");

        let tile_set_filename = map.tile_sets[0].image.clone().into_os_string().into_string()
            .expect("Failed to get tile set filename");

        let tileset_image = graphics::Image::new(ctx, format!("/{}", tile_set_filename))?;

        let mut tiles: TileSet<u32> = TileSet::new(tileset_image, [32, 32]);

        let mut tile_id = 1;
        for row in 0..16 {
            for col in 0..16 {
                tiles.register_tile(tile_id, [col, row])
                    .expect("Failed to register tile");
                tile_id += 1;
            }
        }

        let tile_scale = screen_height / EXPLORE_HEIGHT;

        Ok(Self {
            tiles: tiles,
            tile_scale: tile_scale,
            tiles_offset: (screen_width - EXPLORE_WIDTH * tile_scale) / 2.0,
            x: 0.0,
            y: 0.0,
            // Bogus numbers to trigger calculation on first update
            tile_x: 555,
            tile_y: 555,
            map: map
        })

    }

    pub fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.tiles.draw(
            ctx,
            (Point2::new(
                self.tiles_offset - self.x * self.tile_scale,
                -self.y * self.tile_scale
            ),)
        )?;

        Ok(())
    }

    pub fn update(&mut self, current_move: Move, delta: f32) {

        let movement = EXPLORE_SPEED * delta;
        let diagonal_movement = movement * DIAGONAL_FACTOR;

        match current_move {
            Move::Up => self.y -= movement,
            Move::UpRight => {
                self.y -= diagonal_movement;
                self.x += diagonal_movement;
            },
            Move::Right => self.x += movement,
            Move::DownRight => {
                self.y += diagonal_movement;
                self.x += diagonal_movement;
            },
            Move::Down => self.y += movement,
            Move::DownLeft => {
                self.y += diagonal_movement;
                self.x -= diagonal_movement;
            },
            Move::Left => self.x -= movement,
            Move::UpLeft => {
                self.y -= diagonal_movement;
                self.x -= diagonal_movement;
            },
            Move::None => {}
        }

        let new_tile_x = self.x as i32 / 32;
        let new_tile_y = self.y as i32 / 32;

        if self.tile_x != new_tile_x || self.tile_y != new_tile_y {
            self.tile_x = new_tile_x;
            self.tile_y = new_tile_y;

            self.tiles.clear_queue();

            let tile_cols = (EXPLORE_WIDTH / 32.0) as i32;
            let tile_rows = (EXPLORE_HEIGHT / 32.0) as i32;

            for layer in &self.map.layers {
                match &layer.layer_type {
                    tiled::LayerType::TileLayer(layer_tiles) => {
                        for i in 0..layer_tiles.data.len() {
                            let tile = layer_tiles.data[i];
                            let x = i as i32 % 50;
                            let y = i as i32 / 50;
                            let start_x = self.tile_x;
                            let end_x = start_x + tile_cols;
                            let start_y = self.tile_y;
                            let end_y = start_y + tile_rows;

                            if tile > 0 &&
                                x >= start_x && x <= end_x &&
                                y >= start_y && y <= end_y
                            {
                                self.tiles.queue_tile::<_, TileParams>(
                                    tile,
                                    [x, y],
                                    Some(TileParams {
                                        color: None,
                                        scale: Some([self.tile_scale, self.tile_scale].into())
                                    })
                                ).expect("Failed to queue tile");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }


    }
}


