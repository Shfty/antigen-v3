use std::{fmt::Display, ops::{Deref, DerefMut}};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "egui", derive(deebs::macros::Widget))]
pub struct Label(&'static str);

impl From<&'static str> for Label {
    fn from(label: &'static str) -> Self {
        Label(label)
    }
}

impl From<Label> for &'static str {
    fn from(label: Label) -> Self {
        label.0
    }
}

impl Deref for Label {
    type Target = &'static str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Label {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
