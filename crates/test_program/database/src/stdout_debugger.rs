//! Database-specific [`StdoutDebug`] implementation

use std::{fmt::Display, ops::Deref};

use crate::MyTable;
use antigen_crossterm::crossterm;
use antigen_crossterm::{
    crossterm::{
        cursor::{Hide, Show},
        event::{KeyCode, KeyEvent},
        terminal::{Clear, ClearType},
    },
    queue_async, CrosstermKeyEvents,
};
use antigen_debug_stdout::{queue_lines, run_table_system, StdoutDebug};
use antigen_log::LogRecords;
use async_std::{self, io::prelude::WriteExt, sync::Arc};
use async_trait;
use deebs::{ReadSingleton, Table};
use antigen_components::Label;

#[derive(Debug, Copy, Clone)]
enum DebugTab {
    Log,
    MyTable,
    Integrator,
    HelloTriangle,
}

impl Default for DebugTab {
    fn default() -> Self {
        DebugTab::MyTable
    }
}

impl DebugTab {
    pub fn next(self) -> Self {
        match self {
            DebugTab::Log => DebugTab::MyTable,
            DebugTab::MyTable => DebugTab::Integrator,
            DebugTab::Integrator => DebugTab::HelloTriangle,
            DebugTab::HelloTriangle => DebugTab::Log,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            DebugTab::Log => DebugTab::HelloTriangle,
            DebugTab::MyTable => DebugTab::Log,
            DebugTab::Integrator => DebugTab::MyTable,
            DebugTab::HelloTriangle => DebugTab::Integrator,
        }
    }
}

impl Display for DebugTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DebugTab::Log => "Log",
            DebugTab::MyTable => "Debug",
            DebugTab::Integrator => "Integrator",
            DebugTab::HelloTriangle => "Hello Triangle",
        })
    }
}

#[derive(Debug, Clone)]
pub struct StdoutDebugger {
    tab: DebugTab,
}

impl Default for StdoutDebugger {
    fn default() -> Self {
        async_std::task::block_on(async move {
            let mut stdout = async_std::io::stdout();
            queue_async!(&mut stdout, Clear(ClearType::All), Hide)
                .await
                .unwrap();
            stdout.flush().await.unwrap();
        });

        StdoutDebugger {
            tab: Default::default(),
        }
    }
}

impl Drop for StdoutDebugger {
    fn drop(&mut self) {
        async_std::task::block_on(async move {
            let mut stdout = async_std::io::stdout();
            queue_async!(&mut stdout, Show).await.unwrap();
            stdout.flush().await.unwrap();
        });
    }
}

#[async_trait::async_trait]
impl StdoutDebug<MyTable<'static>> for StdoutDebugger {
    async fn run(&self, table: Arc<MyTable<'static>>) {
        let mut stdout = async_std::io::stdout();

        queue_async!(&mut stdout, Hide).await.unwrap();

        match self.tab {
            DebugTab::Log => {
                let (width, height) = crossterm::terminal::size().unwrap();

                let log = ReadSingleton::<LogRecords>::new(table.deref()).await;
                let string = log
                    .iter()
                    .skip(log.len().checked_sub(height as usize).unwrap_or_default())
                    .map(ToString::to_string)
                    .map(|string| {
                        string
                            .chars()
                            .collect::<Vec<_>>()
                            .chunks(width as usize)
                            .map(|chunk| chunk.iter().collect::<String>() + "\n")
                            .collect::<String>()
                    })
                    .collect();

                queue_lines(&mut stdout, 0, 0, width, height, string)
                    .await
                    .unwrap();
            }
            DebugTab::MyTable => {
                run_table_system::<crate::DebugRow, _>(table.deref()).await
            }
            DebugTab::Integrator => {
                run_table_system::<integrator::DebugRow, _>(table.deref()).await
            }
            DebugTab::HelloTriangle => {
                run_table_system::<hello_triangle::DebugRow, _>(table.deref()).await
            }
        }

        stdout.flush().await.unwrap();
    }

    async fn handle_key_input(&mut self, key: KeyEvent) {
        let clear = match key.code {
            KeyCode::PageUp => {
                self.tab = self.tab.prev();
                true
            }
            KeyCode::PageDown => {
                self.tab = self.tab.next();
                true
            }
            _ => false,
        };

        if clear {
            let mut stdout = async_std::io::stdout();
            queue_async!(&mut stdout, Clear(ClearType::All))
                .await
                .unwrap();
            stdout.flush().await.unwrap();
        }
    }
}

pub async fn assemble(table: Arc<MyTable<'static>>) {
    let key = table.next_key();
    table.insert(key, Label::from("stdout Debugger")).await;
    table.insert(key, CrosstermKeyEvents::default()).await;
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run(table: Arc<MyTable<'static>>) {
    antigen_debug_stdout::run::<StdoutDebugger, _>(table).await;
}
