use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, Font};
use nalgebra::{Vector2};

use crate::palette::{darker};
use crate::projector::{ProjectorTopLeft};

const FIRST_FRAME_DURATION: f32 = 0.4;
const SECOND_FRAME_DURATION: f32 = 0.2;

pub struct ActionFrame {
    time: f32,
    first_frame_timeout: f32,
    second_frame_timeout: f32,
    frame_text: String,
    frame_color: Color
}

impl ActionFrame {
    pub fn new(color: Color) -> Self {
        Self {
            time: 0.0,
            first_frame_timeout: 0.0,
            second_frame_timeout: 0.0,
            frame_text: "".to_string(),
            frame_color: color
        }
    }

    pub fn update_time(&mut self, current_time: f32) {
        self.time = current_time;
    }

    pub fn activate<T: Into<String>>(&mut self, text: T) {
        self.first_frame_timeout = self.time + FIRST_FRAME_DURATION;
        self.second_frame_timeout = self.first_frame_timeout + SECOND_FRAME_DURATION;
        self.frame_text = text.into();
    }

    pub fn draw(&self, ctx: &mut Context, project: &ProjectorTopLeft) -> GameResult {

        if self.time < self.first_frame_timeout {
            self.draw_first_frame(ctx, project)?;
        } else if self.time < self.second_frame_timeout {
            self.draw_second_frame(ctx, project)?;
        }

        Ok(())
    }

    fn draw_first_frame(&self, ctx: &mut Context, project: &ProjectorTopLeft) -> GameResult {
        self.draw_plain_frame(ctx, self.frame_color, project)?;
        Ok(())
    }

    fn draw_second_frame(&self, ctx: &mut Context, project: &ProjectorTopLeft) -> GameResult {
        self.draw_plain_frame(ctx, darker(self.frame_color), project)?;

        let mut text = graphics::Text::new(self.frame_text.clone());
        text.set_font(Font::default(), graphics::Scale::uniform(project.scale(graphics::DEFAULT_FONT_SCALE) * 0.8));
        let half_height = (text.height(ctx) / 2) as f32;
        let centering_offset = (project.scale(60.0) - text.width(ctx) as f32) / 2.0;
        graphics::draw(
            ctx,
            &text,
            (project.origin() + Vector2::new(centering_offset, project.scale(30.0) - half_height),)
        )?;

        Ok(())
    }

    fn draw_plain_frame(&self, ctx: &mut Context, color: Color, project: &ProjectorTopLeft) -> GameResult {
        let frame = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: project.scale(60.0),
                h: project.scale(60.0)
            },
            color
        )?;
        let frame_border = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: project.scale(60.0),
                h: project.scale(60.0)
            },
            graphics::WHITE
        )?;
        graphics::draw(ctx, &frame, (project.origin(),))?;
        graphics::draw(ctx, &frame_border, (project.origin(),))?;

        Ok(())
    }
}
