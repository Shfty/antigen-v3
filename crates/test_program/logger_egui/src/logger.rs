use std::ops::Deref;

use antigen_log::LogRecords;
use async_std::sync::Arc;
use deebs::{BorrowSingleton, ReadSingleton, Table};
use egui::CtxRef;

pub fn logger<'a, T>(table: Arc<T>) -> impl Fn(&CtxRef) + Send + Sync
where
    T: Table + BorrowSingleton<LogRecords> + Send + Sync + 'a,
{
    move |context: &CtxRef| {
        egui::CentralPanel::default().show(context, |ui| {
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                egui::Grid::new("logger").striped(true).show(ui, |ui| {
                    ui.label("level");
                    ui.label("args");
                    ui.end_row();

                    async_std::task::block_on(async {
                        let log_records = ReadSingleton::<LogRecords>::new(table.deref()).await;

                        let mut last = None;
                        for log in log_records.iter() {
                            let response = ui.label(format!("{:?}", log.level));
                            ui.label(&log.args);
                            ui.end_row();
                            last = Some(response);
                        }
                        if let Some(last) = last {
                            last.scroll_to_me(egui::Align::Max);
                        }
                    });
                });
            });
        });
    }
}
