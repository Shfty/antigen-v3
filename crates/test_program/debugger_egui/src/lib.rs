mod debugger;
pub use debugger::*;

use antigen_egui::{EguiUserInterface, Widgets};
use antigen_rendering::{AlwaysRedraw, OnGpu, RedrawFlag};
use antigen_wgpu::{
    wgpu::{PresentMode, TextureFormat, TextureUsage},
    WgpuCommandBuffers, WgpuDevice, WgpuSwapChainFrame,
};
use async_std::sync::Arc;
use deebs::{BorrowView, Insert, Row};
use std::{borrow::Borrow, ops::Deref, sync::atomic::AtomicUsize};

use antigen_components::Label;
use antigen_winit::{
    winit::dpi::{PhysicalSize, Size},
    WindowDescriptor, WinitWindow, WinitWindowEvents,
};
use antigen_winit_wgpu::WinitSwapChain;
use deebs::{BorrowColumn, Table};

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn assemble<'a, R, T>(table: Arc<T>)
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
        + BorrowColumn<AlwaysRedraw<OnGpu>>
        + BorrowView<R>
        + Send
        + Sync
        + 'static,
    R: Row<'a, T> + Widgets + 'static,
{
    tracing::info!("Assembling debugger_egui...");

    // Create window
    let window_key = table.next_key();

    table.insert(window_key, Label::from("Egui Debugger")).await;

    antigen_winit_wgpu::InsertRow::insert(
        table.deref(),
        window_key,
        (
            WinitWindow::Pending(WindowDescriptor {
                title: Some("Debugger".into()),
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
                debugger::<R, T>(table.clone()),
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
