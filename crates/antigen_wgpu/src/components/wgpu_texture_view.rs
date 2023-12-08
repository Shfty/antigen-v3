use std::fmt::Display;

use wgpu::TextureView;

/// Raw texture view
#[derive(Debug)]
pub enum WgpuTextureView {
    Pending,
    Ready(TextureView),
    Dropped,
}

impl Display for WgpuTextureView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WgpuTextureView::Pending => f.write_str("Pending"),
            WgpuTextureView::Ready(_) => f.write_str("Ready"),
            WgpuTextureView::Dropped => f.write_str("Dropped"),
        }
    }
}

#[cfg(feature = "egui")]
impl egui::Widget for &WgpuTextureView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.to_string())
    }
}
