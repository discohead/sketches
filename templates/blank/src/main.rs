use nannou::prelude::*;

fn main() {
    kit::launch(kit::piece!(), nannou::app(model).update(update));
}

struct Model;

fn model(app: &App) -> Model {
    kit::window!(app).view(view).build();
    Model
}

fn update(_app: &App, _model: &mut Model) {}

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    draw.background().color(BLACK);
}
