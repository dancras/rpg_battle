use ggez::{self, graphics};
use nalgebra::{Point2};
use patchwork::{TileSet, TileParams};
use rand::{random};
use std::path::{PathBuf};
use tiled_json_rs as tiled;

use crate::input::{Move};
use crate::palette;

const EXPLORE_WIDTH: f32 = 512.0;
const EXPLORE_HEIGHT: f32 = 288.0;
const EXPLORE_SPEED: f32 = 80.0;
const DIAGONAL_FACTOR: f32 = 0.7071067811865475;

struct Monster {
    id: u32,
    position: Point2<f32>,
    in_battle: bool,
    ko: bool
}

pub enum ExploreEvents {
    MonsterEncounter(u32)
}

// On empty map reset all monsters
// On player death reset all monsters
// Only draw monsters which are on screen
pub struct ExploreState {
    tiles: TileSet<u32>,
    tile_scale: f32,
    tiles_offset: f32,
    tile_x: i32,
    tile_y: i32,
    map: tiled::Map,
    camera_x: f32,
    camera_y: f32,
    scene: SceneState
}

struct SceneState {
    x: f32,
    y: f32,
    monsters: Vec<Monster>
}

impl SceneState {
    fn new() -> Self {

        let mut monsters = Vec::new();
        let mut monster_id = 1;

        for row in 0..8 {
            for col in 0..8 {
                let row = row as f32;
                let col = col as f32;
                let rand_x = 200.0 * col + random::<f32>() * 200.0;
                let rand_y = 200.0 * row + random::<f32>() * 200.0;
                monsters.push(Monster {
                    id: monster_id,
                    position: Point2::new(rand_x, rand_y),
                    in_battle: false,
                    ko: false
                });
                monster_id += 1;
            }
        }

        monsters.remove(0);

        Self {
            monsters: monsters,
            x: 128.0,
            y: 72.0,
        }

    }
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
            // Bogus numbers to trigger calculation on first update
            tile_x: 555,
            tile_y: 555,
            map: map,
            camera_x: 0.0,
            camera_y: 0.0,
            scene: SceneState::new()
        })

    }

    pub fn notify_monster_down(&mut self, id: u32) {

        for monster in &mut self.scene.monsters {
            if monster.id == id {
                monster.ko = true;
                break;
            }
        }

    }

    pub fn notify_battle_end(&mut self) {
        self.scene.monsters.retain(|m| !m.ko);
    }

    pub fn notify_player_defeat(&mut self) {
        self.scene = SceneState::new();
    }

    pub fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.tiles.draw(
            ctx,
            (Point2::new(
                self.tiles_offset - self.camera_x * self.tile_scale,
                -self.camera_y * self.tile_scale
            ),)
        )?;

        for monster in &self.scene.monsters {

            let monster_block = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect {
                    x: -10.0 * self.tile_scale,
                    y: -30.0 * self.tile_scale,
                    w: 20.0 * self.tile_scale,
                    h: 30.0 * self.tile_scale
                },
                if monster.ko { graphics::BLACK } else if monster.in_battle { palette::RED } else { graphics::WHITE }
            )?;

            graphics::draw(
                ctx,
                &monster_block,
                (Point2::new(
                    (monster.position.x - self.camera_x) * self.tile_scale + self.tiles_offset,
                    (monster.position.y - self.camera_y) * self.tile_scale
                ),)
            )?;

        }

        let player = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect {
                x: -13.0 * self.tile_scale,
                y: -46.0 * self.tile_scale,
                w: 26.0 * self.tile_scale,
                h: 46.0 * self.tile_scale
            },
            graphics::BLACK
        )?;

        graphics::draw(
            ctx,
            &player,
            (Point2::new((self.scene.x - self.camera_x) * self.tile_scale + self.tiles_offset, (self.scene.y - self.camera_y) * self.tile_scale),)
        )?;

        Ok(())
    }

    pub fn update<F: FnMut(ExploreEvents)>(&mut self, current_move: Move, delta: f32, mut notify: F) {

        // Movement
        let movement = EXPLORE_SPEED * delta;
        let diagonal_movement = movement * DIAGONAL_FACTOR;

        match current_move {
            Move::Up => self.scene.y -= movement,
            Move::UpRight => {
                self.scene.y -= diagonal_movement;
                self.scene.x += diagonal_movement;
            },
            Move::Right => self.scene.x += movement,
            Move::DownRight => {
                self.scene.y += diagonal_movement;
                self.scene.x += diagonal_movement;
            },
            Move::Down => self.scene.y += movement,
            Move::DownLeft => {
                self.scene.y += diagonal_movement;
                self.scene.x -= diagonal_movement;
            },
            Move::Left => self.scene.x -= movement,
            Move::UpLeft => {
                self.scene.y -= diagonal_movement;
                self.scene.x -= diagonal_movement;
            },
            Move::None => {}
        }

        // Monster collision
        for monster in &mut self.scene.monsters {
            if !monster.in_battle {
                let dist_x = (monster.position.x - self.scene.x).abs();
                let dist_y = (monster.position.y - self.scene.y).abs();

                let dist = ((dist_x * dist_x) + (dist_y * dist_y)).sqrt();

                if dist < 50.0 {
                    monster.in_battle = true;
                    notify(ExploreEvents::MonsterEncounter(monster.id));
                }
            }
        }

        // Manage tiles
        self.camera_x = self.scene.x - EXPLORE_WIDTH / 2.0;

        if self.camera_x < 0.0 {
            self.camera_x = 0.0;
        }

        self.camera_y = self.scene.y - EXPLORE_HEIGHT / 2.0;

        if self.camera_y < 0.0 {
            self.camera_y = 0.0;
        }

        let new_tile_x = self.camera_x as i32 / 32;
        let new_tile_y = self.camera_y as i32 / 32;

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


