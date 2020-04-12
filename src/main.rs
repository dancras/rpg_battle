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
const ATTACK_FATIGUE_COST: i32 = 5;

// TODO multiple enemies
//  - ability to select target enemy
//  - hover enemy highlights position in action timeline
// TODO multiple player characters
//  - queue up turns if they are both pending
struct MainState {
    font: graphics::Font,
    action_time: f32,
    timeline: ActionTimeline,
    player: Player,
    player_fatigue_guage: ResourceGuage,
    player_balance_guage: BalanceGuage,
    player_attack_pending: bool,
    player_timeline_handle: i32,
    randomise_timer: f32,
    enemies: Vec<EnemyInBattle>
}

struct Player {
    max_fatigue: i32,
    current_fatigue: i32,
    current_balance: f32,
    next_action_time: f32
}

struct Enemy {
    max_hp: i32,
    current_hp: i32,
    current_balance: f32,
    next_action_time: f32
}

struct EnemyInBattle {
    stats: Enemy,
    hp_guage: ResourceGuage,
    balance_guage: BalanceGuage,
    timeline_handle: i32
}

impl EnemyInBattle {
    fn new(enemy: Enemy, timeline: &mut ActionTimeline) -> Self {
        EnemyInBattle {
            hp_guage: ResourceGuage::new(
                enemy.max_hp as f32,
                enemy.current_hp as f32,
                palette::RED
            ),
            balance_guage: BalanceGuage::new(enemy.current_balance),
            timeline_handle: timeline.add_subject(
                palette::RED,
                enemy.next_action_time
            ),
            stats: enemy
        }
    }
}

fn calculate_balance() -> f32 {
    let damage_group = random::<f32>();

    if damage_group < 0.1 {
        random::<f32>() * 0.3
    } else if damage_group > 0.8 {
        random::<f32>() * 0.7 + 0.3
    } else {
        0.3
    }
}

fn calculate_balance_dmg(base_dmg: i32, balance: f32) -> i32 {
    if balance < 0.3 {
        base_dmg - (base_dmg / (1.0 + ezing::linear(balance / 0.3) * 9.0) as i32)
    } else if balance > 0.3 {
        (base_dmg as f32 * (ezing::quad_in((balance - 0.3) / 0.7) * 4.0 + 1.0)) as i32
    } else {
        base_dmg
    }
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
            current_balance: calculate_balance(),
            next_action_time: PLAYER_FIRST_ACTION
        };

        let mut timeline = ActionTimeline::new();

        let player_timeline_handle = timeline.add_subject(
            palette::GREEN,
            player.next_action_time
        );

        let s = MainState {
            font: font,
            action_time: 0.0,
            player_fatigue_guage: ResourceGuage::new(
                player.max_fatigue as f32,
                player.current_fatigue as f32,
                palette::GREEN
            ),
            player_balance_guage: BalanceGuage::new(player.current_balance),
            player: player,
            player_attack_pending: false,
            player_timeline_handle: player_timeline_handle,
            randomise_timer: 0.0,
            enemies: vec![
                EnemyInBattle::new(
                    Enemy {
                        max_hp: ENEMY_MAX_HP,
                        current_hp: ENEMY_MAX_HP,
                        current_balance: calculate_balance(),
                        next_action_time: ENEMY_FIRST_ACTION
                    },
                    &mut timeline
                ),
                EnemyInBattle::new(
                    Enemy {
                        max_hp: ENEMY_MAX_HP,
                        current_hp: ENEMY_MAX_HP,
                        current_balance: calculate_balance(),
                        next_action_time: ENEMY_FIRST_ACTION
                    },
                    &mut timeline
                )
            ],
            timeline: timeline
        };

        Ok(s)
    }
}

impl event::EventHandler for MainState {

    fn text_input_event(&mut self, _ctx: &mut ggez::Context, character: char) {
        if character == '1' && self.player_attack_pending {
            self.player_attack_pending = false;
            self.player.next_action_time = self.action_time + ATTACK_ACTION_TIME;
            let dmg = calculate_balance_dmg(ATTACK_DAMAGE, self.player.current_balance);
            println!("damage dealt {}", dmg);
            let enemy = &mut self.enemies[0];
            enemy.stats.current_hp -= dmg;
            self.player.current_fatigue -= ATTACK_FATIGUE_COST;
            self.player.current_balance = calculate_balance();

            self.player_balance_guage.update(self.player.current_balance);
            self.player_fatigue_guage.update(self.player.current_fatigue as f32);
            enemy.hp_guage.update(enemy.stats.current_hp as f32);
            self.timeline.update_subject(self.player_timeline_handle, self.player.next_action_time);
        }
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let delta = 1.0 / (DESIRED_FPS as f32);

            self.randomise_timer += delta;

            self.action_time += ACTION_POINTS_PER_SECOND * delta;
            self.timeline.update(self.action_time);

            for enemy in &mut self.enemies {
                if self.action_time > enemy.stats.next_action_time {
                    // Enemy attack
                    let dmg = calculate_balance_dmg(ATTACK_DAMAGE, enemy.stats.current_balance);
                    println!("enemy damage dealt {}", dmg);
                    self.player.current_fatigue -= dmg;
                    enemy.stats.current_balance = calculate_balance();
                    enemy.stats.next_action_time = self.action_time + ATTACK_ACTION_TIME;

                    enemy.balance_guage.update(enemy.stats.current_balance);
                    self.player_fatigue_guage.update(self.player.current_fatigue as f32);
                    self.timeline.update_subject(enemy.timeline_handle, enemy.stats.next_action_time);
                }
            }

            if self.action_time > self.player.next_action_time {
                // Player attack
                self.player_attack_pending = true;
            }

            balance_guage::update(&mut self.player_balance_guage, delta);
            resource_guage::update(&mut self.player_fatigue_guage, delta);

            for enemy in &mut self.enemies {
                balance_guage::update(&mut enemy.balance_guage, delta);
                resource_guage::update(&mut enemy.hp_guage, delta);
            }

            if self.randomise_timer > RANDOMISE_INTERVAL {
                self.randomise_timer = self.randomise_timer % RANDOMISE_INTERVAL;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let mut hello_world = graphics::Text::new("HELLO WORLD");

        hello_world.set_font(self.font, graphics::Scale::uniform(graphics::DEFAULT_FONT_SCALE * 2.0));

        let player_fatigue_guage = resource_guage::create_mesh(ctx, &self.player_fatigue_guage)?;
        let player_balance_guage = balance_guage::create_mesh(ctx, &self.player_balance_guage)?;

        graphics::draw(ctx, &hello_world, (Point2::new(100.0, 0.0),))?;

        let mut enemy_display_offset = 0.0;
        for enemy in &self.enemies {
            let enemy_hp_guage = resource_guage::create_mesh(ctx, &enemy.hp_guage)?;
            let enemy_balance_guage = balance_guage::create_mesh(ctx, &enemy.balance_guage)?;
            graphics::draw(ctx, &enemy_hp_guage, (Point2::new(600.0 - enemy_display_offset, 50.0),))?;
            graphics::draw(ctx, &enemy_balance_guage, (Point2::new(600.0 - enemy_display_offset, 80.0),))?;
            enemy_display_offset += 140.0;
        }

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