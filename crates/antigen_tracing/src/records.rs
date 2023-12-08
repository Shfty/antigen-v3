use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use tracing::field::{Field, Visit};

#[derive(Debug, Default, Clone)]
pub struct RecordMap<T>(HashMap<&'static str, T>);

impl<T> IntoIterator for RecordMap<T> {
    type Item = (&'static str, T);

    type IntoIter = std::collections::hash_map::IntoIter<&'static str, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Deref for RecordMap<T> {
    type Target = HashMap<&'static str, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RecordMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "egui")]
impl<T> egui::Widget for &RecordMap<T>
where
    T: ToString,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            for (key, value) in self.iter() {
                ui.horizontal(|ui| {
                    ui.label(*key);
                    ui.label(value.to_string());
                });
            }
        })
        .response
    }
}

#[derive(Debug, Default, Clone)]
pub struct Records {
    debug: RecordMap<String>,
    i64: RecordMap<i64>,
    u64: RecordMap<u64>,
    bool: RecordMap<bool>,
    str: RecordMap<String>,
    error: RecordMap<String>,
}

impl Records {
    pub fn is_empty(&self) -> bool {
        self.debug.is_empty()
            && self.i64.is_empty()
            && self.u64.is_empty()
            && self.bool.is_empty()
            && self.str.is_empty()
            && self.error.is_empty()
    }

    pub fn join(&mut self, other: Records) -> &mut Self {
        let Records {
            debug,
            i64,
            u64,
            bool,
            str,
            error
        } = other;

        self.debug.extend(debug.into_iter());
        self.i64.extend(i64.into_iter());
        self.u64.extend(u64.into_iter());
        self.bool.extend(bool.into_iter());
        self.str.extend(str.into_iter());
        self.error.extend(error.into_iter());

        self
    }
}

#[cfg(feature = "egui")]
impl egui::Widget for &Records {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            if !self.debug.is_empty() {
                self.debug.ui(ui);
            }

            if !self.i64.is_empty() {
                self.i64.ui(ui);
            }

            if !self.u64.is_empty() {
                self.u64.ui(ui);
            }

            if !self.bool.is_empty() {
                self.bool.ui(ui);
            }

            if !self.str.is_empty() {
                self.str.ui(ui);
            }

            if !self.debug.is_empty() {
                self.error.ui(ui);
            }
        })
        .response
    }
}

impl Visit for Records {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.debug.insert(field.name(), format!("{:?}", value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.i64.insert(field.name(), value);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.u64.insert(field.name(), value);
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.bool.insert(field.name(), value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.str.insert(field.name(), value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.error.insert(field.name(), format!("{}", value));
    }
}
