use std::ops::Deref;

use futures::StreamExt;

use antigen_rendering::RedrawFlag;
use antigen_wgpu::WgpuSwapChainFrame;
use async_std::sync::Arc;
use deebs::{
    macros::{CommonKeys, Row},
    BorrowColumn, BorrowSingleton, CommonKeys, ReadCell, ReadSingleton, Row, Table, WriteCell,
};

use crate::{WgpuSwapChains, WinitSwapChain};

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_swap_chain_frame_system<T>(table: Arc<T>)
where
    T: Table
        + BorrowSingleton<WgpuSwapChains>
        + BorrowColumn<RedrawFlag>
        + BorrowColumn<WinitSwapChain>
        + BorrowColumn<WgpuSwapChainFrame>
        + Send
        + Sync,
{
    #[derive(Row, CommonKeys)]
    struct SwapChainFrameRow<'a> {
        redraw_flag: ReadCell<'a, RedrawFlag>,
        swap_chain: ReadCell<'a, WinitSwapChain>,
        swap_chain_frame: WriteCell<'a, WgpuSwapChainFrame>,
    }

    let mut stream = SwapChainFrameRow::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let SwapChainFrameRow {
            redraw_flag,
            swap_chain,
            mut swap_chain_frame,
        } = SwapChainFrameRow::new(table.deref(), &key).await;

        if **redraw_flag {
            if let WinitSwapChain::Ready { window_id, .. } = swap_chain.deref() {
                let swap_chains = ReadSingleton::<WgpuSwapChains>::new(table.deref()).await;
                let (_, swap_chain) = swap_chains
                    .get(window_id)
                    .expect("Invalid SwapChain Window ID.");
                *swap_chain_frame = WgpuSwapChainFrame::Ready(
                    swap_chain
                        .get_current_frame()
                        .expect("Failed to get next SwapChain frame."),
                );
            }
        }
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_swap_chain_present_system<T>(table: Arc<T>)
where
    T: Table + BorrowColumn<WgpuSwapChainFrame> + Send + Sync,
{
    let mut stream = table.keys::<WgpuSwapChainFrame>().await;
    while let Some(key) = stream.next().await {
        let mut swap_chain_frame = table.get_mut::<WgpuSwapChainFrame>(&key).await.unwrap();
        if let WgpuSwapChainFrame::Ready(_) = swap_chain_frame.deref() {
            *swap_chain_frame = WgpuSwapChainFrame::Dropped;
        }
    }
}
