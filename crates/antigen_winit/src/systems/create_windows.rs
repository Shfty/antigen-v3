use std::{ops::Deref, sync::Arc};

use deebs::{BorrowColumn, BorrowSingleton, Table, WriteSingleton};
use futures::StreamExt;
use winit::{
    event_loop::EventLoopWindowTarget,
    window::{WindowBuilder, WindowId},
};

use crate::{WinitWindow, WinitWindows};

pub struct CreateWindowSystem;

impl CreateWindowSystem {
    pub async fn run<T>(table: Arc<T>, window_target: &EventLoopWindowTarget<()>) -> Vec<WindowId>
    where
        T: Table + BorrowSingleton<WinitWindows> + BorrowColumn<WinitWindow> + Sync + 'static,
    {
        let mut windows = WriteSingleton::new(table.deref()).await;

        let mut keys = table.keys::<WinitWindow>().await;
        let mut out_keys = vec![];
        while let Some(key) = keys.next().await {
            let mut winit_window = table.get_mut::<WinitWindow>(&key).await.unwrap();
            if let WinitWindow::Pending(window_desc) = winit_window.deref() {
                let builder = WindowBuilder::from(window_desc);
                let window = builder.build(window_target).unwrap();
                let window_id = window.id();
                windows.insert(window_id, window);

                *winit_window = WinitWindow::Ready {
                    window_desc: window_desc.clone(),
                    window_id: window_id,
                };

                out_keys.push(window_id);
            }
        }
        out_keys
    }
}
