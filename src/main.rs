mod tools;

use circular_queue::CircularQueue;

use tools::*;
use nannou::prelude::*;
use nannou_audio as audio;
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::yin::YINDetector;
use pitch_detection::detector::PitchDetector;
use pitch_detection::Pitch;
use std::sync::{Arc, Mutex};

use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::scaling::divide_by_N_sqrt;


type AudioQueue = Arc<Mutex<CircularQueue<f32>>>;

const BUFFER_LEN_FRAMES: usize = 2048;
const POWER_THRESHOLD: f32 = 0.1;
const CLARITY_THRESHOLD: f32 = 0.01;

struct Model {
    queue: AudioQueue,
    _stream: audio::Stream<AudioQueue>, // keeps stream alive
    detector: McLeodDetector<f32>,
    pitch: Option<Pitch<f32>>,
    musical_note: String,
    deviation: f32,
    sample_rate: u32,
    power_spectrum: Vec<f32>,
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
        musical_note: "".to_string(),
        deviation: 0.0,
        sample_rate,
        power_spectrum: vec![0.0; BUFFER_LEN_FRAMES],
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

    let signal = current_audio_data
        .iter()
        .map(|&x| x.to_owned())
        .collect::<Vec<f32>>();

    let pitch = model.detector.get_pitch(
        &signal,
        model.sample_rate as usize,
        POWER_THRESHOLD,
        CLARITY_THRESHOLD,
    );

    let note_and_deviation = match pitch.as_ref() {
        Some(pitch) => frequency_to_note_with_deviation(pitch.frequency),
        None => NoteAndDeviation {
            musical_note: "".to_string(),
            deviation: 0.0,
        },
    };

    let hann_window = hann_window(&signal);
    // calc spectrum
    let spectrum_hann_window = samples_fft_to_spectrum(
        // (windowed) samples
        &hann_window,
        // sampling rate
        44100,
        // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
        FrequencyLimit::All,
        // optional scale
        Some(&divide_by_N_sqrt),
    ).unwrap();

    let power_spectrum = spectrum_hann_window.data().iter().map(|(_, x)| x.val().abs().powf(2.0)).collect::<Vec<f32>>();

    model.pitch = pitch;
    model.musical_note = note_and_deviation.musical_note;
    model.deviation = note_and_deviation.deviation;
    model.power_spectrum = power_spectrum;
    
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let boundary = app.window_rect();

    let binding = model.queue.lock().unwrap();
    let current_audio_data = binding.iter().collect::<Vec<_>>();

    draw.background().rgb(0.07, 0.09, 0.15);
    draw_pitch_and_deviation(&draw, boundary, &model.pitch, model.deviation);
    // draw_wav_form(&draw, boundary, current_audio_data);
    draw_power_spectrum(&draw, boundary, &model.power_spectrum);
    draw.to_frame(app, &frame).unwrap();
}

fn draw_wav_form(draw: &Draw, boundary: geom::Rect, audio_data: Vec<&f32>) {
    // draw each point in the vector to create a wav form
    for (i, sample) in audio_data.iter().enumerate() {
        let x = map_range(
            i,
            0,
            audio_data.len(),
            boundary.left(),
            boundary.right()
        );

        draw.rect()
            .rgb(0.95, 0.55, 0.25)
            .x_y(x, 0.0)
            .w_h(1.0, *sample * 1000.0);
    }
}


fn draw_pitch_and_deviation(draw: &Draw, boundary: geom::Rect, pitch: &Option<Pitch<f32>>, deviation: f32) {
    let text = match pitch {
        Some(pitch) => frequency_to_note(pitch.frequency),
        None => "".to_string(),
    };

    draw.text(&text)
        .rgb(0.95, 0.55, 0.25)
        .font_size(24)
        .y(boundary.top() - 50.0);

    // let deviation_text = format!("Deviation: {:.2}", deviation);
    // draw.text(&deviation_text)
    //     .rgb(0.95, 0.55, 0.25)
    //     .font_size(24)
    //     .y(boundary.top() - 80.0);

    // draw a meter below the pitch text in the center to show the deviation from the note
    let x_centered = boundary.x();
    let w = 100.0;
    let h = 20.0;
    let y = boundary.top() - 100.0;
    let deviation_meter_rect = geom::Rect::from_x_y_w_h(x_centered, y, w, h);
 
    // draw.rect()
    //     .color(rgba(0.95, 0.55, 0.25, 0.5))
    //     .stroke(rgba(0.95, 0.55, 0.25, 0.5))
    //     .stroke_weight(1.0)
    //     .x_y(deviation_meter_rect.x(), deviation_meter_rect.y())
    //     .w_h(deviation_meter_rect.w(), deviation_meter_rect.h());

    let deviation_meter_needle_x = map_range(
        deviation,
        -100.0,
        100.0,
        deviation_meter_rect.left(),
        deviation_meter_rect.right(),
    );

    // draw.rect()
    //     .color(rgba(0.95, 0.55, 0.25, 0.9))
    //     .stroke(GREEN)
    //     .stroke_weight(1.0)
    //     .x_y(deviation_meter_needle_x, deviation_meter_rect.y())
    //     .w_h(2.0, deviation_meter_rect.h());
    

}
