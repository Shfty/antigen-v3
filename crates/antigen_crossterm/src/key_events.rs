use async_std::sync::Arc;
use deebs::{BorrowColumn, BorrowSingleton, ReadSingleton, Table};
use futures::StreamExt;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crossterm::event::KeyEvent;

use crate::CrosstermEvents;

#[derive(Debug, Default)]
pub struct CrosstermKeyEvents(Vec<KeyEvent>);

impl Deref for CrosstermKeyEvents {
    type Target = Vec<KeyEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CrosstermKeyEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for CrosstermKeyEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        for (i, key) in self.0.iter().enumerate() {
            string += format!("{:?}", key.code).as_str();
            if i < self.0.len() - 1 {
                string += "\n";
            }
        }
        f.write_str(&string)
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_key_events_system<T>(table: Arc<T>)
where
    T: Table + BorrowSingleton<CrosstermEvents> + BorrowColumn<CrosstermKeyEvents> + Send + Sync,
{
    let events = ReadSingleton::new(table.deref()).await;

    let mut stream = table.keys::<CrosstermKeyEvents>().await;
    while let Some(key) = stream.next().await {
        let mut key_events = table.get_mut::<CrosstermKeyEvents>(&key).await.unwrap();
        for event in events.iter() {
            if let crossterm::event::Event::Key(key) = event {
                key_events.push(*key);
            }
        }
    }
}
