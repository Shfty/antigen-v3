use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use async_std::sync::Arc;
use crossterm::event::Event;
use deebs::{BorrowSingleton, Table, WriteSingleton};

#[derive(Debug, Default)]
pub struct CrosstermEvents(Vec<Event>);

impl Deref for CrosstermEvents {
    type Target = Vec<Event>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CrosstermEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for CrosstermEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Events")
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_events_system<T>(table: Arc<T>)
where
    T: Table + BorrowSingleton<CrosstermEvents> + Send + Sync,
{
    let mut events = WriteSingleton::new(table.deref()).await;

    events.clear();
    while let Ok(true) = crossterm::event::poll(Default::default()) {
        events.push(crossterm::event::read().unwrap());
    }
}
