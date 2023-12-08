use async_std::sync::Arc;
use deebs::{BorrowColumn, BorrowSingleton, ReadSingleton, Table};
use futures::StreamExt;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::CrosstermEvents;

#[derive(Debug, Default)]
pub struct CrosstermResizeEvents(Vec<(u16, u16)>);

impl Deref for CrosstermResizeEvents {
    type Target = Vec<(u16, u16)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CrosstermResizeEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for CrosstermResizeEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        for (i, (width, height)) in self.0.iter().enumerate() {
            string += format!("{}, {}", width, height).as_str();
            if i < self.0.len() - 1 {
                string += "\n";
            }
        }
        f.write_str(&string)
    }
}

pub async fn run_resize_events_system<T>(table: Arc<T>)
where
    T: Table + BorrowSingleton<CrosstermEvents> + BorrowColumn<CrosstermResizeEvents> + Send + Sync,
{
    let events = ReadSingleton::new(table.deref()).await;

    let mut stream = table.keys::<CrosstermResizeEvents>().await;
    while let Some(key) = stream.next().await {
        let mut resize_events = table.get_mut::<CrosstermResizeEvents>(&key).await.unwrap();
        for event in events.iter() {
            if let crossterm::event::Event::Resize(width, height) = event {
                resize_events.push((*width, *height));
            }
        }
    }
}
