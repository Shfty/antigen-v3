use std::ops::Deref;

use async_std::{self, sync::Arc};
use futures::StreamExt;
use deebs::{
    macros::{CommonKeys, Row},
    BorrowColumn, CommonKeys, Row, Table, WriteCell,
};
use egui::{CtxRef, DragValue, ScrollArea, Slider};

use super::{QuadPosition, QuadSize};

pub fn hello_quads_inspector<T>(table: Arc<T>) -> impl Fn(&CtxRef) + Send + Sync + 'static
where
    T: Table + BorrowColumn<QuadPosition> + BorrowColumn<QuadSize> + Send + Sync + 'static,
{
    move |context: &CtxRef| {
        let table = table.clone();
        egui::CentralPanel::default().show(context, move |ui| {
            ScrollArea::auto_sized().show(ui, |ui| {
                async_std::task::block_on(async move {
                    #[derive(Row, CommonKeys)]
                    struct QuadRow<'a> {
                        quad_position: WriteCell<'a, QuadPosition>,
                        quad_size: WriteCell<'a, QuadSize>,
                    }

                    let mut stream = QuadRow::common_keys(table.deref()).await;
                    while let Some(key) = stream.next().await {
                        let QuadRow {
                            mut quad_position,
                            mut quad_size,
                        } = async_std::task::block_on(QuadRow::new(table.deref(), &key));

                        ui.collapsing(format!("Rect {}", key), |ui| {
                            ui.label("Position");
                            ui.add(Slider::new(&mut quad_position.x, -1.0..=1.0).fixed_decimals(2));
                            ui.add(
                                DragValue::new(&mut quad_position.x)
                                    .fixed_decimals(2)
                                    .speed(0.01),
                            );

                            ui.add(Slider::new(&mut quad_position.y, -1.0..=1.0).fixed_decimals(2));
                            ui.add(
                                DragValue::new(&mut quad_position.y)
                                    .fixed_decimals(2)
                                    .speed(0.01),
                            );

                            ui.separator();

                            ui.label("Size");
                            ui.add(Slider::new(&mut quad_size.w, 0.0..=1.0).fixed_decimals(2));
                            ui.add(
                                DragValue::new(&mut quad_size.w)
                                    .fixed_decimals(2)
                                    .speed(0.01),
                            );

                            ui.add(Slider::new(&mut quad_size.h, 0.0..=1.0).fixed_decimals(2));
                            ui.add(
                                DragValue::new(&mut quad_size.h)
                                    .fixed_decimals(2)
                                    .speed(0.01),
                            );
                        });
                    }
                })
            });
        });
    }
}
