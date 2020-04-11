use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra::{Point2};
use ggez::timer;
use rand::{random};
use rpg_battle::palette;
use rpg_battle::hud::action_timeline::{self, ActionTimeline};
use rpg_battle::hud::resource_guage::{self, ResourceGuage};
use rpg_battle::hud::balance_guage::{self, BalanceGuage};

const DESIRED_FPS: u32 = 60;
const RANDOMISE_INTERVAL: f32 = 2.0;
const PLAYER_MAX_FATIGUE: i32 = 100;
const PLAYER_FIRST_ACTION: f32 = 50.0;
const ENEMY_MAX_HP: i32 = 50;
const ENEMY_FIRST_ACTION: f32 = 100.0;
const ACTION_POINTS_PER_SECOND: f32 = 60.0;
const ATTACK_DAMAGE: i32 = 10;
const ATTACK_ACTION_TIME: f32 = 250.0;

struct MainState {
    font: graphics::Font,
    pos_x: f32,
    action_time: f32,
    timeline: ActionTimeline,
    player: Player,
    player_fatigue_guage: ResourceGuage,
    player_balance: BalanceGuage,
    player_attack_pending: bool,
    player_timeline_handle: i32,
    randomise_timer: f32,
    enemy: Enemy,
    enemy_hp_guage: ResourceGuage,
    enemy_timeline_handle: i32
}

struct Player {
    max_fatigue: i32,
    current_fatigue: i32,
    next_action_time: f32
}

struct Enemy {
    max_hp: i32,
    current_hp: i32,
    next_action_time: f32
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

        let player = Player {
            max_fatigue: PLAYER_MAX_FATIGUE,
            current_fatigue: PLAYER_MAX_FATIGUE,
            next_action_time: PLAYER_FIRST_ACTION
        };

        let enemy = Enemy {
            max_hp: ENEMY_MAX_HP,
            current_hp: ENEMY_MAX_HP,
            next_action_time: ENEMY_FIRST_ACTION
        };

        let mut timeline = ActionTimeline::new();

        let enemy_timeline_handle = timeline.add_subject(
            palette::RED,
            enemy.next_action_time
        );

        let player_timeline_handle = timeline.add_subject(
            palette::GREEN,
            player.next_action_time
        );

        let s = MainState {
            font: font,
            action_time: 0.0,
            timeline: timeline,
            player_fatigue_guage: ResourceGuage::new(
                player.max_fatigue as f32,
                player.current_fatigue as f32,
                palette::GREEN
            ),
            player_balance: BalanceGuage::new(0.0),
            player: player,
            player_attack_pending: false,
            player_timeline_handle: player_timeline_handle,
            pos_x: 0.0,
            randomise_timer: 0.0,
            enemy_hp_guage: ResourceGuage::new(
                enemy.max_hp as f32,
                enemy.current_hp as f32,
                palette::RED
            ),
            enemy_timeline_handle: enemy_timeline_handle,
            enemy: enemy,
        };

        Ok(s)
    }
}

impl event::EventHandler for MainState {

    fn text_input_event(&mut self, _ctx: &mut ggez::Context, character: char) {
        if character == '1' && self.player_attack_pending {
            self.player_attack_pending = false;
            self.player.next_action_time = self.action_time + ATTACK_ACTION_TIME;
            self.enemy.current_hp -= ATTACK_DAMAGE;

            self.enemy_hp_guage.update(self.enemy.current_hp as f32);
            self.timeline.update_subject(self.player_timeline_handle, self.player.next_action_time);
        }
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let delta = 1.0 / (DESIRED_FPS as f32);

            self.randomise_timer += delta;

            self.action_time += ACTION_POINTS_PER_SECOND * delta;
            self.timeline.update(self.action_time);

            if self.action_time > self.enemy.next_action_time {
                // Enemy attack
                self.player.current_fatigue -= ATTACK_DAMAGE;
                self.enemy.next_action_time = self.action_time + ATTACK_ACTION_TIME;

                self.player_fatigue_guage.update(self.player.current_fatigue as f32);
                self.timeline.update_subject(self.enemy_timeline_handle, self.enemy.next_action_time);
            }

            if self.action_time > self.player.next_action_time {
                // Player attack
                self.player_attack_pending = true;
            }

            balance_guage::update(&mut self.player_balance, delta);
            resource_guage::update(&mut self.player_fatigue_guage, delta);
            resource_guage::update(&mut self.enemy_hp_guage, delta);

            if self.randomise_timer > RANDOMISE_INTERVAL {
                self.randomise_timer = self.randomise_timer % RANDOMISE_INTERVAL;
                self.player_balance.update(random::<f32>());
            }

            self.pos_x = self.pos_x % 800.0 + 1.0;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let mut hello_world = graphics::Text::new("HELLO WORLD");

        hello_world.set_font(self.font, graphics::Scale::uniform(graphics::DEFAULT_FONT_SCALE * 2.0));

        let player_fatigue_guage = resource_guage::create_mesh(ctx, &self.player_fatigue_guage)?;
        let player_balance_guage = balance_guage::create_mesh(ctx, &self.player_balance)?;

        let enemy_hp_guage = resource_guage::create_mesh(ctx, &self.enemy_hp_guage)?;

        graphics::draw(ctx, &hello_world, (Point2::new(100.0, 0.0),))?;

        graphics::draw(ctx, &enemy_hp_guage, (Point2::new(600.0, 50.0),))?;

        graphics::draw(ctx, &player_fatigue_guage, (Point2::new(100.0, 500.0),))?;

        graphics::draw(ctx, &player_balance_guage, (Point2::new(100.0, 530.0),))?;

        if self.player_attack_pending {
            let player_highlight = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::stroke(2.0),
                graphics::Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 120.0,
                    h: 70.0
                },
                palette::YELLOW
            )?;
            graphics::draw(ctx, &player_highlight, (Point2::new(90.0, 490.0),))?;
        }

        let timeline_mesh = action_timeline::create_mesh(ctx, &self.timeline)?;

        graphics::draw(ctx, &timeline_mesh, (Point2::new(200.0, 400.0),))?;

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