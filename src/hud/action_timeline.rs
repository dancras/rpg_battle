use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, Mesh, MeshBuilder};
use ggez::nalgebra::{Point2};
use std::collections::HashMap;

const ACTION_POINTS_PER_SECOND: f32 = 60.0;

pub struct ActionTimeline {
    next_subject_id: i32,
    time: f32,
    subject_colors: HashMap<i32, Color>,
    subject_times: HashMap<i32, f32>,
    ordered_subjects: Vec<i32>
}

impl ActionTimeline {
    pub fn new() -> Self {
        Self {
            next_subject_id: 0,
            time: 0.0,
            subject_colors: HashMap::new(),
            subject_times: HashMap::new(),
            ordered_subjects: Vec::new()
        }
    }

    pub fn add_subject(&mut self, color: Color, time: f32) -> i32 {
        self.next_subject_id += 1;

        self.subject_colors.insert(self.next_subject_id, color);
        self.subject_times.insert(self.next_subject_id, time);

        let mut insert_position = 0;

        for id in &self.ordered_subjects {
            if self.subject_times[id] < time {
                break;
            }

            insert_position += 1;
        }

        self.ordered_subjects.insert(insert_position, self.next_subject_id);

        self.next_subject_id
    }

    pub fn update_subject(&mut self, subject_id: i32, new_time: f32) {
        self.subject_times.insert(subject_id, new_time);
        self.ordered_subjects.retain(|&id| id != subject_id);

        let mut insert_position = 0;

        for id in &self.ordered_subjects {
            if self.subject_times[id] < new_time {
                break;
            }

            insert_position += 1;
        }

        self.ordered_subjects.insert(insert_position, subject_id);
    }

    pub fn update(&mut self, time: f32) {
        self.time = time;
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

    let tenth_offset = (viewmodel.time % 100.0 / 10.0).floor();

    for i in 0..41 {

        if i == 0 && offset > 0.0 {
            continue;
        }

        let is_tenth = (tenth_offset + i as f32) % 10.0 == 0.0;

        ruler = ruler.line(
            &[
                Point2::new(i as f32 * 10.0 - offset, if is_tenth { -5.0 } else { -3.0 }),
                Point2::new(i as f32 * 10.0 - offset, 0.0)
            ],
            1.0,
            graphics::WHITE
        )?;
    }

    let mut previous_position = -100.0;
    let mut previous_stack = 0;

    for id in &viewmodel.ordered_subjects {
        let subject_time = viewmodel.subject_times[id];
        let mut subject_position = subject_time - viewmodel.time;
        let mut current_stack = 1;

        if subject_position < 0.0 {
            subject_position = 0.0;
        }

        if previous_position + 16.0 > subject_position &&
            previous_position - 16.0 < subject_position {
            current_stack = previous_stack + 1;
        }

        previous_position = subject_position;
        previous_stack = current_stack;

        ruler = ruler.circle(
            graphics::DrawMode::fill(),
            Point2::new(subject_position, -18.0 * current_stack as f32),
            8.0,
            0.5,
            viewmodel.subject_colors[id]
        );
    }
    
    return ruler.build(ctx);
}