use ezing;
use ggez::{Context, GameResult};
use ggez::graphics::{self, Mesh, MeshBuilder};
use ggez::nalgebra::{Point2};

const ACTION_POINTS_PER_SECOND: f32 = 20.0;

pub struct ActionTimeline {
    time: f32
}

impl ActionTimeline {
    pub fn new() -> Self {
        Self {
            time: 0.0
        }
    }
}

pub fn update(viewmodel: &mut ActionTimeline, delta: f32) {
    viewmodel.time += ACTION_POINTS_PER_SECOND * delta;
}

pub fn create_mesh(ctx: &mut Context, viewmodel: &ActionTimeline) -> GameResult<Mesh> {

    let mut ruler = &mut MeshBuilder::new();
    let offset = viewmodel.time % 10.0;

    ruler = ruler.line(
        &[
            Point2::new(0.0, 0.0),
            Point2::new(400.0, 0.0)
        ],
        2.0,
        graphics::WHITE
    )?;

    for i in 0..41 {

        if i == 0 && offset > 0.0 {
            continue;
        }

        ruler = ruler.line(
            &[
                Point2::new(i as f32 * 10.0 - offset, -5.0),
                Point2::new(i as f32 * 10.0 - offset, 0.0)
            ],
            1.0,
            graphics::WHITE
        )?;
    }
    
    return ruler.build(ctx);
}