use futures::StreamExt;

use std::ops::{Deref, DerefMut};

use async_std::sync::Arc;
use deebs::{
    macros::{CommonKeys, Row},
    BorrowColumn, BorrowSingleton, CommonKeys, ReadCell, ReadSingleton, Row, Table, WriteCell,
    WriteSingleton,
};

use antigen_winit::{WinitWindow, WinitWindows};

use crate::{WgpuSwapChains, WinitSwapChain};
use antigen_wgpu::{WgpuDevice, WgpuInstance};

/// Create, resize or drop [`WgpuSwapChain`] instances based on an associated [`WinitWindow`]
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn create_swap_chains<T>(table: Arc<T>)
where
    T: Table
        + BorrowSingleton<WinitWindows>
        + BorrowSingleton<WgpuInstance>
        + BorrowSingleton<WgpuSwapChains>
        + BorrowColumn<WgpuDevice>
        + BorrowColumn<WinitWindow>
        + BorrowColumn<WinitSwapChain>
        + Send
        + Sync,
{
    let mut swap_chains = WriteSingleton::<WgpuSwapChains>::new(table.deref()).await;

    let instance = ReadSingleton::<WgpuInstance>::new(table.deref()).await;
    let instance = if let WgpuInstance::Ready(instance) = instance.deref() {
        instance
    } else {
        return;
    };

    let device = table
        .get::<WgpuDevice>(
            &table
                .keys::<WgpuDevice>()
                .await
                .next()
                .await
                .expect("No Wgpu cell in table."),
        )
        .await
        .unwrap();

    #[derive(Row, CommonKeys)]
    struct MaintainSwapChainsRow<'a> {
        window: ReadCell<'a, WinitWindow>,
        swap_chain: WriteCell<'a, WinitSwapChain>,
    }

    let mut stream = MaintainSwapChainsRow::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let MaintainSwapChainsRow {
            window,
            mut swap_chain,
        } = MaintainSwapChainsRow::new(table.deref(), &key).await;

        match window.deref() {
            WinitWindow::Ready { window_id, .. } => {
                let windows = ReadSingleton::<WinitWindows>::new(table.deref()).await;
                let window = windows.get(window_id).expect("Invalid Window ID.");

                match swap_chain.deref_mut() {
                    WinitSwapChain::Pending(swap_chain_desc) => {
                        let size = window.inner_size();
                        swap_chain_desc.width = size.width;
                        swap_chain_desc.height = size.height;

                        let surface = unsafe { instance.create_surface(window) };
                        let sc = device.device.create_swap_chain(&surface, swap_chain_desc);

                        swap_chains.insert(*window_id, (surface, sc));

                        *swap_chain = WinitSwapChain::Ready {
                            window_id: *window_id,
                            swap_chain_desc: swap_chain_desc.clone(),
                        };
                    }
                    WinitSwapChain::Ready {
                        window_id,
                        swap_chain_desc,
                    } => {
                        let size = window.inner_size();
                        if (size.width > 0 && size.height > 0)
                            && (swap_chain_desc.width != size.width
                                || swap_chain_desc.height != size.height)
                        {
                            // Recreate the swap chain with the new size
                            swap_chain_desc.width = size.width;
                            swap_chain_desc.height = size.height;

                            let (surface, _) = swap_chains
                                .remove(window_id)
                                .expect("Invalid SwapChain Window ID.");

                            let swap_chain =
                                device.device.create_swap_chain(&surface, swap_chain_desc);

                            swap_chains.insert(*window_id, (surface, swap_chain));
                        }
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    }
}

/// Create, resize or drop [`WinitSwapChain`] instances based on an associated [`WinitWindow`]
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn drop_swap_chains<T>(table: Arc<T>)
where
    T: Table
        + BorrowSingleton<WinitWindows>
        + BorrowSingleton<WgpuInstance>
        + BorrowSingleton<WgpuSwapChains>
        + BorrowColumn<WgpuDevice>
        + BorrowColumn<WinitWindow>
        + BorrowColumn<WinitSwapChain>
        + Send
        + Sync,
{
    let windows = ReadSingleton::<WinitWindows>::new(table.deref()).await;
    let mut swap_chains = WriteSingleton::<WgpuSwapChains>::new(table.deref()).await;

    let mut stream = table.keys::<WinitSwapChain>().await;
    while let Some(key) = stream.next().await {
        let mut swap_chain = table.get_mut::<WinitSwapChain>(&key).await.unwrap();
        if let WinitSwapChain::Ready { window_id, .. } = swap_chain.deref() {
            if !windows.contains_key(window_id) {
                swap_chains
                    .remove(&window_id)
                    .expect("Invalid SwapChain Window ID.");
                *swap_chain = WinitSwapChain::Dropped;
            }
        }
    }
}
