use ggez::{self, ContextBuilder};
use ggez::conf::{WindowMode, FullscreenType};
use ggez::event;
use ggez::graphics;
use ggez::input::mouse::{MouseButton};
use ggez::timer;
use nalgebra::{Point2};
use patchwork::{TileSet, TileParams};
use std::path::{PathBuf};
use std::env;
use tiled_json_rs as tiled;

use rpg_battle::battle::{BattleState, BattleEvents};
use rpg_battle::ui::options::{Options};
use rpg_battle::projector::{Projector};

const SCREEN_WIDTH: f32 = 1440.0;
const SCREEN_HEIGHT: f32 = 900.0;
const DESIRED_FPS: u32 = 60;
const RANDOMISE_INTERVAL: f32 = 2.0;
const EXPLORE_WIDTH: f32 = 512.0;
const EXPLORE_HEIGHT: f32 = 288.0;

// TODO make important state changes wait for animation (eg end battle)
// TODO consider remaining_update_time delta in the draw step
// TODO split battle module into more parts
// TODO revise privacy settings for structs and members
struct MainState {
    font: graphics::Font,
    randomise_timer: f32,
    battle: BattleState,
    events: Vec<MainEvents>,
    ui_scale: f32,
    ui_scale_input: Options,
    display_settings: bool,
    tiles: TileSet<u32>,
    tiles_offset: f32
}


enum MainEvents {
    BattleEvent(BattleEvents)
}

fn battle_event_notifier<'a>(main_events: &'a mut Vec<MainEvents>) -> impl 'a + FnMut(BattleEvents) {
    move |battle_event| main_events.push(MainEvents::BattleEvent(battle_event))
}

impl MainState {
    fn new(ctx: &mut ggez::Context) -> ggez::GameResult<MainState> {

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

        let tile_scale = SCREEN_HEIGHT / EXPLORE_HEIGHT;
        let tile_cols = (EXPLORE_WIDTH / 32.0) as i32;
        let tile_rows = (EXPLORE_HEIGHT / 32.0) as i32;

        for layer in &map.layers {
            match &layer.layer_type {
                tiled::LayerType::TileLayer(layer_tiles) => {
                    for i in 0..layer_tiles.data.len() {
                        let tile = layer_tiles.data[i];
                        let x = i as i32 % 50;
                        let y = i as i32 / 50;

                        if tile > 0 && x < tile_cols && y < tile_rows {
                            tiles.queue_tile::<_, TileParams>(
                                tile,
                                [x, y],
                                Some(TileParams {
                                    color: None,
                                    scale: Some([tile_scale, tile_scale].into())
                                })
                            ).expect("Failed to queue tile");
                        }
                    }
                }
                _ => {}
            }
        }

        let maybe_font = graphics::Font::new_glyph_font_bytes(
            ctx,
            include_bytes!("../resources/ferrum.ttf")
        );
        let font = match maybe_font {
            Ok(result) => result,
            Err(e) => panic!("{}", e)
        };

        let s = MainState {
            font: font,
            randomise_timer: 0.0,
            battle: BattleState::new(),
            events: Vec::new(),
            ui_scale: 1.0,
            ui_scale_input: Options::new(5, 2),
            display_settings: false,
            tiles: tiles,
            tiles_offset: (SCREEN_WIDTH - EXPLORE_WIDTH * tile_scale) / 2.0
        };

        Ok(s)
    }

    fn flush_events(&mut self) {
        let main_events = &mut self.events;

        while main_events.len() > 0 {
            let event = main_events.remove(0);

            match event {
                MainEvents::BattleEvent(BattleEvents::End) => {
                    self.battle = BattleState::new();
                },
                MainEvents::BattleEvent(e) => {
                    self.battle.handle_event(&e, battle_event_notifier(main_events));
                }
            }
        }
    }
}

impl event::EventHandler for MainState {

    fn text_input_event(&mut self, _ctx: &mut ggez::Context, character: char) {
        if character == '1' && self.battle.player_move_pending() {
            self.battle.player_attack_move(battle_event_notifier(&mut self.events));
        }

        if character == '2' && self.battle.player_move_pending() {
            self.battle.player_block_move(battle_event_notifier(&mut self.events));
        }

        if character == 's' {
            self.display_settings = !self.display_settings;
        }

        self.flush_events();
    }

    fn mouse_button_down_event(
        &mut self, _ctx: &mut ggez::Context, _button: MouseButton, x: f32, y: f32
    ) {
        // Select hovered enemy
        if let Some(i) = self.battle.hovered_enemy {
            self.battle.target_enemy = i;
        }

        // Update UI scale input
        if self.display_settings {
            let projector = Projector::new(
                Point2::new(0.0, 0.0),
                self.ui_scale,
                SCREEN_WIDTH,
                SCREEN_HEIGHT
            ).margins(90.0, 20.0);

            let settings_projector = projector.centered(300.0, 20.0);
            let ui_scale_input_projector = settings_projector.top_right(170.0);
            let input_value = self.ui_scale_input.handle_mouse_down(
                ui_scale_input_projector.to_local_x(x),
                ui_scale_input_projector.to_local_y(y),
                &ui_scale_input_projector
            );

            self.ui_scale = match input_value {
                0 => 0.7,
                1 => 0.9,
                3 => 1.2,
                4 => 1.5,
                _ => 1.0
            };
        }
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut ggez::Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32
    ) {
        let projector = Projector::new(
            Point2::new(0.0, 0.0),
            self.ui_scale,
            SCREEN_WIDTH,
            SCREEN_HEIGHT
        ).margins(90.0, 20.0);

        self.battle.handle_mouse_move(x, y, &projector);
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {

        while timer::check_update_time(ctx, DESIRED_FPS) {

            let delta = 1.0 / (DESIRED_FPS as f32);

            self.randomise_timer += delta;

            self.battle.tick(delta, battle_event_notifier(&mut self.events));

            if self.randomise_timer > RANDOMISE_INTERVAL {
                self.randomise_timer = self.randomise_timer % RANDOMISE_INTERVAL;
            }

            self.flush_events();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        self.tiles.draw(ctx, (Point2::new(self.tiles_offset, 0.0),))?;

        let projector = Projector::new(
            Point2::new(0.0, 0.0),
            self.ui_scale,
            SCREEN_WIDTH,
            SCREEN_HEIGHT
        ).margins(90.0, 20.0);

        self.battle.draw(ctx, &projector)?;

        if self.display_settings {
            let settings_projector = projector.centered(300.0, 20.0);
            let mut ui_scale_text = graphics::Text::new(format!("Scale {}", self.ui_scale));
            ui_scale_text.set_font(self.font, graphics::Scale::uniform(settings_projector.scale(graphics::DEFAULT_FONT_SCALE * 2.0)));

            graphics::draw(ctx, &ui_scale_text, (settings_projector.origin(),))?;
            self.ui_scale_input.draw(ctx, &settings_projector.top_right(170.0))?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> ggez::GameResult {

    // Make a Context.
    let mut cb = ContextBuilder::new("dancras/rpg_battle", "dancras");

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }

    let (ctx, event_loop) = &mut cb
        .window_mode(
            WindowMode::default()
                .dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
                .fullscreen_type(FullscreenType::True)
        )
        .build()
        .expect("Failed to build ggez context");

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
