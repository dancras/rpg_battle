use ggez;
use ggez::event;
use ggez::graphics::{self, Color};
use ggez::input::mouse::{MouseButton};
use ggez::nalgebra::{Point2};
use ggez::timer;
use rand::{random};
use rpg_battle::palette;
use rpg_battle::hud::action_timeline::{self, ActionTimeline};
use rpg_battle::hud::resource_guage::{self, ResourceGuage};
use rpg_battle::hud::balance_guage::{self, BalanceGuage};
use std::cmp;

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

// TODO implement death / exhaustion
//  - remove from timeline and swap enemy target to living
//  - don't allow attack if dead enemy targeted
//  - reset when either party is completely dead
struct MainState {
    font: graphics::Font,
    randomise_timer: f32,
    battle: BattleState
}

struct Player {
    color: Color,
    max_fatigue: i32,
    current_fatigue: i32,
    current_balance: f32,
    next_action_time: f32
}

struct PlayerInBattle {
    stats: Player,
    fatigue_guage: ResourceGuage,
    balance_guage: BalanceGuage,
    timeline_handle: i32
}

impl PlayerInBattle {
    fn new(player: Player, timeline: &mut ActionTimeline) -> Self {
        Self {
            fatigue_guage: ResourceGuage::new(
                player.max_fatigue as f32,
                player.current_fatigue as f32,
                player.color
            ),
            balance_guage: BalanceGuage::new(player.current_balance),
            timeline_handle: timeline.add_subject(
                player.color,
                player.next_action_time
            ),
            stats: player
        }
    }
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

struct BattleState {
    action_time: f32,
    timeline: ActionTimeline,
    players: Vec<PlayerInBattle>,
    players_pending: Vec<usize>,
    enemies: Vec<EnemyInBattle>,
    target_enemy: usize
}

impl BattleState {
    fn new() -> Self {
        let mut timeline = ActionTimeline::new();

        Self {
            action_time: 0.0,
            players: vec![
                PlayerInBattle::new(
                    Player {
                        color: palette::GREEN,
                        max_fatigue: PLAYER_MAX_FATIGUE,
                        current_fatigue: PLAYER_MAX_FATIGUE,
                        current_balance: calculate_balance(),
                        next_action_time: PLAYER_FIRST_ACTION
                    },
                    &mut timeline
                ),
                PlayerInBattle::new(
                    Player {
                        color: palette::BLUE,
                        max_fatigue: PLAYER_MAX_FATIGUE,
                        current_fatigue: PLAYER_MAX_FATIGUE,
                        current_balance: calculate_balance(),
                        next_action_time: PLAYER_FIRST_ACTION
                    },
                    &mut timeline
                )
            ],
            players_pending: Vec::new(),
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
            target_enemy: 0,
            timeline: timeline
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

fn has_item<T: PartialEq>(list: &Vec<T>, search_item: &T) -> bool {
    for item in list {
        if item == search_item {
            return true;
        }
    }
    return false;
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
            battle: BattleState::new()
        };

        Ok(s)
    }

    fn check_for_surviving_enemies(&mut self) {
        for enemy in &self.battle.enemies {
            if enemy.stats.current_hp > 0 {
                return;
            }
        }

        self.battle = BattleState::new();
    }
}

impl event::EventHandler for MainState {

    fn text_input_event(&mut self, _ctx: &mut ggez::Context, character: char) {
        if character == '1' && self.battle.players_pending.len() > 0 {
            let attacking_player_index = self.battle.players_pending.remove(0);
            let attacking_player = &mut self.battle.players[attacking_player_index];
            attacking_player.stats.next_action_time = self.battle.action_time + ATTACK_ACTION_TIME;
            let dmg = calculate_balance_dmg(ATTACK_DAMAGE, attacking_player.stats.current_balance);
            let enemy = &mut self.battle.enemies[self.battle.target_enemy];
            enemy.stats.current_hp -= dmg;
            enemy.stats.current_hp = cmp::max(0, enemy.stats.current_hp);

            attacking_player.stats.current_fatigue -= ATTACK_FATIGUE_COST;
            attacking_player.stats.current_balance = calculate_balance();

            attacking_player.balance_guage.update(attacking_player.stats.current_balance);
            attacking_player.fatigue_guage.update(attacking_player.stats.current_fatigue as f32);
            enemy.hp_guage.update(enemy.stats.current_hp as f32);
            self.battle.timeline.update_subject(attacking_player.timeline_handle, attacking_player.stats.next_action_time);

            if enemy.stats.current_hp == 0 {
                self.battle.timeline.remove_subject(enemy.timeline_handle);

                self.battle.target_enemy = if self.battle.target_enemy == 0 { 1 } else { 0 };

                self.check_for_surviving_enemies();
            }
        }
    }

    fn mouse_button_down_event(
        &mut self, _ctx: &mut ggez::Context, _button: MouseButton, x: f32, y: f32
    ) {
        if y < 110.0 {
            if x > 600.0 && self.battle.enemies[0].stats.current_hp > 0 {
                self.battle.target_enemy = 0;
            } else if x > 460.0 && self.battle.enemies[1].stats.current_hp > 0 {
                self.battle.target_enemy = 1;
            }
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
        self.battle.timeline.highlighted_subject = None;

        if y < 110.0 {
            if x > 600.0 && self.battle.enemies[0].stats.current_hp > 0 {
                self.battle.timeline.highlighted_subject = Some(self.battle.enemies[0].timeline_handle);
            } else if x > 460.0 && self.battle.enemies[1].stats.current_hp > 0 {
                self.battle.timeline.highlighted_subject = Some(self.battle.enemies[1].timeline_handle);
            }
        }
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let delta = 1.0 / (DESIRED_FPS as f32);

            self.randomise_timer += delta;

            self.battle.action_time += ACTION_POINTS_PER_SECOND * delta;
            self.battle.timeline.update(self.battle.action_time);

            for enemy in &mut self.battle.enemies {
                if enemy.stats.current_hp > 0 && self.battle.action_time > enemy.stats.next_action_time {
                    // Enemy attack
                    let target_player = &mut self.battle.players[0];
                    let dmg = calculate_balance_dmg(ATTACK_DAMAGE, enemy.stats.current_balance);
                    target_player.stats.current_fatigue -= dmg;
                    enemy.stats.current_balance = calculate_balance();
                    enemy.stats.next_action_time = self.battle.action_time + ATTACK_ACTION_TIME;

                    enemy.balance_guage.update(enemy.stats.current_balance);
                    target_player.fatigue_guage.update(target_player.stats.current_fatigue as f32);
                    self.battle.timeline.update_subject(enemy.timeline_handle, enemy.stats.next_action_time);
                }
            }

            for (i, player) in self.battle.players.iter_mut().enumerate() {
                if self.battle.action_time > player.stats.next_action_time {
                    // Queue up player for attack
                    if !has_item(&self.battle.players_pending, &i) {
                        self.battle.players_pending.push(i);
                    }
                }

                balance_guage::update(&mut player.balance_guage, delta);
                resource_guage::update(&mut player.fatigue_guage, delta);
            }

            for enemy in &mut self.battle.enemies {
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

        graphics::draw(ctx, &hello_world, (Point2::new(100.0, 0.0),))?;

        let mut enemy_display_offset = 0.0;
        for enemy in &self.battle.enemies {
            let enemy_hp_guage = resource_guage::create_mesh(ctx, &enemy.hp_guage)?;
            let enemy_balance_guage = balance_guage::create_mesh(ctx, &enemy.balance_guage)?;
            graphics::draw(ctx, &enemy_hp_guage, (Point2::new(600.0 - enemy_display_offset, 50.0),))?;
            graphics::draw(ctx, &enemy_balance_guage, (Point2::new(600.0 - enemy_display_offset, 80.0),))?;
            enemy_display_offset += 140.0;
        }

        let enemy_highlight = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: 120.0,
                h: 70.0
            },
            palette::GREY
        )?;
        graphics::draw(ctx, &enemy_highlight, (Point2::new(590.0 - 140.0 * self.battle.target_enemy as f32, 40.0),))?;

        let mut player_display_offset = 0.0;
        for player in &self.battle.players {
            let player_fatigue_guage = resource_guage::create_mesh(ctx, &player.fatigue_guage)?;
            let player_balance_guage = balance_guage::create_mesh(ctx, &player.balance_guage)?;

            graphics::draw(ctx, &player_fatigue_guage, (Point2::new(100.0 + player_display_offset, 500.0),))?;
            graphics::draw(ctx, &player_balance_guage, (Point2::new(100.0 + player_display_offset, 530.0),))?;
            player_display_offset += 140.0;
        }

        if self.battle.players_pending.len() > 0 {
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
            let player_highlight_offset = self.battle.players_pending[0] as f32 * 140.0;
            graphics::draw(ctx, &player_highlight, (Point2::new(90.0 + player_highlight_offset, 490.0),))?;
        }

        let timeline_mesh = action_timeline::create_mesh(ctx, &self.battle.timeline)?;

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
