use ggez::{self, ContextBuilder};
use ggez::conf::{WindowMode, FullscreenType};
use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics;
use ggez::input::mouse::{MouseButton};
use ggez::timer;
use nalgebra::{Point2};
use std::path::{PathBuf};
use std::env;

use rpg_battle::battle::{BattleState, BattleEvents};
use rpg_battle::explore::{ExploreState, ExploreEvents};
use rpg_battle::fps_meter::{FpsMeter};
use rpg_battle::input::{MoveState};
use rpg_battle::ui::options::{Options};
use rpg_battle::projector::{Projector};

const SCREEN_WIDTH: f32 = 1440.0;
const SCREEN_HEIGHT: f32 = 900.0;
const DESIRED_FPS: u32 = 60;
const RANDOMISE_INTERVAL: f32 = 2.0;

// TODO move all UI into single draw step with spritebatch
// TODO make important state changes wait for animation (eg end battle)
// TODO consider remaining_update_time delta in the draw step
// TODO split battle module into more parts
// TODO revise privacy settings for structs and members
struct MainState {
    fps_meter: FpsMeter,
    font: graphics::Font,
    randomise_timer: f32,
    battle: Option<BattleState>,
    events: Vec<MainEvents>,
    ui_scale: f32,
    ui_scale_input: Options,
    display_settings: bool,
    explore: ExploreState,
    move_state: MoveState
}


enum MainEvents {
    BattleEvent(BattleEvents),
    ExploreEvent(ExploreEvents)
}

fn battle_event_notifier<'a>(main_events: &'a mut Vec<MainEvents>) -> impl 'a + FnMut(BattleEvents) {
    move |battle_event| main_events.push(MainEvents::BattleEvent(battle_event))
}

fn explore_event_notifier<'a>(main_events: &'a mut Vec<MainEvents>) -> impl 'a + FnMut(ExploreEvents) {
    move |explore_event| main_events.push(MainEvents::ExploreEvent(explore_event))
}

// idea
// load image 50x60
// add image 100x100
// total dims 150x160
// split rgba vec of first at 50*4, and second by 100x4
// add 40 blank rows to the first
// zip them together

// let left_image = graphics::Image::new(ctx, "/mountain_landscape_23.png")?;
// let right_image = graphics::Image::new(ctx, "/lidia_spritesheet_fix.png")?;

// let combined_width = std::cmp::max(left_image.width(), right_image.width());
// let combined_height = std::cmp::max(left_image.height(), right_image.height());


impl MainState {
    fn new(ctx: &mut ggez::Context) -> ggez::GameResult<MainState> {

        let maybe_font = graphics::Font::new_glyph_font_bytes(
            ctx,
            include_bytes!("../resources/ferrum.ttf")
        );
        let font = match maybe_font {
            Ok(result) => result,
            Err(e) => panic!("{}", e)
        };

        let s = MainState {
            fps_meter: FpsMeter::new(),
            font: font,
            randomise_timer: 0.0,
            battle: None,
            events: Vec::new(),
            ui_scale: 1.0,
            ui_scale_input: Options::new(5, 2),
            display_settings: false,
            explore: ExploreState::new(ctx, SCREEN_WIDTH, SCREEN_HEIGHT)?,
            move_state: Default::default()
        };

        Ok(s)
    }

    fn flush_events(&mut self) {
        let main_events = &mut self.events;

        while main_events.len() > 0 {
            let event = main_events.remove(0);

            match event {
                MainEvents::BattleEvent(BattleEvents::End(victory)) => {
                    self.battle = None;

                    if victory {
                        self.explore.notify_battle_end();
                    } else {
                        self.explore.notify_player_defeat();
                    }
                },
                MainEvents::BattleEvent(BattleEvents::EnemyDown(id)) => {
                    self.explore.notify_monster_down(id);
                },
                MainEvents::BattleEvent(e) => {
                    match &mut self.battle {
                        Some(battle) => battle.handle_event(&e, battle_event_notifier(main_events)),
                        None => {}
                    }
                },
                MainEvents::ExploreEvent(ExploreEvents::MonsterEncounter(id)) => {
                    match &mut self.battle {
                        Some(battle) => {
                            battle.add_enemy(id);
                        },
                        None => {
                            self.battle = Some(BattleState::new(id));
                        }
                    }
                }
            }
        }
    }
}

impl event::EventHandler for MainState {

    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool
    ) {
        if keycode == KeyCode::Escape {
            std::process::exit(0);
        }

        self.move_state.handle_key_down(&keycode);
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: KeyCode,
        _keymods: KeyMods
    ) {
        self.move_state.handle_key_up(&keycode);
    }

    fn text_input_event(&mut self, _ctx: &mut ggez::Context, character: char) {

        match &mut self.battle {
            Some(battle) => {
                if character == '1' && battle.player_move_pending() {
                    battle.player_attack_move(battle_event_notifier(&mut self.events));
                }

                if character == '2' && battle.player_move_pending() {
                    battle.player_block_move(battle_event_notifier(&mut self.events));
                }
            },
            None => {}
        }

        if character == 'h' {
            self.display_settings = !self.display_settings;
        }

        self.flush_events();
    }

    fn mouse_button_down_event(
        &mut self, _ctx: &mut ggez::Context, _button: MouseButton, x: f32, y: f32
    ) {
        // Select hovered enemy
        match &mut self.battle {
            Some(battle) => {
                if let Some(i) = battle.hovered_enemy {
                    battle.target_enemy = i;
                }
            },
            None => {}
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

        match &mut self.battle {
            Some(battle) => {
                battle.handle_mouse_move(x, y, &projector);
            },
            None => {}
        }

    }

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {

        self.fps_meter.update_start(ctx);

        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.fps_meter.update_loop(ctx);

            let delta = 1.0 / (DESIRED_FPS as f32);

            self.randomise_timer += delta;

            self.explore.update(self.move_state.get_move(), delta, explore_event_notifier(&mut self.events));

            match &mut self.battle {
                Some(battle) => {
                    battle.tick(delta, battle_event_notifier(&mut self.events));
                },
                None => {}
            }

            if self.randomise_timer > RANDOMISE_INTERVAL {
                self.randomise_timer = self.randomise_timer % RANDOMISE_INTERVAL;
            }

            self.flush_events();
        }

        self.fps_meter.update_end(ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.fps_meter.draw_start(ctx);

        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        self.explore.draw(ctx)?;

        let projector = Projector::new(
            Point2::new(0.0, 0.0),
            self.ui_scale,
            SCREEN_WIDTH,
            SCREEN_HEIGHT
        ).margins(90.0, 20.0);

        match &mut self.battle {
            Some(battle) => {
                battle.draw(ctx, &projector)?;
            },
            None => {}
        }

        if self.display_settings {
            let settings_projector = projector.centered(300.0, 20.0);
            let mut ui_scale_text = graphics::Text::new(format!("Scale {}", self.ui_scale));
            ui_scale_text.set_font(self.font, graphics::Scale::uniform(settings_projector.scale(graphics::DEFAULT_FONT_SCALE * 2.0)));

            graphics::draw(ctx, &ui_scale_text, (settings_projector.origin(),))?;
            self.ui_scale_input.draw(ctx, &settings_projector.top_right(170.0))?;
        }

        self.fps_meter.draw(ctx)?;
        graphics::present(ctx)?;

        self.fps_meter.draw_end(ctx);
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
