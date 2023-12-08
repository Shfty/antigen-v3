use std::{fmt::Display, ops::{Deref, DerefMut}};

/// A [`Table`] key.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(usize);

impl From<usize> for Key {
    fn from(key: usize) -> Self {
        Key(key)
    }
}

impl From<Key> for usize {
    fn from(key: Key) -> Self {
        key.0
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for Key {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Key {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
