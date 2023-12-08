use async_std::{io::prelude::WriteExt, sync::Arc};
use deebs::{BorrowColumn, BorrowSingleton, ReadSingleton, Table};
use futures::StreamExt;
use std::{
    error::Error,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{queue_async, CrosstermEvents};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, MouseEvent};

#[derive(Debug)]
pub struct CrosstermMouseEvents(Vec<MouseEvent>);

pub async fn set_mouse_capture_enabled(enabled: bool) -> Result<(), Box<dyn Error>> {
    let mut stdout = async_std::io::stdout();

    if enabled {
        queue_async!(&mut stdout, EnableMouseCapture).await?;
    } else {
        queue_async!(&mut stdout, DisableMouseCapture).await?;
    }

    stdout.flush().await?;

    Ok(())
}

impl CrosstermMouseEvents {
    pub fn new(events: Vec<MouseEvent>) -> Self {
        async_std::task::block_on(set_mouse_capture_enabled(true)).unwrap();
        CrosstermMouseEvents(events)
    }
}

impl Default for CrosstermMouseEvents {
    fn default() -> Self {
        CrosstermMouseEvents::new(Default::default())
    }
}

impl Drop for CrosstermMouseEvents {
    fn drop(&mut self) {
        async_std::task::block_on(set_mouse_capture_enabled(false)).unwrap();
    }
}

impl Deref for CrosstermMouseEvents {
    type Target = Vec<MouseEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CrosstermMouseEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for CrosstermMouseEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        for (i, mouse) in self.0.iter().enumerate() {
            string += format!(
                "{}",
                match mouse.kind {
                    crossterm::event::MouseEventKind::Down(_) => "Down",
                    crossterm::event::MouseEventKind::Up(_) => "Up",
                    crossterm::event::MouseEventKind::Drag(_) => "Drag",
                    crossterm::event::MouseEventKind::Moved => "Moved",
                    crossterm::event::MouseEventKind::ScrollDown => "ScrollDown",
                    crossterm::event::MouseEventKind::ScrollUp => "ScrollUp",
                }
            )
            .as_str();
            if i < self.0.len() - 1 {
                string += "\n";
            }
        }
        f.write_str(&string)
    }
}

pub async fn run_mouse_events_system<T>(table: Arc<T>)
where
    T: Table + BorrowSingleton<CrosstermEvents> + BorrowColumn<CrosstermMouseEvents> + Send + Sync,
{
    let events = ReadSingleton::new(table.deref()).await;

    let mut stream = table.keys::<CrosstermMouseEvents>().await;
    while let Some(key) = stream.next().await {
        let mut mouse_events = table.get_mut::<CrosstermMouseEvents>(&key).await.unwrap();
        for event in events.iter() {
            if let crossterm::event::Event::Mouse(mouse) = event {
                mouse_events.push(*mouse);
            }
        }
    }
}
