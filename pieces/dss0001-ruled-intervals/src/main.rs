use nannou::prelude::*;

// Euclidean rhythm E(5, 13): 5 onsets spread over 13 steps.
const STEPS: usize = 13;
const ONSETS: usize = 5;
// Two cycles of the rhythm: 26 bands of 36 px make a 936 px field inside a
// 72 px margin.
const BANDS: usize = 2 * STEPS;
const BAND: f32 = 36.0;
// Two full rotations of the rhythm per loop: a beat every 0.25 s.
const BEATS: usize = 2 * STEPS;
const MARGIN: f32 = 72.0;
const THICK: f32 = 9.0;
const HAIR: f32 = 1.0;

fn main() {
    kit::launch(kit::piece!(), nannou::app(model).update(update));
}

struct Model;

fn model(app: &App) -> Model {
    kit::window!(app).view(view).build();
    Model
}

fn update(_app: &App, _model: &mut Model) {}

fn onset(i: usize) -> bool {
    (i * ONSETS) % STEPS < ONSETS
}

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    let paper = Color::srgb(0.925, 0.910, 0.875);
    let graphite = Color::srgb(0.17, 0.17, 0.18);
    draw.background().color(paper);

    // One step per beat. Across each beat every rule eases from its thickness at
    // this step to its thickness at the next.
    let beats = kit::loop_phase(app) * BEATS as f32;
    let step = (beats.floor() as usize) % STEPS;
    let ease = 0.5 - 0.5 * (beats.fract() * PI).cos();

    let half = 540.0;
    let width = 1080.0 - 2.0 * MARGIN;
    for b in 0..BANDS {
        let from = if onset((b + step) % STEPS) { THICK } else { HAIR };
        let to = if onset((b + step + 1) % STEPS) { THICK } else { HAIR };
        let th = from + (to - from) * ease;
        // Top edge of the rule in pixels from the top, centred in its band.
        let top = MARGIN + b as f32 * BAND + (BAND - th) / 2.0;
        let y = half - top - th / 2.0;
        draw.rect().x_y(0.0, y).w_h(width, th).color(graphite);
    }
}
