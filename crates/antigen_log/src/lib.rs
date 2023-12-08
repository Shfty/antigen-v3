use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    time::SystemTime,
};

use async_std::sync::Arc;

use deebs::{BorrowSingleton, Table, WriteSingleton};
use log::{Level, Log, Metadata};

#[derive(Debug)]
pub struct Logger<T> {
    table: Arc<T>,
}

impl<T> Logger<T> {
    pub fn new(table: Arc<T>) -> Self {
        Logger { table }
    }
}

#[derive(Debug, Default, Clone)]
pub struct LogRecords(Vec<LogRecord>);

impl Display for LogRecords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LogRecords")
    }
}

impl Deref for LogRecords {
    type Target = Vec<LogRecord>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LogRecords {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct LogRecord {
    pub time: SystemTime,
    pub level: Level,
    pub target: String,
    pub args: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl Display for LogRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msec = self
            .time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let sec = msec / 1000;
        let mins = sec / 60;
        let hours = mins / 60;

        f.write_fmt(format_args!(
            "{:02}:{:02}:{:02}:{:03} | {} | {}:{} | {}",
            hours % 24,
            mins % 60,
            sec % 60,
            msec % 1000,
            self.level,
            self.target,
            self.line
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            self.args
        ))
    }
}

impl<'a> From<&log::Record<'a>> for LogRecord {
    fn from(record: &log::Record<'a>) -> Self {
        let time = SystemTime::now();
        let metadata = record.metadata();
        let level = metadata.level();
        let target = metadata.target().to_string();
        let args = record.args().to_string();
        let module_path = record.module_path().map(ToString::to_string);
        let file = record.file().map(ToString::to_string);
        let line = record.line();

        LogRecord {
            time,
            level,
            target,
            args,
            module_path,
            file,
            line,
        }
    }
}

impl<T> Log for Logger<T>
where
    T: Table + BorrowSingleton<LogRecords> + Send + Sync,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        async_std::task::block_on(async move {
            let mut records = WriteSingleton::<LogRecords>::new(self.table.deref()).await;
            records.push(record.into());
        });
    }

    fn flush(&self) {}
}
