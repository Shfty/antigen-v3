mod tracer;
pub use tracer::*;

use antigen_egui::EguiUserInterface;
use antigen_rendering::{AlwaysRedraw, OnGpu, RedrawFlag};
use antigen_tracing::TraceRoot;
use antigen_wgpu::{
    wgpu::{PresentMode, TextureFormat, TextureUsage},
    WgpuCommandBuffers, WgpuDevice, WgpuSwapChainFrame,
};
use async_std::sync::Arc;
use deebs::{BorrowSingleton, Insert};
use std::{borrow::Borrow, ops::Deref, sync::atomic::AtomicUsize};

use antigen_components::Label;
use antigen_winit::{
    winit::dpi::{PhysicalSize, Size},
    WindowDescriptor, WinitWindow, WinitWindowEvents,
};
use antigen_winit_wgpu::WinitSwapChain;
use deebs::{BorrowColumn, Table};

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn assemble<'a, T>(table: Arc<T>)
where
    T: Table
        + Borrow<AtomicUsize>
        + BorrowSingleton<TraceRoot>
        + BorrowColumn<Label>
        + BorrowColumn<WgpuDevice>
        + BorrowColumn<WinitWindow>
        + BorrowColumn<WinitSwapChain>
        + BorrowColumn<WgpuSwapChainFrame>
        + BorrowColumn<WgpuCommandBuffers>
        + BorrowColumn<RedrawFlag>
        + BorrowColumn<WinitWindowEvents>
        + BorrowColumn<EguiUserInterface<T>>
        + BorrowColumn<AlwaysRedraw<OnGpu>>
        + Send
        + Sync
        + 'static,
{
    tracing::info!("Assembling tracer_egui...");

    // Create window
    let window_key = table.next_key();

    table.insert(window_key, Label::from("Egui Tracer")).await;

    antigen_winit_wgpu::InsertRow::insert(
        table.deref(),
        window_key,
        (
            WinitWindow::Pending(WindowDescriptor {
                title: Some("Tracer".into()),
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
                tracer(table.clone()),
            )
            .await,
        )
        .await;

    // Setup a WinitWindowEvent sink for the window
    table.insert(window_key, WinitWindowEvents::default()).await;

    // Redraw every frame
    table
        .insert(window_key, AlwaysRedraw::<OnGpu>::default())
        .await;
}
