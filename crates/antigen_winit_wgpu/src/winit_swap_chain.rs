use async_std::sync::Arc;
use deebs::{
    macros::{CommonKeys, Row},
    BorrowColumn, CommonKeys, ReadCell, Row, Table, WriteCell,
};

use futures::StreamExt;

use std::{
    collections::BTreeMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use antigen_wgpu::{
    wgpu::{Surface, SwapChain, SwapChainDescriptor},
    Render, WgpuCommandBuffers, WgpuSwapChainFrame,
};
use antigen_winit::winit::window::WindowId;

#[derive(Debug)]
pub enum WinitSwapChain {
    Pending(SwapChainDescriptor),
    Ready {
        window_id: WindowId,
        swap_chain_desc: SwapChainDescriptor,
    },
    Dropped,
}

impl Display for WinitSwapChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WinitSwapChain::Pending(_) => f.write_str("Pending"),
            WinitSwapChain::Ready { .. } => f.write_str("Ready"),
            WinitSwapChain::Dropped => f.write_str("Dropped"),
        }
    }
}

#[cfg(feature = "egui")]
impl egui::Widget for &WinitSwapChain {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.to_string())
    }
}

#[derive(Debug, Default)]
pub struct WgpuSwapChains(BTreeMap<WindowId, (Surface, SwapChain)>);

impl Deref for WgpuSwapChains {
    type Target = BTreeMap<WindowId, (Surface, SwapChain)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WgpuSwapChains {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_swap_chain_render_system<F, T>(table: Arc<T>)
where
    T: Table
        + BorrowColumn<WgpuSwapChainFrame>
        + BorrowColumn<F>
        + BorrowColumn<WgpuCommandBuffers>
        + Send
        + Sync,
    F: Render + Send + Sync + 'static,
{
    #[derive(Row, CommonKeys)]
    struct RenderRow<'a, F>
    where
        F: Render + Send + Sync + 'static,
    {
        swap_chain_frame: ReadCell<'a, WgpuSwapChainFrame>,
        renderer: WriteCell<'a, F>,
        command_buffers: WriteCell<'a, WgpuCommandBuffers>,
    }

    let mut stream = RenderRow::<F>::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let RenderRow {
            swap_chain_frame,
            mut renderer,
            mut command_buffers,
        } = RenderRow::<F>::new(table.deref(), &key).await;

        if let WgpuSwapChainFrame::Ready(frame) = swap_chain_frame.deref() {
            command_buffers.push(renderer.render(&frame.output.view));
        }
    }
}
