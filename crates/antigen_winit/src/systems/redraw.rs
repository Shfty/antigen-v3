use std::ops::Deref;

use async_std::sync::Arc;
use deebs::{
    macros::CommonKeys, macros::Row, BorrowColumn, BorrowSingleton, CommonKeys, ReadCell,
    ReadSingleton, Row, Table, WriteCell, WriteSingleton,
};
use futures::StreamExt;

use crate::{WinitRedrawEvents, WinitWindow, WinitWindows};
use antigen_rendering::RedrawFlag;

#[derive(Row, CommonKeys)]
struct RedrawRow<'a> {
    redraw_flag: ReadCell<'a, RedrawFlag>,
    window: ReadCell<'a, WinitWindow>,
}

/// Issues a redraw request for any entities with both a [`WinitWindow`] and a set [`RedrawFlag`]
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_redraw_system<T>(table: Arc<T>)
where
    T: Table
        + BorrowSingleton<WinitWindows>
        + BorrowColumn<WinitWindow>
        + BorrowColumn<RedrawFlag>
        + Send
        + Sync,
{
    let mut stream = RedrawRow::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let RedrawRow {
            redraw_flag,
            window,
        } = RedrawRow::new(table.deref(), &key).await;

        if let WinitWindow::Ready { window_id, .. } = window.deref() {
            if **redraw_flag {
                let windows = ReadSingleton::<WinitWindows>::new(table.deref()).await;
                let window = &windows[window_id];
                window.request_redraw();
                window.set_visible(true);
            }
        }
    }
}

/// Sets [`RedrawFlag`] to true on entities that have a [`WinitWindow`] with a pending [`WinitRedrawEvent`]
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_redraw_flag_system<T>(table: Arc<T>)
where
    T: Table
        + BorrowSingleton<WinitRedrawEvents>
        + BorrowColumn<WinitWindow>
        + BorrowColumn<RedrawFlag>
        + Send
        + Sync,
{
    let mut events = WriteSingleton::<WinitRedrawEvents>::new(table.deref()).await;

    #[derive(Row, CommonKeys)]
    struct RedrawRow<'a> {
        window: ReadCell<'a, WinitWindow>,
        redraw_flag: WriteCell<'a, RedrawFlag>,
    }

    let mut stream = RedrawRow::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let RedrawRow {
            window,
            mut redraw_flag,
        } = RedrawRow::new(table.deref(), &key).await;

        if let WinitWindow::Ready { window_id, .. } = window.deref() {
            if events.iter().any(|event| event.0 == *window_id) {
                **redraw_flag = true;
            }
        }
    }

    events.clear();
}
