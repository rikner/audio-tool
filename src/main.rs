use audio::cpal;
use circular_queue::CircularQueue;

use nannou::prelude::*;
use nannou_audio as audio;
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use pitch_detection::Pitch;
use std::sync::{Arc, Mutex};

type AudioQueue = Arc<Mutex<CircularQueue<f32>>>;

const BUFFER_LEN_FRAMES: usize = 1024;

struct Model {
    queue: AudioQueue,
    _stream: audio::Stream<AudioQueue>, // keeps stream alive
    detector: McLeodDetector<f32>,
    pitch: Option<Pitch<f32>>,
    sample_rate: u32,
}

fn main() {
    nannou::app(model).simple_window(view).update(update).run();
}

fn model(_app: &App) -> Model {
    let queue: AudioQueue = Arc::new(Mutex::new(CircularQueue::<f32>::with_capacity(
        BUFFER_LEN_FRAMES,
    )));
    let audio_host = audio::Host::new();
    let stream = audio_host
        .new_input_stream(queue.clone())
        .capture(capture)
        .build()
        .unwrap();

    let config = stream.cpal_config();
    let sample_rate = config.sample_rate.0;

    stream.play().unwrap();

    let padding: usize = BUFFER_LEN_FRAMES / 2;

    let detector: McLeodDetector<f32> = McLeodDetector::new(BUFFER_LEN_FRAMES, padding);

    Model {
        detector,
        queue,
        _stream: stream,
        pitch: None,
        sample_rate,
    }
}

fn capture(queue: &mut AudioQueue, buffer: &audio::Buffer) {
    let mut queue = queue.lock().unwrap();
    for frame in buffer.frames() {
        queue.push(frame[0]);
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    let binding = model.queue.lock().unwrap();
    let current_audio_data = binding.iter().collect::<Vec<_>>();

    // copy current audio data into a Vec<f32>
    let signal = current_audio_data
        .iter()
        .map(|&x| x.to_owned())
        .collect::<Vec<f32>>();

    let pitch = model
        .detector
        .get_pitch(&signal, model.sample_rate as usize, 2.0, 0.3);

    model.pitch = pitch
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let boundary = app.window_rect();
    let binding = model.queue.lock().unwrap();
    let current_audio_data = binding.iter().collect::<Vec<_>>();

    draw.background().rgb(0.07, 0.09, 0.15);
    draw_pitch(&draw, boundary, &model.pitch);
    draw_wav_form(&draw, boundary, current_audio_data);
    draw.to_frame(app, &frame).unwrap();
}

fn draw_wav_form(draw: &Draw, boundary: geom::Rect, audio_data: Vec<&f32>) {
    // draw each point in the vector to create a wav form
    for (i, sample) in audio_data.iter().enumerate() {
        let x = map_range(i, 0, audio_data.len(), boundary.left(), boundary.right());

        draw.rect()
            .rgb(0.95, 0.55, 0.25)
            .x_y(x, 0.0)
            .w_h(1.0, *sample * 1000.0);
    }
}

fn draw_pitch(draw: &Draw, boundary: geom::Rect, pitch: &Option<Pitch<f32>>) {
    let text = match pitch {
        Some(pitch) => format!("{:.2} Hz", pitch.frequency),
        None => "No pitch detected".to_string(),
    };

    draw.text(&text)
        .rgb(0.95, 0.55, 0.25)
        .font_size(24)
        .y(boundary.top() - 50.0);
}
