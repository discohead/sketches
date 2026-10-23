//! Shared runtime for pieces: launch, the fixed-size window, and frame capture.
//!
//! ```ignore
//! fn main() {
//!     kit::launch(kit::piece!(), nannou::app(model).update(update));
//! }
//!
//! fn model(app: &App) -> Model {
//!     kit::window(app).view(view).build();
//!     Model
//! }
//! ```

use std::sync::OnceLock;

use nannou::App;
use nannou::prelude::Entity;

pub use piece_meta::PieceMeta;

#[cfg(not(target_arch = "wasm32"))]
mod capture;

static META: OnceLock<PieceMeta> = OnceLock::new();

/// The piece's `piece.toml`, read at compile time from the crate root.
#[macro_export]
macro_rules! piece {
    () => {
        $crate::PieceMeta::from_toml_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/piece.toml"
        )))
        .expect("piece.toml")
    };
}

/// Stores `meta` and runs the app.
///
/// When `KIT_CAPTURE_DIR` and `KIT_CAPTURE_FRAMES` are set, time is frame-locked to
/// `1/fps` and the requested frames are saved as `<dir>/<frame:05>.png`, then the app exits.
pub fn launch<M: 'static + Send + Sync>(meta: PieceMeta, builder: nannou::app::Builder<M>) {
    assert!(META.set(meta).is_ok(), "kit::launch was called twice");

    #[cfg(not(target_arch = "wasm32"))]
    let builder = match capture::CaptureConfig::from_env() {
        Ok(Some(config)) => builder
            .synchronous_pipeline_compilation(true)
            .add_plugin(capture::CapturePlugin::new(config, self::meta().time.fps)),
        Ok(None) => builder,
        Err(e) => panic!("kit capture: {e:#}"),
    };

    builder.run();
}

/// The meta passed to [`launch`].
pub fn meta() -> &'static PieceMeta {
    META.get().expect("kit::meta() called before kit::launch")
}

/// Where the app is in its loop, in `[0, 1)`.
pub fn loop_phase(app: &App) -> f32 {
    phase_at(app.time(), meta().time.seconds)
}

/// `(t / seconds).rem_euclid(1.0)`.
pub fn phase_at(t: f32, seconds: f32) -> f32 {
    (t / seconds).rem_euclid(1.0)
}

/// The piece's window: primary, not resizable, exactly `canvas.width × canvas.height` pixels.
///
/// On native the scale factor is forced to 1.0, so on a Retina display the window is
/// half as many points and captures still match the canvas pixel for pixel.
pub fn window<'a, 'w, 's, M: 'static>(app: &'a App<'w, 's>) -> WindowBuilder<'a, 'w, 's, M> {
    WindowBuilder { app, view: None }
}

/// Returned by [`window`]. Nannou's own window builder lives in a private module, so its
/// type cannot be named here; this wrapper forwards to it in [`WindowBuilder::build`].
pub struct WindowBuilder<'a, 'w, 's, M> {
    app: &'a App<'w, 's>,
    view: Option<fn(&App<'_, '_>, &M)>,
}

impl<M: 'static> WindowBuilder<'_, '_, '_, M> {
    /// The view function, as in nannou's `window::Builder::view`.
    pub fn view(mut self, view: fn(&App<'_, '_>, &M)) -> Self {
        self.view = Some(view);
        self
    }

    /// Builds the window, returning its entity.
    pub fn build(self) -> Entity {
        let canvas = &meta().canvas;
        let mut builder = self.app.new_window::<M>();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut window = nannou::prelude::Window {
                present_mode: nannou::prelude::PresentMode::Mailbox,
                ..Default::default()
            };
            window.resolution.set_scale_factor_override(Some(1.0));
            builder = builder.window(window);
        }

        builder = builder
            .primary()
            .size_pixels(canvas.width, canvas.height)
            .resizable(false);
        if let Some(view) = self.view {
            builder = builder.view(view);
        }
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_wraps() {
        assert_eq!(phase_at(0.0, 4.0), 0.0);
        assert_eq!(phase_at(5.0, 4.0), 0.25);
        assert_eq!(phase_at(-1.0, 4.0), 0.75);
    }
}
