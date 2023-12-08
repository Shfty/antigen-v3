use std::ops::Deref;

use antigen_tracing::TraceRoot;
use async_std::sync::Arc;
use deebs::{BorrowSingleton, Table, WriteSingleton};
use egui::{CtxRef, Widget};

pub fn tracer<'a, T>(table: Arc<T>) -> impl Fn(&CtxRef) + Send + Sync
where
    T: Table + BorrowSingleton<TraceRoot> + Send + Sync + 'a,
{
    move |context: &CtxRef| {
        egui::CentralPanel::default().show(context, |ui| {
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                async_std::task::block_on(async {
                    let mut trace_root = WriteSingleton::<TraceRoot>::new(table.deref()).await;
                    for tree in trace_root.children().values() {
                        tree.ui(ui);
                    }
                });
            });
        });
    }
}
