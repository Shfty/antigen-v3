use std::ops::Deref;

use deebs::{
    BorrowView,  ReadView, Row, Table, 
};

use async_std::sync::Arc;
use futures::StreamExt;
use egui::{CtxRef};
use antigen_egui::Widgets;

pub fn debugger<'a, R, T>(table: Arc<T>) -> impl Fn(&CtxRef) + Send + Sync
where
    T: Table + BorrowView<R> + Send + Sync + 'a,
    R: Row<'a, T> + Widgets + 'static,
{
    move |context: &CtxRef| {
        egui::CentralPanel::default().show(context, |ui| {
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                egui::Grid::new("debugger").striped(true).show(ui, |ui| {
                    for cell in std::iter::once(&"Key")
                        .chain(R::HEADER)
                        .map(|str| str.chars().filter(|char| *char != ' ').collect::<String>())
                    {
                        ui.label(cell);
                    }
                    ui.end_row();

                    async_std::task::block_on(async {
                        let view = ReadView::new(table.deref()).await;

                        let keys = view.keys().collect::<Vec<_>>().await;
                        for key in keys {
                            let ptr = Arc::<T>::as_ptr(&table);
                            let table = unsafe { ptr.as_ref().unwrap() };
                            let mut view = R::new(table, key).await;
                            ui.label(key.to_string());
                            view.widgets(ui);
                            ui.end_row();
                        }
                    });
                });
            });
        });
    }
}
