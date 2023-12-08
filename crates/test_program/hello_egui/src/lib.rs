mod inspector;
use antigen_rendering::RedrawFlag;
pub use inspector::*;

use antigen_egui::EguiUserInterface;
use antigen_wgpu::{
    wgpu::{PresentMode, TextureFormat, TextureUsage},
    WgpuCommandBuffers, WgpuDevice, WgpuSwapChainFrame,
};
use async_std::sync::Arc;
use deebs::Insert;
use std::{borrow::Borrow, ops::Deref, sync::atomic::AtomicUsize};

use antigen_winit::{
    winit::dpi::{PhysicalSize, Size},
    WindowDescriptor, WinitWindow, WinitWindowEvents,
};
use antigen_winit_wgpu::WinitSwapChain;
use deebs::{BorrowColumn, Table};

use antigen_components::Label;

use hello_quads::{QuadPosition, QuadSize};

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn assemble<T>(table: Arc<T>)
where
    T: Table
        + Borrow<AtomicUsize>
        + BorrowColumn<Label>
        + BorrowColumn<WgpuDevice>
        + BorrowColumn<WinitWindow>
        + BorrowColumn<WinitSwapChain>
        + BorrowColumn<WgpuSwapChainFrame>
        + BorrowColumn<WgpuCommandBuffers>
        + BorrowColumn<RedrawFlag>
        + BorrowColumn<WinitWindowEvents>
        + BorrowColumn<EguiUserInterface<T>>
        + BorrowColumn<QuadPosition>
        + BorrowColumn<QuadSize>
        + Send
        + Sync
        + 'static,
{
    tracing::info!("Assembling hello_egui...");

    // Create window
    let window_key = table.next_key();

    table.insert(window_key, Label::from("Hello Egui")).await;

    antigen_winit_wgpu::InsertRow::insert(
        table.deref(),
        window_key,
        (
            WinitWindow::Pending(WindowDescriptor {
                title: Some("Hello Egui".into()),
                visible: Some(false),
                inner_size: Some(Size::Physical(PhysicalSize::<u32>::new(640, 480))),
                ..Default::default()
            }),
            RedrawFlag(true),
            WinitSwapChain::Pending(antigen_wgpu::wgpu::SwapChainDescriptor {
                usage: TextureUsage::RENDER_ATTACHMENT,
                format: TextureFormat::Bgra8UnormSrgb,
                width: 640,
                height: 480,
                present_mode: PresentMode::Mailbox,
            }),
            WgpuSwapChainFrame::default(),
            WgpuCommandBuffers::default(),
        ),
    )
    .await;

    // Setup a render pass for the window
    table
        .insert(
            window_key,
            EguiUserInterface::boxed(
                table.clone(),
                TextureFormat::Bgra8UnormSrgb,
                hello_quads_inspector(table.clone()),
            )
            .await,
        )
        .await;

    // Setup a WinitWindowEvent sink for the window
    table.insert(window_key, WinitWindowEvents::default()).await;
}
