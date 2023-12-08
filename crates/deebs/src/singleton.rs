use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut},
};

use async_std::sync::RwLock;

#[derive(Debug, Default)]
pub struct Singleton<T>(RwLock<T>);

impl<T> Deref for Singleton<T> {
    type Target = RwLock<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Singleton<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait BorrowSingleton<V>: Borrow<Singleton<V>> {}
impl<V, T> BorrowSingleton<V> for T where T: Borrow<Singleton<V>> {}
