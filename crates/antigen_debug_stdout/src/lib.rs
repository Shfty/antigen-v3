//! stdout-based debugging trait and system for rendering [`Table`]s as ASCII.

use std::ops::{Deref, DerefMut};

use antigen_crossterm::{
    crossterm::{
        cursor::{Hide, MoveTo},
        event::KeyEvent,
        style::{style, PrintStyledContent},
    },
    queue_async, CrosstermKeyEvents,
};
use async_std::{io::prelude::WriteExt, sync::Arc};
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ToCell};
use deebs::{
    macros::CommonKeys, macros::Row, BorrowColumn, BorrowSingleton, BorrowView, CommonKeys, Map,
    ReadView, Row, Table, ToStringMapper, WriteCell, WriteSingleton,
};
use futures::StreamExt;

use antigen_crossterm::crossterm::{
    self,
    terminal::{Clear, ClearType},
};

#[derive(Row, CommonKeys)]
struct StdoutDebugRow<'a> {
    key_events: WriteCell<'a, CrosstermKeyEvents>,
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run<D, T>(table: Arc<T>)
where
    T: Table + BorrowSingleton<D> + BorrowColumn<CrosstermKeyEvents> + Send + Sync,
    D: StdoutDebug<T> + Send + Sync + 'static,
{
    let mut debug = WriteSingleton::<D>::new(table.deref()).await;

    let mut stream = StdoutDebugRow::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let StdoutDebugRow { mut key_events } = StdoutDebugRow::new(table.deref(), &key).await;

        for key_event in key_events.drain(..) {
            StdoutDebug::handle_key_input(debug.deref_mut(), key_event).await;
        }
        StdoutDebug::run(debug.deref(), table.clone()).await;
    }
}

#[async_trait::async_trait]
pub trait StdoutDebug<T> {
    /// Perform any necessary conditional logic and invoke antigen_debug_stdout over the desired type.
    async fn run(&self, table: Arc<T>);

    /// Respond to incoming key input.
    async fn handle_key_input(&mut self, key: KeyEvent);
}

pub async fn run_table_system<'a, R, T>(table: &'a T)
where
    R: Row<'a, T> + Map<ToStringMapper, Item = String> + 'a,
    T: Table + BorrowView<R>,
{
    let (width, height) = crossterm::terminal::size().unwrap();

    let mut comfy_table = comfy_table::Table::new();
    comfy_table.set_header(
        std::iter::once(&"Key")
            .chain(R::HEADER)
            .map(|str| str.chars().filter(|char| *char != ' ').collect::<String>()),
    );

    {
        let view = ReadView::new(table).await;
        let mut stream = view.keys();
        while let Some(key) = stream.next().await {
            let view = R::new(table, key).await;
            let mut row = comfy_table::Row::new();
            row.add_cell(key.to_cell());
            for string in view.map() {
                row.add_cell(string.unwrap_or_default().to_cell());
            }
            comfy_table.add_row(row);
        }
    }

    comfy_table.load_preset(UTF8_FULL);
    comfy_table.apply_modifier(UTF8_ROUND_CORNERS);

    let lines = comfy_table.to_string();

    let mut stdout = async_std::io::stdout();
    queue_async!(&mut stdout, Hide).await.unwrap();
    queue_lines(&mut stdout, 0, 0, width, height, lines)
        .await
        .unwrap();
    queue_async!(&mut stdout, Clear(ClearType::FromCursorDown))
        .await
        .unwrap();
    stdout.flush().await.unwrap();
}

fn truncate(width: usize) -> impl Fn(&str) -> &str {
    move |s: &str| match s.char_indices().nth(width) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

pub async fn queue_lines(
    stdout: &mut async_std::io::Stdout,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    lines: String,
) -> Result<u16, Box<dyn std::error::Error>> {
    let mut iter = lines.split('\n').map(truncate(width as usize)).enumerate();

    let mut max_y = 0;
    while let Some((i, line)) = iter.next() {
        antigen_crossterm::queue_async!(
            stdout,
            MoveTo(x, y + i as u16),
            PrintStyledContent(style(line))
        )
        .await?;
        max_y = max_y.max(y + i as u16);

        if i as u16 == height - y - 2 {
            break;
        }
    }

    if let Some((_, line)) = iter.last() {
        antigen_crossterm::queue_async!(
            stdout,
            MoveTo(x, max_y + 1),
            PrintStyledContent(style(line))
        )
        .await?;
        max_y += 1;
    }

    Ok(max_y)
}
