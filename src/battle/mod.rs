use ggez::graphics::{self, Color};
use rand::{random};
use std::cmp;

use crate::palette;
use crate::projector::{Projector};
use crate::hud::action_frame::{ActionFrame};
use crate::hud::action_hotbar;
use crate::hud::action_timeline::{self, ActionTimeline};
use crate::hud::resource_guage::{self, ResourceGuage};
use crate::hud::balance_guage::{self, BalanceGuage};

const PLAYER_MAX_FATIGUE: i32 = 100;
const PLAYER_FIRST_ACTION: f32 = 50.0;
const ENEMY_MAX_HP: i32 = 50;
const ENEMY_FIRST_ACTION: f32 = 100.0;
const ACTION_POINTS_PER_SECOND: f32 = 60.0;
const ATTACK_DAMAGE: i32 = 10;
const ATTACK_ACTION_TIME: f32 = 250.0;
const ATTACK_FATIGUE_COST: i32 = 5;
const BLOCK_ACTION_TIME: f32 = 300.0;
const BLOCK_FATIGUE_COST: i32 = 5;
const BLOCK_HIT_TIME_PENALTY: f32 = 100.0;

struct Player {
    color: Color,
    max_fatigue: i32,
    current_fatigue: i32,
    current_balance: f32,
    next_action_time: f32,
    is_blocking: bool,
    block_end_time: f32
}

struct PlayerInBattle {
    stats: Player,
    fatigue_guage: ResourceGuage,
    balance_guage: BalanceGuage,
    timeline_handle: i32,
    action_frame: ActionFrame
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
            action_frame: ActionFrame::new(player.color),
            stats: player
        }
    }
}

pub struct Enemy {
    max_hp: i32,
    pub current_hp: i32,
    current_balance: f32,
    next_action_time: f32
}

pub struct EnemyInBattle {
    pub stats: Enemy,
    hp_guage: ResourceGuage,
    balance_guage: BalanceGuage,
    pub timeline_handle: i32,
    action_frame: ActionFrame
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
            stats: enemy,
            action_frame: ActionFrame::new(palette::RED)
        }
    }
}

pub struct BattleState {
    action_time: f32,
    pub timeline: ActionTimeline,
    players: Vec<PlayerInBattle>,
    players_pending: Vec<usize>,
    pub enemies: Vec<EnemyInBattle>,
    pub hovered_enemy: Option<usize>,
    pub target_enemy: usize
}

impl BattleState {

    pub fn handle_mouse_move(&mut self, x: f32, y: f32, projector: &Projector) {
        self.timeline.highlighted_subject = None;
        self.hovered_enemy = None;

        if y < projector.scale(70.0) {
            for (i, enemy) in self.enemies.iter().enumerate().rev() {
                if projector.top_right((i + 1) as f32 * 140.0).to_local_x(x) > 0.0 && enemy.stats.current_hp > 0 {
                    self.hovered_enemy = Some(i);
                    self.timeline.highlighted_subject = Some(enemy.timeline_handle);
                }
            }
        }
    }

    pub fn tick<F: FnMut(BattleEvents)>(&mut self, delta: f32, mut notify: F) {

        self.action_time += ACTION_POINTS_PER_SECOND * delta;
        self.timeline.update(self.action_time);

        for enemy in &mut self.enemies {

            enemy.action_frame.update_time(self.action_time / ACTION_POINTS_PER_SECOND);

            if enemy.stats.current_hp > 0 && self.action_time > enemy.stats.next_action_time {

                // Enemy attack
                let mut target_player_index = 0;
                while self.players.len() > target_player_index + 1 &&
                    self.players[target_player_index].stats.current_fatigue == 0 {
                    target_player_index += 1;
                }
                let target_player = &mut self.players[target_player_index];

                let mut dmg = calculate_balance_dmg(ATTACK_DAMAGE, enemy.stats.current_balance);

                if target_player.stats.is_blocking {
                    dmg = dmg / 4;
                    target_player.stats.next_action_time += BLOCK_HIT_TIME_PENALTY;
                    target_player.action_frame.activate("Block");
                    self.timeline.update_subject(target_player.timeline_handle, target_player.stats.next_action_time);
                }

                target_player.stats.current_fatigue -= dmg;
                target_player.stats.current_fatigue = cmp::max(0, target_player.stats.current_fatigue);
                notify(BattleEvents::PlayerTakesDamage(target_player_index));

                enemy.stats.current_balance = calculate_balance();
                enemy.stats.next_action_time = self.action_time + ATTACK_ACTION_TIME;

                enemy.action_frame.activate("Attack");
                enemy.balance_guage.update(enemy.stats.current_balance);

                self.timeline.update_subject(enemy.timeline_handle, enemy.stats.next_action_time);
            }
        }

        for (i, player) in self.players.iter_mut().enumerate() {

            player.action_frame.update_time(self.action_time / ACTION_POINTS_PER_SECOND);

            if player.stats.current_fatigue > 0 && self.action_time > player.stats.next_action_time {
                // Queue up player for attack
                if !has_item(&self.players_pending, &i) {
                    self.players_pending.push(i);
                }
            }

            if player.stats.is_blocking && self.action_time > player.stats.block_end_time {
                player.stats.is_blocking = false;
            }

            balance_guage::update(&mut player.balance_guage, delta);
            resource_guage::update(&mut player.fatigue_guage, delta);
        }

        for enemy in &mut self.enemies {
            balance_guage::update(&mut enemy.balance_guage, delta);
            resource_guage::update(&mut enemy.hp_guage, delta);
        }

    }

    pub fn handle_event<F: FnMut(BattleEvents)>(&mut self, event: &BattleEvents, mut notify: F) {

        match event {
            BattleEvents::End => {},
            BattleEvents::EnemyTakesDamage(i) => {
                let enemy = &mut self.enemies[*i];

                enemy.hp_guage.update(enemy.stats.current_hp as f32);

                if enemy.stats.current_hp == 0 {
                    self.timeline.remove_subject(enemy.timeline_handle);

                    self.target_enemy = 0;

                    while self.target_enemy < self.enemies.len() &&
                          self.enemies[self.target_enemy].stats.current_hp == 0 {
                        self.target_enemy += 1;
                    }
                }

                if self.target_enemy == self.enemies.len() {
                    notify(BattleEvents::End);
                }
            },
            BattleEvents::PlayerTakesDamage(i) => {
                let player = &mut self.players[*i];

                player.fatigue_guage.update(player.stats.current_fatigue as f32);

                if player.stats.current_fatigue == 0 {
                    self.players_pending.retain(|j| j != i);
                    self.timeline.remove_subject(player.timeline_handle);

                    if !self.any_surviving_players() {
                        notify(BattleEvents::End);
                    }
                }
            }
        }
    }

    pub fn new() -> Self {
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
                        next_action_time: PLAYER_FIRST_ACTION,
                        is_blocking: false,
                        block_end_time: 0.0
                    },
                    &mut timeline
                ),
                PlayerInBattle::new(
                    Player {
                        color: palette::BLUE,
                        max_fatigue: PLAYER_MAX_FATIGUE,
                        current_fatigue: PLAYER_MAX_FATIGUE,
                        current_balance: calculate_balance(),
                        next_action_time: PLAYER_FIRST_ACTION,
                        is_blocking: false,
                        block_end_time: 0.0
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
            hovered_enemy: None,
            target_enemy: 0,
            timeline: timeline
        }
    }

    fn any_surviving_players(&mut self) -> bool{
        for player in &self.players {
            if player.stats.current_fatigue > 0 {
                return true;
            }
        }

        false
    }

    pub fn player_move_pending(&self) -> bool {
        self.players_pending.len() > 0
    }

    pub fn player_attack_move<F: FnMut(BattleEvents)>(&mut self, mut notify: F) {
        let attacking_player_index = self.players_pending.remove(0);
        let attacking_player = &mut self.players[attacking_player_index];
        attacking_player.stats.next_action_time = self.action_time + ATTACK_ACTION_TIME;
        let dmg = calculate_balance_dmg(ATTACK_DAMAGE, attacking_player.stats.current_balance);
        let enemy = &mut self.enemies[self.target_enemy];
        enemy.stats.current_hp -= dmg;
        enemy.stats.current_hp = cmp::max(0, enemy.stats.current_hp);
        notify(BattleEvents::EnemyTakesDamage(self.target_enemy));

        attacking_player.stats.current_fatigue -= ATTACK_FATIGUE_COST;
        attacking_player.stats.current_balance = calculate_balance();

        notify(BattleEvents::PlayerTakesDamage(attacking_player_index));
        attacking_player.balance_guage.update(attacking_player.stats.current_balance);

        attacking_player.action_frame.activate("Attack");
        self.timeline.update_subject(attacking_player.timeline_handle, attacking_player.stats.next_action_time);
    }

    pub fn player_block_move<F: FnMut(BattleEvents)>(&mut self, mut notify: F) {
        let attacking_player_index = self.players_pending.remove(0);
        let attacking_player = &mut self.players[attacking_player_index];
        attacking_player.stats.next_action_time = self.action_time + BLOCK_ACTION_TIME;

        attacking_player.stats.is_blocking = true;
        attacking_player.stats.block_end_time = attacking_player.stats.next_action_time;

        attacking_player.stats.current_fatigue -= BLOCK_FATIGUE_COST;
        attacking_player.stats.current_balance = calculate_balance();

        notify(BattleEvents::PlayerTakesDamage(attacking_player_index));
        attacking_player.balance_guage.update(attacking_player.stats.current_balance);

        self.timeline.update_subject(attacking_player.timeline_handle, attacking_player.stats.next_action_time);
    }

    pub fn draw(&mut self, ctx: &mut ggez::Context, projector: &Projector) -> ggez::GameResult {

        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            draw_enemy_display(
                ctx,
                enemy,
                &projector.top_right((i + 1) as f32 * 140.0),
                i == self.target_enemy
            )?;
        }

        if self.players_pending.len() > 0 {
            action_hotbar::draw(
                ctx,
                &projector.bottom_left(150.0).centered_horizontal(490.0)
            )?;
        }

        let mut player_display_offset = 0.0;
        for (i, player) in self.players.iter_mut().enumerate() {
            draw_player_display(
                ctx,
                player,
                &projector.bottom_left(90.0)
                    .local_relative(90.0 + player_display_offset, 0.0),
                self.players_pending.len() > 0 && self.players_pending[0] == i
            )?;
            player_display_offset += 140.0;
        }

        let timeline_mesh = action_timeline::create_mesh(
            ctx,
            &self.timeline,
            &projector.local()
        )?;

        graphics::draw(
            ctx,
            &timeline_mesh,
            (projector.bottom_left(170.0).centered_horizontal(400.0).origin(),)
        )?;

        Ok(())
    }
}

pub enum BattleEvents {
    End,
    EnemyTakesDamage(usize),
    PlayerTakesDamage(usize)
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


fn draw_enemy_display(
    ctx: &mut ggez::Context,
    enemy: &EnemyInBattle,
    project: &Projector,
    is_highlighted: bool
) -> ggez::GameResult {
    let enemy_hp_guage = resource_guage::create_mesh(ctx, &enemy.hp_guage, &project.local())?;
    let enemy_balance_guage = balance_guage::create_mesh(ctx, &enemy.balance_guage, &project.local())?;
    graphics::draw(ctx, &enemy_hp_guage, (project.coords(10.0, 10.0),))?;
    graphics::draw(ctx, &enemy_balance_guage, (project.coords(10.0, 40.0),))?;

    if is_highlighted {
        let enemy_highlight = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: project.scale(120.0),
                h: project.scale(70.0)
            },
            palette::GREY
        )?;
        graphics::draw(
            ctx,
            &enemy_highlight,
            (project.origin(),)
        )?;
    }

    enemy.action_frame.draw(ctx, &project.local_relative(30.0, 80.0))?;

    Ok(())
}

fn draw_player_display(
    ctx: &mut ggez::Context,
    player: &PlayerInBattle,
    project: &Projector,
    is_highlighted: bool
) -> ggez::GameResult {

    let player_fatigue_guage = resource_guage::create_mesh(ctx, &player.fatigue_guage, &project.local())?;
    let player_balance_guage = balance_guage::create_mesh(ctx, &player.balance_guage, &project.local())?;

    graphics::draw(ctx, &player_fatigue_guage, (project.coords(10.0, 10.0),))?;
    graphics::draw(ctx, &player_balance_guage, (project.coords(10.0, 40.0),))?;

    if is_highlighted {
        let player_highlight = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: project.scale(120.0),
                h: project.scale(70.0)
            },
            palette::YELLOW
        )?;
        graphics::draw(
            ctx,
            &player_highlight,
            (project.origin(),)
        )?;
    }

    if player.stats.is_blocking {
        let block_icon = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: project.scale(10.0),
                h: project.scale(10.0)
            },
            graphics::WHITE
        )?;
        graphics::draw(ctx, &block_icon, (project.coords(10.0, 80.0),))?;
    }

    player.action_frame.draw(ctx, &project.local_relative(30.0, -70.0))?;

    Ok(())
}
