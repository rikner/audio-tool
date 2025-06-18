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
    let channels = buffer.channels();

    match channels {
        1 => {
            for frame in buffer.frames() {
                queue.push(frame[0]);
            }
        }
        2 => {
            for frame in buffer.frames() {
                let sample = (frame[0] + frame[1]) / 2.0;
                queue.push(sample);
            }
        }
        _ => panic!("Unsupported number of channels"),
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
    draw_tuner_meter(&draw, boundary, &model.pitch);
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

fn draw_tuner_meter(draw: &Draw, boundary: geom::Rect, pitch: &Option<Pitch<f32>>) {
    // Meter dimensions
    let meter_width = 300.0;
    let meter_height = 30.0;
    let center_x = 0.0;
    let center_y = boundary.bottom() + 80.0;

    // Draw meter background
    draw.rect()
        .x_y(center_x, center_y)
        .w_h(meter_width, meter_height)
        .rgb(0.2, 0.2, 0.2);

    // Draw center line (in-tune)
    draw.line()
        .start(pt2(center_x, center_y - meter_height / 2.0))
        .end(pt2(center_x, center_y + meter_height / 2.0))
        .weight(2.0)
        .rgb(0.95, 0.95, 0.25);

    // Only draw needle if pitch is detected
    if let Some(pitch) = pitch {
        // Find the closest note
        let freq = pitch.frequency;
        let note_num = (12.0 * (freq / 440.0).log2()).round();
        let note_freq = 440.0 * 2.0_f32.powf(note_num / 12.0);

        // Cents difference from the closest note
        let cents = 1200.0 * (freq / note_freq).log2();

        // Map cents (-50 to +50) to meter width
        let max_cents = 50.0;
        let needle_x = (cents / max_cents).clamp(-1.0, 1.0) * (meter_width / 2.0 - 10.0);

        // Draw needle
        draw.line()
            .start(pt2(center_x + needle_x, center_y - meter_height / 2.0))
            .end(pt2(center_x + needle_x, center_y + meter_height / 2.0))
            .weight(4.0)
            .rgb(0.25, 0.95, 0.55);
    }
}
