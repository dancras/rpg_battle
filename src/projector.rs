use nalgebra::{Point2, Vector2};

pub const PROJECTOR_UNUSED: f32 = 0.0;

pub struct ProjectorTopLeft {
    anchor_point: Point2<f32>,
    scale: f32
}

fn scale_value(value: f32, scale: f32) -> f32 {
    // todo rounding
    value * scale
}

impl ProjectorTopLeft {
    pub fn new(
        anchor_point: Point2<f32>,
        scale: f32
    ) -> Self {
        Self {
            anchor_point: anchor_point,
            scale: scale
        }
    }

    pub fn origin(&self) -> Point2<f32> {
        self.anchor_point
    }

    pub fn scale(&self, value: f32) -> f32 {
        scale_value(value, self.scale)
    }

    pub fn coords(&self, x: f32, y: f32) -> Point2<f32> {
        self.origin() + Vector2::new(self.scale(x), self.scale(y))
    }
}

pub struct ProjectorBottomLeft {
    anchor_point: Point2<f32>,
    scaled_height: f32,
    scale: f32
}

impl ProjectorBottomLeft {
    pub fn new(
        anchor_point: Point2<f32>,
        height: f32,
        scale: f32
    ) -> Self {
        Self {
            anchor_point: anchor_point,
            scaled_height: scale_value(height, scale),
            scale: scale
        }
    }

    pub fn local(&self) -> ProjectorTopLeft {
        ProjectorTopLeft {
            anchor_point: Point2::new(0.0, 0.0),
            scale: self.scale
        }
    }

    pub fn local_relative(&self, x: f32, y: f32) -> ProjectorTopLeft {
        ProjectorTopLeft {
            anchor_point: self.origin() + Vector2::new(self.scale(x), self.scale(y)),
            scale: self.scale
        }
    }

    pub fn origin(&self) -> Point2<f32> {
        self.anchor_point - Vector2::new(0.0, self.scaled_height)
    }

    pub fn scale(&self, value: f32) -> f32 {
        scale_value(value, self.scale)
    }

    pub fn coord(&self, value: f32) -> f32 {
        self.anchor_point.coords.y - self.scaled_height + self.scale(value)
    }

    pub fn coords(&self, x: f32, y: f32) -> Point2<f32> {
        self.origin() + Vector2::new(self.scale(x), self.scale(y))
    }

    pub fn point(&self, point: Point2<f32>) -> Point2<f32> {
        self.origin() + Vector2::new(self.scale(point.coords.x), self.scale(point.coords.y))
    }
}
