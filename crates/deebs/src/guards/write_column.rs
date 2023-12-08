use std::ops::{Deref, DerefMut};

use async_std::sync::RwLockWriteGuard;

use crate::{BorrowColumn, ColumnCollection};

/// A view into a [`Column`]
#[derive(Debug)]
pub struct WriteColumn<'a, T> {
    column_guard: RwLockWriteGuard<'a, ColumnCollection<T>>,
}

impl<'a, T> WriteColumn<'a, T> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(table)))]
    pub async fn new<DB>(table: &'a DB) -> WriteColumn<'a, T>
    where
        T: 'a,
        DB: BorrowColumn<T>,
    {
        let column = table.borrow();
        let column_guard = column.write().await;
        WriteColumn { column_guard }
    }

    pub fn column(&self) -> &ColumnCollection<T> {
        self.column_guard.deref()
    }

    pub fn column_mut(&mut self) -> &mut ColumnCollection<T> {
        self.column_guard.deref_mut()
    }
}

impl<'a, T> Deref for WriteColumn<'a, T> {
    type Target = ColumnCollection<T>;

    fn deref(&self) -> &Self::Target {
        self.column()
    }
}

impl<'a, T> DerefMut for WriteColumn<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.column_mut()
    }
}