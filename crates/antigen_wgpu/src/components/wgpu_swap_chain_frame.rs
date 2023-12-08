use std::fmt::Display;

use wgpu::SwapChainFrame;

/// SwapChain-derived texture view
#[derive(Debug)]
pub enum WgpuSwapChainFrame {
    Pending,
    Ready(SwapChainFrame),
    Dropped,
}

impl Default for WgpuSwapChainFrame {
    fn default() -> Self {
        WgpuSwapChainFrame::Pending
    }
}

impl Display for WgpuSwapChainFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WgpuSwapChainFrame::Pending => f.write_str("Pending"),
            WgpuSwapChainFrame::Ready(_) => f.write_str("Ready"),
            WgpuSwapChainFrame::Dropped => f.write_str("Dropped"),
        }
    }
}

#[cfg(feature = "egui")]
impl egui::Widget for &WgpuSwapChainFrame {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.to_string())
    }
}