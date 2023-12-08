mod event;
mod layers;
mod records;
mod trace_root;
mod trace_tree;
mod env_log_tracer;

pub use event::*;
pub use layers::*;
pub use records::*;
pub use trace_root::*;
pub use trace_tree::*;
pub use env_log_tracer::*;

use egui::{Color32, Visuals};
use tracing::Level;

pub fn widgets_level_style(level: &tracing::metadata::Level, visuals: &mut Visuals) {
    let color = match level {
        &Level::TRACE => Color32::from_gray(30),
        &Level::DEBUG => Color32::from_gray(50),
        &Level::INFO => Color32::from_gray(70),
        &Level::WARN => Color32::from_rgb(127, 70, 0),
        &Level::ERROR => Color32::from_rgb(70, 0, 0),
    };

    visuals.widgets.noninteractive.bg_fill = color;
    visuals.widgets.inactive.bg_fill = color;
    visuals.widgets.hovered.bg_fill = color;
    visuals.widgets.active.bg_fill = color;
}