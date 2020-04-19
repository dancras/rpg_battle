use ggez::{Context, GameResult};
use ggez::graphics::{self, Mesh, MeshBuilder};
use ggez::nalgebra::{Point2};

use crate::palette::{self, darker};

pub struct Options {
    options: i16,
    current_option: i16,
    cached_mesh: Option<Mesh>
}

impl Options {

    pub fn new(options: i16, default_option: i16) -> Self {
        Self {
            options: options,
            current_option: default_option,
            cached_mesh: None
        }
    }

    pub fn handle_mouse_down(
        &mut self,
        x: f32,
        y: f32
    ) -> i16 {
        if y > 0.0 && y < 20.0 && x > 0.0 && x % 30.0 < 20.0 {
            let possible_option = x as i32 / 30;
            if possible_option < self.options as i32 {
                self.current_option = possible_option as i16;
                self.cached_mesh = None;
            }
        }

        self.current_option
    }

    pub fn draw(&mut self, ctx: &mut Context, position: Point2<f32>) -> GameResult {

        let mesh_result = match self.cached_mesh.take() {
            Some(m) => Ok(m),
            None => {
                let mut inputs = &mut MeshBuilder::new();

                for i in 0..self.options {
                    inputs = inputs.circle(
                        graphics::DrawMode::stroke(2.0),
                        Point2::new(10.0 + 30.0 * i as f32, 10.0),
                        10.0,
                        0.5,
                        graphics::WHITE
                    );

                    inputs = inputs.circle(
                        graphics::DrawMode::fill(),
                        Point2::new(10.0 + 30.0 * i as f32, 10.0),
                        7.0,
                        0.5,
                        darker(graphics::WHITE)
                    );

                    if i == self.current_option {
                        inputs = inputs.circle(
                            graphics::DrawMode::fill(),
                            Point2::new(10.0 + 30.0 * i as f32, 10.0),
                            5.0,
                            0.5,
                            palette::BLUE
                        );
                    }
                }

                inputs.build(ctx)
            }
        };

        let mesh = mesh_result?;

        graphics::draw(ctx, &mesh, (position,))?;
        self.cached_mesh = Some(mesh);

        Ok(())

    }
}
