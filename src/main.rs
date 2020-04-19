use ggez::{self, ContextBuilder};
use ggez::conf::{WindowMode, FullscreenType};
use ggez::event;
use ggez::graphics;
use ggez::input::mouse::{MouseButton};
use ggez::timer;
use nalgebra::{Point2};

use rpg_battle::battle::{BattleState, BattleEvents};
use rpg_battle::ui::options::{Options};

const DESIRED_FPS: u32 = 60;
const RANDOMISE_INTERVAL: f32 = 2.0;

// TODO make rendering responsive
//  - anchor rendering to viewport corners
//  - all dimension values to be passed through a scaling function
// TODO make important state changes wait for animation (eg end battle)
// TODO split battle module into more parts
// TODO revise privacy settings for structs and members
struct MainState {
    font: graphics::Font,
    randomise_timer: f32,
    battle: BattleState,
    events: Vec<MainEvents>,
    ui_scale: f32,
    ui_scale_input: Options
}


enum MainEvents {
    BattleEvent(BattleEvents)
}

fn battle_event_notifier<'a>(main_events: &'a mut Vec<MainEvents>) -> impl 'a + FnMut(BattleEvents) {
    move |battle_event| main_events.push(MainEvents::BattleEvent(battle_event))
}

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
            font: font,
            randomise_timer: 0.0,
            battle: BattleState::new(),
            events: Vec::new(),
            ui_scale: 1.0,
            ui_scale_input: Options::new(5, 2)
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

        self.flush_events();
    }

    fn mouse_button_down_event(
        &mut self, _ctx: &mut ggez::Context, _button: MouseButton, x: f32, y: f32
    ) {
        if let Some(i) = self.battle.hovered_enemy {
            self.battle.target_enemy = i;
        }

        let input_value = self.ui_scale_input.handle_mouse_down(x - 500.0, y - 500.0);

        self.ui_scale = match input_value {
            0 => 0.7,
            1 => 0.9,
            3 => 1.2,
            4 => 1.5,
            _ => 1.0
        };
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut ggez::Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32
    ) {
        self.battle.timeline.highlighted_subject = None;
        self.battle.hovered_enemy = None;

        if y < 110.0 {
            for (i, enemy) in self.battle.enemies.iter().enumerate().rev() {
                if x > 600.0 - 140.0 * i as f32 && enemy.stats.current_hp > 0 {
                    self.battle.hovered_enemy = Some(i);
                    self.battle.timeline.highlighted_subject = Some(enemy.timeline_handle);
                }
            }
        }
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

        let mut hello_world = graphics::Text::new(format!("Scale {}", self.ui_scale));

        hello_world.set_font(self.font, graphics::Scale::uniform(graphics::DEFAULT_FONT_SCALE * 2.0));

        graphics::draw(ctx, &hello_world, (Point2::new(100.0, 0.0),))?;

        self.battle.draw(ctx)?;

        self.ui_scale_input.draw(ctx, Point2::new(500.0, 500.0))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> ggez::GameResult {

    // Make a Context.
    let (ctx, event_loop) = &mut ContextBuilder::new("dancras/rpg_battle", "dancras")
        .window_mode(
            WindowMode::default()
                .dimensions(1440.0, 900.0)
                .fullscreen_type(FullscreenType::True)
        )
        .build()
        .expect("Failed to build ggez context");

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
