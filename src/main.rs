use nannou::{color::chromatic_adaptation::AdaptInto, prelude::*};
use std::time::SystemTime;
use circular_queue::CircularQueue;
use std::sync::{Arc, Mutex};

// https://docs.rs/nannou_audio/latest/nannou_audio/
use nannou_audio as audio;

type AudioQueue = Arc<Mutex<CircularQueue<f32>>>;

struct StreamModel {
	audio_queue: AudioQueue,
}

struct Model {
	audio_queue: AudioQueue,
    stream: audio::Stream<StreamModel>,
}

fn main() {
    nannou::app(model).simple_window(view).update(update).run();
}

fn on_audio_data(model: &mut StreamModel, buffer: &audio::Buffer) {
	println!("updating audio data at time: {:?}", SystemTime::now());
	let mut queue = model.audio_queue.lock().unwrap();
	buffer.frames().for_each(|frame| {
		frame.iter().for_each(|sample| {
			let sample_value: f32 = *sample;
			queue.push(sample_value);
		});
	});
}

fn model(_app: &App) -> Model {
	let audio_queue = Arc::new(Mutex::new(CircularQueue::<f32>::with_capacity(8192)));
    let stream_model = StreamModel {
    	audio_queue: audio_queue.clone(),
    };
    let audio_host = audio::Host::new();
    let input_stream = audio_host
        .new_input_stream(stream_model)
        .capture(on_audio_data)
        .build()
        .unwrap();
    input_stream.play().unwrap();

    Model {
        stream: input_stream,
        audio_queue,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    println!("updating view at time: {:?}", SystemTime::now());
    let draw = app.draw();
    let boundary = app.window_rect();

    draw.background().rgb(0.07, 0.09, 0.15);

    let binding = model.audio_queue.lock().unwrap();
    let current_audio_data = binding.iter().collect::<Vec<_>>();
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
