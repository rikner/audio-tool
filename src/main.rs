use std::time::SystemTime;
use nannou::prelude::*;

struct Model {}

fn main() {
    nannou::app(model).simple_window(view).update(update).run();
}

fn model(_app: &App) -> Model {
    Model {}
}


fn update(_app: &App, model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
	println!("updating view at time: {:?}", SystemTime::now());
    let draw = app.draw();
    let boundary = app.window_rect();

    draw.text("Hello World!").font_size(48).color(ORANGE);

    draw.to_frame(app, &frame).unwrap();
}
