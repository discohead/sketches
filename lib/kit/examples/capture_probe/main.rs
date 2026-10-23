//! A white disc on black circling the centre once every 2 seconds: uniform motion, so every
//! frame-to-frame change is the same size, including the wrap from the last frame to the first.
//!
//! The orbit radius is 140 px so the disc stays on the 540×360 canvas. A disc moving only along
//! x (`200 cos`) would stop at the wrap, and the seam check in `tests/capture.rs` needs uniform
//! motion there.

use nannou::prelude::*;

// Uniform circular motion, not the plan's `x = 200 cos(TAU * phase)`: that disc stops at
// phase 0, the loop seam, so d(59->0) would be about 8% of the mean step even with perfect
// frame locking and the seam check could never pass.
const ORBIT: f32 = 140.0;

fn main() {
    let meta = kit::PieceMeta::from_toml_str(include_str!("piece.toml")).expect("piece.toml");
    kit::launch(meta, nannou::app(model));
}

struct Model;

fn model(app: &App) -> Model {
    kit::window!(app).view(view).build();
    Model
}

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    draw.background().color(BLACK);
    let a = TAU * kit::loop_phase(app);
    draw.ellipse()
        .x_y(ORBIT * a.cos(), ORBIT * a.sin())
        .radius(40.0)
        .color(WHITE);
}
