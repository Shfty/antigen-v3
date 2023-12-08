use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use antigen_rendering::RedrawFlag;
use async_std::sync::Arc;
use deebs::{macros::CommonKeys, macros::Row, BorrowColumn, CommonKeys, Row, Table, WriteCell};
use futures::StreamExt;
use wgpu::CommandBuffer;

use crate::WgpuQueue;
#[derive(Debug, Default)]
#[cfg_attr(feature = "egui", derive(deebs::macros::Widget))]
pub struct WgpuCommandBuffers(Vec<CommandBuffer>);

impl Display for WgpuCommandBuffers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Buffers: {}", self.len()))
    }
}

impl Deref for WgpuCommandBuffers {
    type Target = Vec<CommandBuffer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WgpuCommandBuffers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Clear all command buffers.
/// Useful to call at the start of rendering to prevent duplication when triggering redraws from an event loop that ticks at its own rate.
/// (ex. When triggering redraws from the game loop.)
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_clear_command_buffers_system<T>(table: Arc<T>)
where
    T: Table + BorrowColumn<WgpuCommandBuffers> + Send + Sync,
{
    let mut stream = table.keys::<WgpuCommandBuffers>().await;
    while let Some(key) = stream.next().await {
        let mut command_buffers = WriteCell::<WgpuCommandBuffers>::new(table.deref(), &key)
            .await
            .unwrap();
        command_buffers.clear();
    }
}

/// Flush all command buffers to the wgpu queue for rendering.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_flush_command_buffers_system<T>(table: Arc<T>)
where
    T: Table
        + BorrowColumn<WgpuQueue>
        + BorrowColumn<WgpuCommandBuffers>
        + BorrowColumn<RedrawFlag>
        + Send
        + Sync,
{
    let queue = table
        .get::<WgpuQueue>(
            &table
                .keys::<WgpuQueue>()
                .await
                .next()
                .await
                .expect("No WgpuQueue in table."),
        )
        .await
        .unwrap();

    #[derive(Row, CommonKeys)]
    struct FlushRow<'a> {
        command_buffers: WriteCell<'a, WgpuCommandBuffers>,
        redraw_flag: Option<WriteCell<'a, RedrawFlag>>,
    }

    let mut stream = FlushRow::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let FlushRow {
            mut command_buffers,
            redraw_flag,
        } = FlushRow::new(table.deref(), &key).await;

        if !command_buffers.is_empty() {
            queue.submit(command_buffers.drain(..));

            if let Some(mut redraw_flag) = redraw_flag {
                **redraw_flag = false;
            }
        }
    }
}
