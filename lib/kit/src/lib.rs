//! Shared runtime for pieces: launch, the fixed-size window, and frame capture.
//!
//! ```ignore
//! fn main() {
//!     kit::launch(kit::piece!(), nannou::app(model).update(update));
//! }
//!
//! fn model(app: &App) -> Model {
//!     kit::window!(app).view(view).build();
//!     Model
//! }
//! ```

use std::sync::OnceLock;

use nannou::App;

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

/// Nannou's window builder for the piece's window: primary, not resizable, and
/// `canvas.width × canvas.height` pixels. Chain `.view(view)`, event handlers, and `.build()`.
///
/// On native, [`launch`] forces the primary window's scale factor to 1.0, so on a Retina
/// display the window is half as many points and every pixel matches the canvas.
#[macro_export]
macro_rules! window {
    ($app:expr) => {{
        let canvas = &$crate::meta().canvas;
        $app.new_window()
            .primary()
            .size_pixels(canvas.width, canvas.height)
            .resizable(false)
    }};
}

/// Stores `meta` and runs the app.
///
/// When `KIT_CAPTURE_DIR` and `KIT_CAPTURE_FRAMES` are set, time is frame-locked to
/// `1/fps` and the requested frames are saved as `<dir>/<frame:05>.png`, then the app exits.
/// The process exits 0 only if every requested frame was written; otherwise it exits 1.
pub fn launch<M: 'static + Send + Sync>(meta: PieceMeta, builder: nannou::app::Builder<M>) {
    assert!(META.set(meta).is_ok(), "kit::launch was called twice");

    #[cfg(not(target_arch = "wasm32"))]
    let builder = {
        let builder = builder.add_plugin(ExactPixelsPlugin);
        match capture::CaptureConfig::from_env() {
            Ok(Some(config)) => {
                builder.add_plugin(capture::CapturePlugin::new(config, self::meta().time.fps))
            }
            Ok(None) => builder,
            Err(e) => {
                eprintln!("kit capture: {e:#}");
                std::process::exit(1);
            }
        }
    };

    builder.run();

    #[cfg(not(target_arch = "wasm32"))]
    if capture::failed() {
        std::process::exit(1);
    }
}

/// Keeps the primary window at exactly the canvas size in physical pixels, in live preview and
/// capture alike.
#[cfg(not(target_arch = "wasm32"))]
struct ExactPixelsPlugin;

#[cfg(not(target_arch = "wasm32"))]
impl bevy::app::Plugin for ExactPixelsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        use bevy::ecs::prelude::*;
        use bevy::window::{PrimaryWindow, Window};

        // Set the override as the window is spawned, before winit creates the OS window.
        app.add_observer(
            |add: On<Add, PrimaryWindow>, mut windows: Query<&mut Window>| {
                if let Ok(mut window) = windows.get_mut(add.entity) {
                    window.resolution.set_scale_factor_override(Some(1.0));
                }
            },
        );

        // bevy_winit applies the monitor's scale factor to the physical size when it creates
        // the window (1080×720 for a 540×360 canvas on Retina) until winit's resize arrives.
        // Put it back before anything renders at the wrong size.
        app.add_systems(
            bevy::app::First,
            |mut windows: Query<&mut Window, With<PrimaryWindow>>| {
                let canvas = &meta().canvas;
                for mut window in &mut windows {
                    let r = &window.resolution;
                    if r.scale_factor_override() == Some(1.0)
                        && (r.physical_width(), r.physical_height())
                            != (canvas.width, canvas.height)
                    {
                        window
                            .resolution
                            .set_physical_resolution(canvas.width, canvas.height);
                    }
                }
            },
        );
    }
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
