use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use crate::{Event, Records, SelfDuration, TotalDuration, widgets_level_style};

use egui::CollapsingHeader;
use tracing::{callsite::Identifier, Metadata};

pub type TraceInner = HashMap<Identifier, TraceTree>;

const TIMING_BUFFER_LEN: usize = 240;
const TIMING_WINDOW_LEN: usize = 60;

#[derive(Debug)]
pub struct TraceTree {
    is_root: bool,
    self_duration: VecDeque<SelfDuration>,
    total_duration: VecDeque<TotalDuration>,
    metadata: &'static Metadata<'static>,
    records: Records,
    events: Vec<Event>,
    children: TraceInner,
}

impl TraceTree {
    pub fn new(is_root: bool, metadata: &'static Metadata<'static>) -> Self {
        TraceTree {
            is_root,
            self_duration: VecDeque::with_capacity(TIMING_BUFFER_LEN),
            total_duration: VecDeque::with_capacity(TIMING_BUFFER_LEN),
            metadata,
            records: Default::default(),
            events: Default::default(),
            children: Default::default(),
        }
    }

    pub fn metadata(&self) -> &'static Metadata<'static> {
        self.metadata
    }

    pub fn records(&self) -> &Records {
        &self.records
    }

    pub fn records_mut(&mut self) -> &mut Records {
        &mut self.records
    }

    pub fn push_self_duration(&mut self, self_duration: SelfDuration) {
        if self.self_duration.len() >= TIMING_BUFFER_LEN {
            self.self_duration.pop_front();
        }
        self.self_duration.push_back(self_duration);
    }

    pub fn push_total_duration(&mut self, total_duration: TotalDuration) {
        if self.total_duration.len() >= TIMING_BUFFER_LEN {
            self.total_duration.pop_front();
        }
        self.total_duration.push_back(total_duration);
    }

    pub fn children(&self) -> &TraceInner {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut TraceInner {
        &mut self.children
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    pub fn event(&mut self, mut event: Event) {
        event.set_id(self.events.len());
        self.events.push(event);
    }
}

#[cfg(feature = "egui")]
impl egui::Widget for &TraceTree {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            let label = self.metadata.target().to_string() + "::" + self.metadata.name();
            let cached_visuals = ui.style().visuals.clone();
            widgets_level_style(self.metadata.level(), &mut ui.style_mut().visuals);
            CollapsingHeader::new(&label)
                .default_open(self.is_root)
                .id_source(self.metadata.callsite())
                .show(ui, |ui| {
                    ui.style_mut().visuals = cached_visuals;

                    // Timings
                    let has_self_duration = self.self_duration.len() > 0;
                    let has_total_duration = self.total_duration.len() > 0;

                    if has_self_duration || has_total_duration {
                        ui.group(|ui| {
                            ui.heading("Timings");
                            if has_self_duration {
                                let self_duration = self
                                    .self_duration
                                    .iter()
                                    .take(TIMING_WINDOW_LEN)
                                    .map(|dur| **dur)
                                    .sum::<Duration>()
                                    / self.self_duration.len().clamp(1, TIMING_WINDOW_LEN) as u32;

                                ui.label(format!("Self duration: {:?}", self_duration));
                            }

                            if has_total_duration {
                                let total_duration = self
                                    .total_duration
                                    .iter()
                                    .take(TIMING_WINDOW_LEN)
                                    .map(|dur| **dur)
                                    .sum::<Duration>()
                                    / self.total_duration.len().clamp(1, TIMING_WINDOW_LEN) as u32;

                                if total_duration > Duration::default() {
                                    ui.label(format!("Total duration: {:?}", total_duration));
                                }
                            }

                            let plot_self_duration = self.self_duration.len() > 1;
                            let plot_total_duration = self.total_duration.len() > 1;

                            if plot_self_duration | plot_total_duration {
                                let mut plot = egui::plot::Plot::new("Duration")
                                    .show_x(false)
                                    .include_x(0.0)
                                    .allow_zoom(false)
                                    .allow_drag(false);

                                if plot_self_duration {
                                    plot = plot.curve(
                                        egui::plot::Curve::from_values_iter(
                                            self.self_duration.iter().enumerate().map(
                                                |(i, duration)| {
                                                    egui::plot::Value::new(
                                                        i as f64 / TIMING_BUFFER_LEN as f64,
                                                        duration.as_secs_f64(),
                                                    )
                                                },
                                            ),
                                        )
                                        .name("Self Time"),
                                    );
                                }

                                if plot_total_duration {
                                    plot = plot.curve(
                                        egui::plot::Curve::from_values_iter(
                                            self.total_duration.iter().enumerate().map(
                                                |(i, duration)| {
                                                    egui::plot::Value::new(
                                                        i as f64 / TIMING_BUFFER_LEN as f64,
                                                        duration.as_secs_f64(),
                                                    )
                                                },
                                            ),
                                        )
                                        .name("Total Time"),
                                    );
                                }

                                plot.ui(ui);
                            }
                        });
                    }

                    // Records
                    if !self.records.is_empty() {
                        ui.group(|ui| {
                            ui.heading("Records");
                            self.records.ui(ui);
                        });
                    }

                    // Events
                    if !self.events.is_empty() {
                        ui.collapsing("Events", |ui| {
                            egui::ScrollArea::auto_sized().show(ui, |ui| {
                                for event in self.events.iter() {
                                    event.ui(ui);
                                }
                            })
                        });
                    }

                    if !self.children.is_empty() {
                        ui.group(|ui| {
                            ui.heading("Children");
                            for tree in self.children.values() {
                                egui::Widget::ui(tree, ui);
                            }
                        });
                    }
                })
        })
        .response
    }
}
