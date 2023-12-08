use std::ops::Deref;

use async_std::sync::{RwLockReadGuard};

use crate::{BorrowSingleton};

/// A view into a [`Singleton`].
#[derive(Debug)]
pub struct ReadSingleton<'a, T> {
    singleton_guard: RwLockReadGuard<'a, T>,
}

impl<'a, T> ReadSingleton<'a, T> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(table)))]
    pub async fn new<DB>(table: &'a DB) -> ReadSingleton<'a, T>
    where
        T: 'a,
        DB: BorrowSingleton<T>,
    {
        let singleton = table.borrow();
        let singleton_guard = singleton.read().await;
        ReadSingleton { singleton_guard }
    }

    pub fn singleton(&'a self) -> &'a T {
        self.singleton_guard.deref()
    }
}

impl<'a, T> Deref for ReadSingleton<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.singleton()
    }
}
