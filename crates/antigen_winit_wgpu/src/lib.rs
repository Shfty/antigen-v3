//! Glue code for bridging [`antigen_winit`] and [`antigen_wgpu`]

mod system;
mod wgpu_swap_chain_frame;
mod winit_swap_chain;

use antigen_rendering::RedrawFlag;
use antigen_wgpu::{WgpuCommandBuffers, WgpuSwapChainFrame};
use antigen_winit::WinitWindow;
pub use system::*;
pub use wgpu_swap_chain_frame::*;
pub use winit_swap_chain::*;

use deebs::{ReadCell, macros::{Row, Insert, CommonKeys}};

#[derive(Row, Insert, CommonKeys)]
pub struct InsertRow<'a> {
    window: ReadCell<'a, WinitWindow>,
    redraw_flag: ReadCell<'a, RedrawFlag>,
    swap_chain: ReadCell<'a, WinitSwapChain>,
    swap_chain_frame: ReadCell<'a, WgpuSwapChainFrame>,
    command_buffers: ReadCell<'a, WgpuCommandBuffers>,
}
