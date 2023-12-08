pub mod stdout_debugger;

use antigen_crossterm::{CrosstermEvents, CrosstermKeyEvents};
use antigen_egui::EguiUserInterface;
use antigen_rendering::{AlwaysRedraw, OnCpu, OnGpu, RedrawFlag};
use antigen_tracing::TraceRoot;
use antigen_winit_wgpu::{WgpuSwapChains, WinitSwapChain};
use borrow_derive::Borrow;
use std::{fmt::Debug, sync::atomic::AtomicUsize};

use deebs::{
    macros::{CommonKeys, Map, Row, Table, Widgets},
    Column, ReadCell, Singleton, View, WriteCell,
};

use antigen_components::Label;
use antigen_wgpu::{
    WgpuCommandBuffers, WgpuDevice, WgpuInstance, WgpuQueue, WgpuRenderer, WgpuSwapChainFrame,
    WgpuTextureView,
};
use antigen_winit::{
    WinitMainEvents, WinitRedrawEvents, WinitWindow, WinitWindowEvents, WinitWindows,
};

use hello_quads::{QuadPosition, QuadSize};
use stdout_debugger::StdoutDebugger;

use antigen_log::LogRecords;

/// A user-created table struct holding columns
#[derive(Debug, Default, Borrow, Table)]
pub struct MyTable<'a> {
    // Primary Key
    key_head: AtomicUsize,

    // Singletons
    crossterm_events: Singleton<CrosstermEvents>,

    winit_window_pool: Singleton<WinitWindows>,
    winit_main_events: Singleton<WinitMainEvents>,
    winit_redraw_events: Singleton<WinitRedrawEvents>,

    wgpu_instance: Singleton<WgpuInstance>,
    wgpu_swap_chain_pool: Singleton<WgpuSwapChains>,
    log_records: Singleton<LogRecords>,
    stdout_debug: Singleton<StdoutDebugger>,

    trace_root: Singleton<TraceRoot>,

    // Columns
    labels: Column<Label>,

    // Test
    bools: Column<bool>,
    ints: Column<i32>,
    floats: Column<f32>,
    chars: Column<char>,
    strs: Column<&'static str>,
    strings: Column<String>,
    quad_positions: Column<QuadPosition>,
    quad_sizes: Column<QuadSize>,

    // crossterm
    crossterm_key_events: Column<CrosstermKeyEvents>,

    // rendering
    redraw_flags: Column<RedrawFlag>,
    always_redraw_on_cpus: Column<AlwaysRedraw<OnCpu>>,
    always_redraw_on_gpus: Column<AlwaysRedraw<OnGpu>>,

    // winit
    winit_window_events: Column<WinitWindowEvents>,
    windows: Column<WinitWindow>,

    // wgpu
    wgpu_devices: Column<WgpuDevice>,
    wgpu_queues: Column<WgpuQueue>,
    wgpu_swap_chains: Column<WinitSwapChain>,
    wgpu_swap_chain_frames: Column<WgpuSwapChainFrame>,
    wgpu_texture_views: Column<WgpuTextureView>,
    wgpu_renderers: Column<WgpuRenderer>,
    wgpu_command_buffers: Column<WgpuCommandBuffers>,

    // egui
    egui_user_interface: Column<EguiUserInterface<MyTable<'static>>>,

    // Views
    debug_view: View<DebugRow<'a>>,
    integrator_view: View<integrator::DebugRow<'a>>,
    hello_triangle_view: View<hello_triangle::DebugRow<'a>>,
}

/// A user-created row query result holding references to all cells in a table.
#[derive(Debug, Row, CommonKeys, Map, Widgets)]
pub struct DebugRow<'a> {
    pub label: Option<ReadCell<'a, Label>>,

    pub winit_window: Option<ReadCell<'a, WinitWindow>>,
    pub winit_swap_chain: Option<ReadCell<'a, WinitSwapChain>>,

    pub redraw_flag: Option<ReadCell<'a, RedrawFlag>>,
    pub always_redraw_on_cpu: Option<ReadCell<'a, AlwaysRedraw<OnCpu>>>,
    pub always_redraw_on_gpu: Option<ReadCell<'a, AlwaysRedraw<OnGpu>>>,

    pub wgpu_swap_chain_frame: Option<ReadCell<'a, WgpuSwapChainFrame>>,
    pub wgpu_texture_view: Option<ReadCell<'a, WgpuTextureView>>,

    pub egui_user_interface: Option<ReadCell<'a, EguiUserInterface<MyTable<'static>>>>,
    pub wgpu_renderer: Option<ReadCell<'a, WgpuRenderer>>,

    pub wgpu_command_buffers: Option<ReadCell<'a, WgpuCommandBuffers>>,

    pub quad_position: Option<WriteCell<'a, QuadPosition>>,
    pub quad_size: Option<WriteCell<'a, QuadSize>>,
}
