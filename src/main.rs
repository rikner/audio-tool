mod tools;

use tools::*;
use circular_queue::CircularQueue;
use nannou::prelude::*;
use nannou_audio as audio;
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::yin::YINDetector;
use pitch_detection::detector::PitchDetector;
use pitch_detection::Pitch;
use std::sync::{Arc, Mutex};

type AudioQueue = Arc<Mutex<CircularQueue<f32>>>;

const BUFFER_LEN_FRAMES: usize = 1024;
const POWER_THRESHOLD: f32 = 0.1;
const CLARITY_THRESHOLD: f32 = 0.01;

struct Model {
    queue: AudioQueue,
    _stream: audio::Stream<AudioQueue>, // keeps stream alive
    detector: YINDetector<f32>,
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

    let detector: YINDetector<f32> = YINDetector::new(BUFFER_LEN_FRAMES, padding);

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
    let signal: Vec<f32> = binding.iter().cloned().collect();

    let pitch = model.detector.get_pitch(
        &signal,
        model.sample_rate as usize,
        POWER_THRESHOLD,
        CLARITY_THRESHOLD,
    );

    model.pitch = pitch;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let boundary = app.window_rect();

    let binding = model.queue.lock().unwrap();
    let current_audio_data = binding.iter().collect::<Vec<_>>();

    draw.background().rgb(0.07, 0.09, 0.15);
    // draw_pitch(&draw, boundary, &model.pitch);
    draw_note(&draw, boundary, &model.pitch);
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

fn draw_note(
    draw: &Draw,
    boundary: geom::Rect,
    pitch: &Option<Pitch<f32>>
) {
    let text = match pitch {
        Some(pitch) => frequency_to_note(pitch.frequency),
        None => "".to_string(),
    };

    draw.text(&text)
        .rgb(0.95, 0.55, 0.25)
        .font_size(24)
        .y(boundary.top() - 50.0);
}
