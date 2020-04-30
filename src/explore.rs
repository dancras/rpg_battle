use ggez;
use ggez::graphics::{self, DrawParam};
use nalgebra::{Point2};
use patchwork::{TileSet, TileParams};
use rand::{random};
use std::path::{PathBuf};
use tiled_json_rs as tiled;

use crate::input::{Move};

const EXPLORE_WIDTH: f32 = 512.0;
const EXPLORE_HEIGHT: f32 = 288.0;
const EXPLORE_SPEED: f32 = 80.0;
const DIAGONAL_FACTOR: f32 = 0.7071067811865475;
const PLAYER_ANIMATION_FPS: f32 = 10.0;
const MONSTER_ANIMATION_FPS: f32 = 10.0;

struct Monster {
    id: u32,
    position: Point2<f32>,
    in_battle: bool,
    ko: bool
}

pub enum ExploreEvents {
    MonsterEncounter(u32)
}

enum Facing {
    Up,
    Right,
    Down,
    Left
}

// Create something to dynamically pack resources into a spritebatch texture
// Draw world in layers including player character
//  Don't worry about player behind world objects,
//  it adds nothing to gameplay
pub struct ExploreState {
    tiles: TileSet<u32>,
    tile_scale: f32,
    tiles_offset: f32,
    tile_x: i32,
    tile_y: i32,
    map: tiled::Map,
    camera_x: f32,
    camera_y: f32,
    scene: SceneState,
    player_sprite: graphics::Image,
    player_frame_timer: f32,
    player_facing: Facing,
    monster_sprite: graphics::Image,
    monster_frame_timer: f32
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
            y: 128.0,
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
            scene: SceneState::new(),
            player_sprite: graphics::Image::new(ctx, "/lidia_spritesheet_fix.png")?,
            player_frame_timer: 0.0,
            player_facing: Facing::Down,
            monster_sprite: graphics::Image::new(ctx, "/beetle_move_attack.png")?,
            monster_frame_timer: 0.0,
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

        if self.scene.monsters.len() == 0 {
            self.scene = SceneState::new();
        }
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

            if monster.position.x < self.camera_x - 16.0 ||
                monster.position.x > self.camera_x + EXPLORE_WIDTH + 16.0 ||
                monster.position.y < self.camera_y ||
                monster.position.y > self.camera_y + EXPLORE_HEIGHT + 32.0
            {
                continue;
            }

            let monster_frame = (self.monster_frame_timer * MONSTER_ANIMATION_FPS % 4.0) as i8;

            graphics::draw(
                ctx,
                &self.monster_sprite,
                DrawParam {
                    src: graphics::Rect {
                        x: monster_frame as f32 / 10.0,
                        y: 0.5,
                        w: 1.0 / 10.0,
                        h: 0.25
                    },
                    dest: [
                        (monster.position.x - self.camera_x - 16.0) * self.tile_scale + self.tiles_offset,
                        (monster.position.y - self.camera_y - 32.0) * self.tile_scale
                    ].into(),
                    scale: [self.tile_scale, self.tile_scale].into(),
                    ..Default::default()
                }
            )?;

        }

        let player_sprite_y_offset = match self.player_facing {
            Facing::Up => 0.0,
            Facing::Right => 0.75,
            Facing::Down => 0.5,
            Facing::Left => 0.25
        };

        let player_frame = (self.player_frame_timer * PLAYER_ANIMATION_FPS % 9.0) as i8;

        graphics::draw(
            ctx,
            &self.player_sprite,
            DrawParam {
                src: graphics::Rect {
                    x: player_frame as f32 / 9.0,
                    y: player_sprite_y_offset,
                    w: 1.0 / 9.0,
                    h: 0.25
                },
                dest: [(self.scene.x - self.camera_x - 32.0) * self.tile_scale + self.tiles_offset, (self.scene.y - self.camera_y - 64.0) * self.tile_scale].into(),
                scale: [self.tile_scale, self.tile_scale].into(),
                ..Default::default()
            }
        )?;

        Ok(())
    }

    pub fn update<F: FnMut(ExploreEvents)>(&mut self, current_move: Move, delta: f32, mut notify: F) {

        // Movement
        let movement = EXPLORE_SPEED * delta;
        let diagonal_movement = movement * DIAGONAL_FACTOR;

        match current_move {
            Move::Up => {
                self.scene.y -= movement;
                self.player_facing = Facing::Up;
            }
            Move::UpRight => {
                self.scene.y -= diagonal_movement;
                self.scene.x += diagonal_movement;
            },
            Move::Right => {
                self.scene.x += movement;
                self.player_facing = Facing::Right;
            }
            Move::DownRight => {
                self.scene.y += diagonal_movement;
                self.scene.x += diagonal_movement;
            },
            Move::Down => {
                self.scene.y += movement;
                self.player_facing = Facing::Down;
            },
            Move::DownLeft => {
                self.scene.y += diagonal_movement;
                self.scene.x -= diagonal_movement;
            },
            Move::Left => {
                self.scene.x -= movement;
                self.player_facing = Facing::Left
            },
            Move::UpLeft => {
                self.scene.y -= diagonal_movement;
                self.scene.x -= diagonal_movement;
            },
            Move::None => {}
        }

        // Player animation
        match current_move {
            Move::None => self.player_frame_timer = 0.0,
            _ => {
                self.player_frame_timer += if self.player_frame_timer == 0.0 {
                    1.0 / PLAYER_ANIMATION_FPS
                } else {
                    delta
                }
            }
        };

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

        // Monster animation
        self.monster_frame_timer += delta;

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


