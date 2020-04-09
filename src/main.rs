use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra::{Point2};
use ggez::timer;
use rand::{random};
use rpg_battle::palette;
use rpg_battle::hud::resource_guage::{self, ResourceGuage};
use rpg_battle::hud::balance_guage::{self, BalanceGuage};

const DESIRED_FPS: u32 = 60;

struct MainState {
    font: graphics::Font,
    pos_x: f32,
    player_fatigue: ResourceGuage,
    player_balance: BalanceGuage,
    balance_timer: f32
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
            player_fatigue: ResourceGuage {
                color: palette::GREEN,
                max_value: 100.0,
                current_value: 0.0
            },
            player_balance: BalanceGuage::new(0.0),
            pos_x: 0.0,
            balance_timer: 0.0
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let delta = 1.0 / (DESIRED_FPS as f32);

            self.balance_timer += delta;

            balance_guage::update(&mut self.player_balance, delta);

            if self.balance_timer > 3.0 {
                self.balance_timer = self.balance_timer % 3.0;
                self.player_balance.update(random::<f32>());
            }

            self.pos_x = self.pos_x % 800.0 + 1.0;
            self.player_fatigue.current_value = self.player_fatigue.current_value % self.player_fatigue.max_value + 1.0;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let mut hello_world = graphics::Text::new("HELLO WORLD");

        hello_world.set_font(self.font, graphics::Scale::uniform(graphics::DEFAULT_FONT_SCALE * 2.0));

        let player_fatigue_guage = resource_guage::create_mesh(ctx, &self.player_fatigue)?;
        let player_balance_guage = balance_guage::create_mesh(ctx, &self.player_balance)?;

        graphics::draw(ctx, &hello_world, (Point2::new(100.0, 0.0),))?;

        graphics::draw(ctx, &player_fatigue_guage, (Point2::new(100.0, 100.0),))?;

        graphics::draw(ctx, &player_balance_guage, (Point2::new(100.0, 130.0),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> ggez::GameResult { 
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}