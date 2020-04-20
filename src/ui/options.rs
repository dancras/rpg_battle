use ggez::{Context, GameResult};
use ggez::graphics::{self, Mesh, MeshBuilder};

use crate::palette::{self, darker};
use crate::projector::{Projector};

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
        y: f32,
        project: &Projector
    ) -> i16 {
        if y > 0.0 && y < project.scale(20.0) && x > 0.0 && x % project.scale(30.0) < project.scale(20.0) {
            let possible_option = x as i32 / project.scale(30.0) as i32;
            if possible_option < self.options as i32 {
                self.current_option = possible_option as i16;
                self.cached_mesh = None;
            }
        }

        self.current_option
    }

    pub fn draw(&mut self, ctx: &mut Context, project: &Projector) -> GameResult {

        let mesh_result = match self.cached_mesh.take() {
            Some(m) => Ok(m),
            None => {
                let local_project = project.local();
                let mut inputs = &mut MeshBuilder::new();

                for i in 0..self.options {
                    inputs = inputs.circle(
                        graphics::DrawMode::stroke(2.0),
                        local_project.coords(10.0 + 30.0 * i as f32, 10.0),
                        project.scale(10.0),
                        0.5,
                        graphics::WHITE
                    );

                    inputs = inputs.circle(
                        graphics::DrawMode::fill(),
                        local_project.coords(10.0 + 30.0 * i as f32, 10.0),
                        project.scale(7.0),
                        0.5,
                        darker(graphics::WHITE)
                    );

                    if i == self.current_option {
                        inputs = inputs.circle(
                            graphics::DrawMode::fill(),
                            local_project.coords(10.0 + 30.0 * i as f32, 10.0),
                            project.scale(5.0),
                            0.5,
                            palette::BLUE
                        );
                    }
                }

                inputs.build(ctx)
            }
        };

        let mesh = mesh_result?;

        graphics::draw(ctx, &mesh, (project.origin(),))?;
        self.cached_mesh = Some(mesh);

        Ok(())

    }
}
