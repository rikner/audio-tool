use nannou::prelude::*;
use std::time::SystemTime;
use circular_queue::CircularQueue;

// https://docs.rs/nannou_audio/latest/nannou_audio/
use nannou_audio as audio;

struct StreamModel {
	audio_circular_queue: CircularQueue<f32>,
}

struct Model {
	audio_circular_queue: CircularQueue<f32>,
    stream: audio::Stream<StreamModel>,
}

fn main() {
    nannou::app(model).simple_window(view).update(update).run();
}

fn on_audio_data(model: &mut StreamModel, buffer: &audio::Buffer) {
	println!("updating audio data at time: {:?}", SystemTime::now());
	buffer.frames().for_each(|frame| {
		frame.iter().for_each(|sample| {
			model.audio_circular_queue.push(*sample);
		});
	});
}

fn model(_app: &App) -> Model {
	let audio_circular_queue = CircularQueue::<f32>::with_capacity(8192);
    let stream_model = StreamModel {
    	audio_circular_queue: audio_circular_queue,
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
        audio_circular_queue: audio_circular_queue,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    println!("updating view at time: {:?}", SystemTime::now());
    let draw = app.draw();
    let boundary = app.window_rect();

    draw.text("Hello World!").font_size(48).color(ORANGE);

    draw.to_frame(app, &frame).unwrap();
}
