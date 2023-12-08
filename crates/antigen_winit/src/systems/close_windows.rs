use std::{ops::Deref, sync::Arc};

use deebs::{BorrowColumn, BorrowSingleton, ReadSingleton, Table};
use futures::StreamExt;

use crate::{WinitWindow, WinitWindows};

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_close_window_system<'a, T>(table: Arc<T>)
where
    T: Table + BorrowSingleton<WinitWindows> + BorrowColumn<WinitWindow> + Send + Sync + 'static,
{
    let windows = ReadSingleton::new(table.deref()).await;

    let mut keys = table.keys::<WinitWindow>().await;
    while let Some(key) = keys.next().await {
        let mut winit_window = table.get_mut::<WinitWindow>(&key).await.unwrap();
        if let WinitWindow::Ready { window_id, .. } = winit_window.deref() {
            if !windows.contains_key(window_id) {
                *winit_window = WinitWindow::Closed;
            }
        }
    }
}
