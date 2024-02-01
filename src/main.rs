use circular_queue::CircularQueue;

use nannou::prelude::*;
use nannou_audio as audio;
use std::sync::{Arc, Mutex};

type AudioQueue = Arc<Mutex<CircularQueue<f32>>>;
struct Model {
    queue: AudioQueue,
    stream: audio::Stream<AudioQueue>, // keeps stream alive
}

fn main() {
    nannou::app(model).simple_window(view).update(update).run();
}

fn model(app: &App) -> Model {
    let queue: AudioQueue = Arc::new(Mutex::new(CircularQueue::<f32>::with_capacity(8192)));
    let audio_host = audio::Host::new();
    let stream = audio_host
        .new_input_stream(queue.clone())
        .capture(capture)
        .build()
        .unwrap();

    stream.play().unwrap();

    Model { queue, stream }
}

fn capture(queue: &mut AudioQueue, buffer: &audio::Buffer) {
    let mut queue = queue.lock().unwrap();
    for frame in buffer.frames() {
        queue.push(frame[0]);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let boundary = app.window_rect();
    let binding = model.queue.lock().unwrap();
    let current_audio_data = binding.iter().collect::<Vec<_>>();

    draw.background().rgb(0.07, 0.09, 0.15);
    draw_wav_form(&draw, boundary, current_audio_data);
    draw.to_frame(app, &frame).unwrap();
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn draw_wav_form(draw: &Draw, boundary: geom::Rect, audio_data: Vec<&f32>) {
    // draw each point in the vector to create a wav form
    for (i, sample) in audio_data.iter().enumerate() {
        let x = map_range(
            i,
            0,
            audio_data.len(),
            boundary.left() * 0.8,
            boundary.right() * 0.8,
        );

        draw.rect()
            .rgb(0.95, 0.55, 0.25)
            .x_y(x, 0.0)
            .w_h(1.0, *sample * 1000.0);
    }
}
