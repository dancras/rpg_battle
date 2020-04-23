use ggez::event::{KeyCode};

#[derive(Default)]
pub struct MoveState {
    up: bool,
    right: bool,
    down: bool,
    left: bool
}

impl MoveState {
    pub fn handle_key_down(&mut self, keycode: &KeyCode) {
        match keycode {
            KeyCode::W => self.up = true,
            KeyCode::A => self.left = true,
            KeyCode::S => self.down = true,
            KeyCode::D => self.right = true,
            _ => {}
        };
    }

    pub fn handle_key_up(&mut self, keycode: &KeyCode) {
        match keycode {
            KeyCode::W => self.up = false,
            KeyCode::A => self.left = false,
            KeyCode::S => self.down = false,
            KeyCode::D => self.right = false,
            _ => {}
        };
    }

    pub fn get_move(&self) -> Move {
        if self.up && self.down || self.left && self.right {
            Move::None
        } else if self.up {
            if self.left {
                Move::UpLeft
            } else if self.right {
                Move::UpRight
            } else {
                Move::Up
            }
        } else if self.down {
            if self.left {
                Move::DownLeft
            } else if self.right {
                Move::DownRight
            } else {
                Move::Down
            }
        } else {
            Move::None
        }
    }
}

pub enum Move {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
    None
}
