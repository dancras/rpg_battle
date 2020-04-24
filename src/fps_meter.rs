use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, MeshBuilder};
use ggez::timer;
use std::time::Duration;

// update frame time
    // OR remaining update time after frame
// dropped frames
// draw frame time

pub struct FpsMeter {
    flag_draw: bool,
    draw_start_time: Duration,
    draw_times: Vec<u128>,
    dropped_frames: Vec<i32>,
    dropped_frame_count: i32
}

impl FpsMeter {
    pub fn new() -> Self {
        Self {
            flag_draw: false,
            draw_start_time: Duration::new(0, 0),
            draw_times: Vec::new(),
            dropped_frames: Vec::new(),
            dropped_frame_count: 0
        }
    }

    pub fn update_start(&mut self, _ctx: &Context) {
        self.dropped_frame_count = -1;
    }

    pub fn update_loop(&mut self, _ctx: &Context) {
        self.dropped_frame_count += 1
    }

    pub fn update_end(&mut self, _ctx: &Context) {
        self.dropped_frames.push(self.dropped_frame_count);
    }

    pub fn draw_start(&mut self, ctx: &Context) {

        if self.flag_draw {
            panic!("draw_start() called without ending previous draw");
        } else {
            self.flag_draw = true;
            self.draw_start_time = timer::time_since_start(ctx);
        }

    }

    pub fn draw_end(&mut self, ctx: &mut Context) {

        if !self.flag_draw {
            panic!("draw_end() called without starting draw");
        } else {
            self.flag_draw = false;

            let end_time = timer::time_since_start(ctx);
            let total_time = (end_time - self.draw_start_time).as_millis();

            self.draw_times.push(total_time)
        }

    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult {

        if self.draw_times.len() == 0 {
            return Ok(());
        }

        let mut meter = &mut MeshBuilder::new();

        let width = (self.draw_times.len() * 2) as f32;
        let start = 1440.0 - width;
        let baseline = 900.0;

        {
            let (head, tail) = self.draw_times.split_at(1);
            let mut previous = head[0] as f32;

            for (i, time) in tail.iter().enumerate() {

                let i = i as f32;
                let current = *time as f32;

                // println!("start {} {}", start + i * 2.0, baseline - previous);
                // println!("end {} {}", start + (i + 1.0) * 2.0, baseline - current);

                meter = meter.line::<[f32; 2]>(
                    &[
                        [start + i * 2.0, baseline - previous].into(),
                        [start + (i + 1.0) * 2.0, baseline - current].into()
                    ],
                    1.0,
                    Color::new(0.2, 1.0, 0.6, 1.0)
                )?;

                previous = current;
            }
        }

        {
            let (head, tail) = self.dropped_frames.split_at(1);
            let mut previous = head[0] as f32;

            for (i, time) in tail.iter().enumerate() {

                let i = i as f32;
                let current = *time as f32;

                // println!("start {} {}", start + i * 2.0, baseline - previous);
                // println!("end {} {}", start + (i + 1.0) * 2.0, baseline - current);

                meter = meter.line::<[f32; 2]>(
                    &[
                        [start + i * 2.0, baseline - previous * 10.0].into(),
                        [start + (i + 1.0) * 2.0, baseline - current * 10.0].into()
                    ],
                    1.0,
                    Color::new(1.0, 1.0, 0.6, 1.0)
                )?;

                previous = current;
            }
        }


        if self.draw_times.len() > 1 {
            let mesh = meter.build(ctx)?;

            graphics::draw::<_, ([f32; 2],)>(ctx, &mesh, ([0.0, 0.0],))?;
        }

        Ok(())
    }


}
