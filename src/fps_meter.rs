use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, MeshBuilder};
use ggez::timer;
use std::time::Duration;

struct History {
    values: [i32; 400],
    i: usize
}

impl History {
    fn new() -> Self {
        Self {
            values: [0; 400],
            i: 0
        }
    }

    fn add_entry(&mut self, entry: i32) {
        self.values[self.i] = entry;
        self.i = (self.i + 1) % 400;
    }

    fn get_entry(&self, entry: usize) -> i32 {
        let i_for_entry = (400 + (self.i as i32 - 1 - entry as i32)) % 400;
        self.values[i_for_entry as usize]
    }
}

pub struct FpsMeter {
    flag_draw: bool,
    draw_start_time: Duration,
    draw_times: History,
    dropped_frames: History,
    dropped_frame_count: i32,
    update_start_time: Duration,
    update_times: History
}

impl FpsMeter {
    pub fn new() -> Self {
        Self {
            flag_draw: false,
            draw_start_time: Duration::new(0, 0),
            draw_times: History::new(),
            dropped_frames: History::new(),
            dropped_frame_count: 0,
            update_start_time: Duration::new(0, 0),
            update_times: History::new(),
        }
    }

    pub fn update_start(&mut self, ctx: &Context) {
        self.dropped_frame_count = -1;

        self.update_start_time = timer::time_since_start(ctx);
    }

    pub fn update_loop(&mut self, _ctx: &Context) {
        self.dropped_frame_count += 1
    }

    pub fn update_end(&mut self, ctx: &Context) {
        self.dropped_frames.add_entry(self.dropped_frame_count);

        let end_time = timer::time_since_start(ctx);
        let total_time = (end_time - self.update_start_time).as_micros() / 100;

        self.update_times.add_entry(total_time as i32);
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

            self.draw_times.add_entry(total_time as i32);
        }

    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult {

        let mut meter = &mut MeshBuilder::new();
        let right_anchor = 1440.0;
        let baseline = 900.0;


        let mut previous_draw_time = self.draw_times.get_entry(0) as f32;
        let mut previous_dropped_frames = self.dropped_frames.get_entry(0) as f32;
        let mut previous_update_time = self.update_times.get_entry(0) as f32;

        for i in 1..400 {

            let current_draw_time = self.draw_times.get_entry(i) as f32;
            let current_dropped_frames = self.dropped_frames.get_entry(i) as f32;
            let current_update_time = self.update_times.get_entry(i) as f32;
            let i = i as f32;

            meter = meter.line::<[f32; 2]>(
                &[
                    [right_anchor - i * 2.0, baseline - previous_draw_time].into(),
                    [right_anchor - (i + 1.0) * 2.0, baseline - current_draw_time].into()
                ],
                1.0,
                Color::new(0.2, 1.0, 0.6, 1.0)
            )?;

            meter = meter.line::<[f32; 2]>(
                &[
                    [right_anchor - i * 2.0, baseline - previous_dropped_frames * 10.0].into(),
                    [right_anchor - (i + 1.0) * 2.0, baseline - current_dropped_frames * 10.0].into()
                ],
                1.0,
                Color::new(1.0, 1.0, 0.6, 1.0)
            )?;

            meter = meter.line::<[f32; 2]>(
                &[
                    [right_anchor - i * 2.0, baseline - previous_update_time].into(),
                    [right_anchor - (i + 1.0) * 2.0, baseline - current_update_time].into()
                ],
                1.0,
                Color::new(0.2, 3.0, 1.0, 1.0)
            )?;

            previous_draw_time = current_draw_time;
            previous_dropped_frames = current_dropped_frames;
            previous_update_time = current_update_time;
        }

        let mesh = meter.build(ctx)?;

        graphics::draw::<_, ([f32; 2],)>(ctx, &mesh, ([0.0, 0.0],))?;

        Ok(())
    }


}
