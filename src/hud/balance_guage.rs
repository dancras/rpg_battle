use ezing;
use ggez::{Context, GameResult};
use ggez::graphics::{self, Mesh, MeshBuilder};

use crate::projector::{Projector};

const ANIMATION_DURATION: f32 = 0.8;

pub struct BalanceGuage {
    value: f32,
    display_value: f32,
    previous_value: f32,
    delta: f32,
}

impl BalanceGuage {
    pub fn new(initial_value: f32) -> BalanceGuage {
        BalanceGuage {
            value: initial_value,
            display_value: initial_value,
            previous_value: initial_value,
            delta: ANIMATION_DURATION
        }
    }

    pub fn update(&mut self, new_value: f32) {
        self.previous_value = self.value;
        self.value = new_value;
        self.delta = 0.0;
    }
}

pub fn update(viewmodel: &mut BalanceGuage, delta: f32) {

    if viewmodel.delta < ANIMATION_DURATION {
        viewmodel.delta += delta;

        viewmodel.display_value = viewmodel.previous_value +
            (viewmodel.value - viewmodel.previous_value) * ezing::sine_inout(viewmodel.delta / 0.8);
    }

}

pub fn create_mesh(ctx: &mut Context, viewmodel: &BalanceGuage, project: &Projector) -> GameResult<Mesh> {

    let offset = 92.0 * viewmodel.display_value;

    MeshBuilder::new()
        .rectangle(
            graphics::DrawMode::stroke(2.0),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: project.scale(100.0),
                h: project.scale(20.0)
            },
            graphics::WHITE,
        )
        .triangles(
            &[
                project.coords(offset + 4.0, 13.0),
                project.coords(offset + 8.0, 20.0),
                project.coords(offset, 20.0),
            ],
            graphics::WHITE,
        )?
        .build(ctx)
}
