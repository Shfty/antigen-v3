use egui::{CollapsingHeader};
use tracing::Metadata;

use crate::Records;

#[derive(Debug, Clone)]
pub struct Event {
    id: usize,
    metadata: &'static Metadata<'static>,
    records: Records,
}

impl Event {
    pub fn new(metadata: &'static Metadata<'static>) -> Self {
        Event {
            id: Default::default(),
            metadata,
            records: Default::default(),
        }
    }

    pub fn set_id(&mut self, id: usize) {
        self.id = id;
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
}

#[cfg(feature = "egui")]
mod widget {
    use crate::widgets_level_style;

    use super::*;

    impl egui::Widget for &Event {
        fn ui(self, ui: &mut egui::Ui) -> egui::Response {
            widgets_level_style(self.metadata().level(), &mut ui.style_mut().visuals);
            CollapsingHeader::new(self.metadata.name())
                .id_source(self.id)
                .show(ui, |ui| self.records.ui(ui))
                .header_response
        }
    }
}
