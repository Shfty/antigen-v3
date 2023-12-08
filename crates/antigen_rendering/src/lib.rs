//! General-purpose rendering machinery for `antigen`

use async_std::sync::Arc;
use deebs::{
    macros::{CommonKeys, Row},
    BorrowColumn, CommonKeys, ReadCell, Row, Table, WriteCell,
};
use futures::StreamExt;
use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// General-purpose component for flagging an entity as needing to be redrawn.
/// Serves no purpose in isolation - used by integrations like winit and wgpu to redraw windows and surfaces.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RedrawFlag(pub bool);

impl Display for RedrawFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[cfg(feature="egui")]
impl egui::Widget for &RedrawFlag {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.to_string())
    }
}

impl Deref for RedrawFlag {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RedrawFlag {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// General-purpose component for flagging an entity that should redraw every frame.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlwaysRedraw<T = ()>(PhantomData<T>);

impl<T> Display for AlwaysRedraw<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AlwaysRedraw")
    }
}

#[cfg(feature = "egui")]
impl<T> egui::Widget for &AlwaysRedraw<T> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.to_string())
    }
}

/// General-purpose flag indicating some operation should run on the CPU.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnCpu;

/// General-purpose flag indicating some operation should run on the GPU.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnGpu;

/// Set the redraw flag every frame.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_always_redraw_system<F, T>(table: Arc<T>)
where
    T: Table + BorrowColumn<AlwaysRedraw<F>> + BorrowColumn<RedrawFlag> + Send + Sync,
    F: Send + Sync + 'static,
{
    #[derive(Row, CommonKeys)]
    struct RedrawRow<'a, F>
    where
        F: 'static,
    {
        _always: ReadCell<'a, AlwaysRedraw<F>>,
        flag: WriteCell<'a, RedrawFlag>,
    }

    let mut stream = RedrawRow::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let RedrawRow { mut flag, .. } = RedrawRow::new(table.deref(), &key).await;
        **flag = true;
    }
}
