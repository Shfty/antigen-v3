use std::ops::{Deref, DerefMut};

use async_std::sync::RwLockWriteGuard;

use crate::BorrowSingleton;

/// A view into one a [`Column`]
#[derive(Debug)]
pub struct WriteSingleton<'a, T> {
    singleton_guard: RwLockWriteGuard<'a, T>,
}

impl<'a, T> WriteSingleton<'a, T> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(table)))]
    pub async fn new<DB>(table: &'a DB) -> WriteSingleton<'a, T>
    where
        T: 'a,
        DB: BorrowSingleton<T>,
    {
        let singleton = table.borrow();
        let singleton_guard = singleton.write().await;
        WriteSingleton { singleton_guard }
    }

    pub fn singleton(&self) -> &T {
        self.singleton_guard.deref()
    }

    pub fn singleton_mut(&mut self) -> &mut T {
        self.singleton_guard.deref_mut()
    }
}

impl<'a, T> Deref for WriteSingleton<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.singleton()
    }
}

impl<'a, T> DerefMut for WriteSingleton<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.singleton_mut()
    }
}
