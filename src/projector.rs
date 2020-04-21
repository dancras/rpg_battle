use nalgebra::{Point2, Vector2};
use std::ops::Mul;

pub const PROJECTOR_UNUSED: f32 = 0.0;

pub struct Projector {
    anchor_point: Point2<f32>,
    scale: f32,
    width: f32,
    height: f32
}

impl Projector {
    pub fn new(
        anchor_point: Point2<f32>,
        scale: f32,
        width: f32,
        height: f32
    ) -> Self {
        Self {
            anchor_point: anchor_point,
            scale: scale,
            width: width,
            height: height
        }
    }

    pub fn local(&self) -> Projector {
        Projector {
            anchor_point: Point2::new(0.0, 0.0),
            scale: self.scale,
            width: 0.0,
            height: 0.0
        }
    }

    pub fn local_relative(&self, x: f32, y: f32) -> Projector {
        Projector {
            anchor_point: self.origin() + self.scale(Vector2::new(x, y)),
            scale: self.scale,
            width: 0.0,
            height: 0.0
        }
    }

    pub fn centered(&self, width: f32, height: f32) -> Projector {

        let midpoint = self.origin() + Vector2::new(self.width, self.height) / 2.0;
        let centering_translation = self.scale(Vector2::new(width, height) / 2.0);

        Projector {
            anchor_point: midpoint - centering_translation,
            scale: self.scale,
            width: width,
            height: height
        }
    }

    pub fn centered_horizontal(&self, width: f32) -> Projector {

        let midpoint = self.origin() + Vector2::new(self.width / 2.0, 0.0);
        let centering_translation = self.scale(Vector2::new(width / 2.0, 0.0));

        Projector {
            anchor_point: midpoint - centering_translation,
            scale: self.scale,
            width: width,
            height: self.height
        }
    }

    pub fn bottom_left(&self, height: f32) -> Projector {
        Projector {
            anchor_point: self.anchor_point + Vector2::new(0.0, self.height) - self.scale(Vector2::new(0.0, height)),
            scale: self.scale,
            width: self.width,
            height: height
        }
    }

    pub fn top_right(&self, width: f32) -> Projector {
        Projector {
            anchor_point: self.anchor_point + Vector2::new(self.width, 0.0) - self.scale(Vector2::new(width, 0.0)),
            scale: self.scale,
            width: width,
            height: self.height
        }
    }

    pub fn to_local_x(&self, x: f32) -> f32 {
        x - self.origin().coords.x
    }

    pub fn to_local_y(&self, y: f32) -> f32 {
        y - self.origin().coords.y
    }

    pub fn origin(&self) -> Point2<f32> {
        self.anchor_point
    }

    pub fn scale<V: Mul<f32, Output=V>>(&self, value: V) -> V {
        value * self.scale
        //scale_value(value, self.scale)
    }

    pub fn coords(&self, x: f32, y: f32) -> Point2<f32> {
        self.origin() + Vector2::new(self.scale(x), self.scale(y))
    }
}
