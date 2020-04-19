use ggez::graphics::Color;

pub const BLUE: Color = Color::new(0.6, 0.6, 1.0, 1.0);
pub const GREEN: Color = Color::new(0.2, 1.0, 0.4, 1.0);
pub const GREY: Color = Color::new(0.6, 0.6, 0.6, 1.0);
pub const RED: Color = Color::new(1.0, 0.2, 0.3, 1.0);
pub const YELLOW: Color = Color::new(1.0, 1.0, 0.3, 1.0);

const DARKER_FRAME_ADJUSTMENT: f32 = 0.2;

fn darken_tint(tint: f32) -> f32 {
    if tint <= DARKER_FRAME_ADJUSTMENT {
        0.0
    } else {
        tint - DARKER_FRAME_ADJUSTMENT
    }
}

pub fn darker(color: Color) -> Color {
    Color::new(
        darken_tint(color.r),
        darken_tint(color.g),
        darken_tint(color.b),
        color.a
    )
}
