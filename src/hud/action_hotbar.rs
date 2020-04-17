use ggez::{Context, GameResult};
use ggez::graphics::{self, Font};
use nalgebra::{Point2, Vector2};

pub fn draw(ctx: &mut Context, position: Point2<f32>) -> GameResult {

    for i in 0..10 {
        let icon = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(1.0),
            graphics::Rect {
                x: 0.0 + (i * 50) as f32,
                y: 0.0,
                w: 40.0,
                h: 40.0
            },
            graphics::WHITE
        )?;
        graphics::draw(ctx, &icon, (position,))?;

        let hotkey_number = (i + 1) % 10;
        let mut number = graphics::Text::new(hotkey_number.to_string());
        number.set_font(Font::default(), graphics::Scale::uniform(graphics::DEFAULT_FONT_SCALE * 0.5));
        graphics::draw(ctx, &number, (position + Vector2::new(2.0 + (i * 50) as f32, 2.0),))?;

        if i < 2 {
            let mut hotkey_action = "Attack";

            if i == 1 {
                hotkey_action = "Block"
            }

            let mut action = graphics::Text::new(hotkey_action);
            action.set_font(Font::default(), graphics::Scale::uniform(graphics::DEFAULT_FONT_SCALE * 0.8));
            let half_height = (action.height(ctx) / 2) as f32;
            let centering_offset = (40 - action.width(ctx)) as f32 / 2.0;
            graphics::draw(ctx, &action, (position + Vector2::new(centering_offset + (i * 50) as f32, 20.0 - half_height),))?;
        }
    }

    Ok(())

}
