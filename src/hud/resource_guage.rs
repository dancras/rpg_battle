use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, Mesh, MeshBuilder};

pub struct ResourceGuage {
    pub color: Color,
    pub max_value: f32,
    pub current_value: f32,
}

pub fn create_mesh(ctx: &mut Context, viewmodel: &ResourceGuage) -> GameResult<Mesh> {
    MeshBuilder::new()
        .rectangle(
            graphics::DrawMode::stroke(2.0),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 20.0
            },
            graphics::WHITE,
        )
        .rectangle(
            graphics::DrawMode::fill(),
            graphics::Rect {
                x: 4.0,
                y: 4.0,
                w: 92.0 * (viewmodel.current_value / viewmodel.max_value),
                h: 12.0
            },
            viewmodel.color,
        )
        .build(ctx)
}
