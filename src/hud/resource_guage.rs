use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, Mesh, MeshBuilder};

use crate::projector::{ProjectorTopLeft};

const ANIMATION_DURATION: f32 = 0.8;

pub struct ResourceGuage {
    color: Color,
    max_value: f32,
    value: f32,
    display_value: f32,
    previous_value: f32,
    delta: f32,
}

impl ResourceGuage {
    pub fn new(max_value: f32, initial_value: f32, color: Color) -> ResourceGuage {
        ResourceGuage {
            color: color,
            max_value: max_value,
            value: initial_value,
            display_value: initial_value,
            previous_value: initial_value,
            delta: ANIMATION_DURATION
        }
    }

    pub fn update(&mut self, new_value: f32) {
        self.previous_value = self.display_value;
        self.value = new_value;
        self.delta = 0.0;
    }
}

pub fn update(viewmodel: &mut ResourceGuage, delta: f32) {

    if viewmodel.delta < ANIMATION_DURATION {
        viewmodel.delta += delta;

        viewmodel.display_value = viewmodel.previous_value +
            (viewmodel.value - viewmodel.previous_value) * ezing::sine_inout(viewmodel.delta / 0.8);
    }

}

pub fn create_mesh(ctx: &mut Context, viewmodel: &ResourceGuage, project: &ProjectorTopLeft) -> GameResult<Mesh> {
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
        .rectangle(
            graphics::DrawMode::fill(),
            graphics::Rect {
                x: project.scale(4.0),
                y: project.scale(4.0),
                w: project.scale(92.0 * (viewmodel.display_value / viewmodel.max_value)),
                h: project.scale(12.0)
            },
            viewmodel.color,
        )
        .build(ctx)
}
