use std::ops::Deref;

use async_std::sync::RwLockReadGuard;

use crate::{BorrowColumn, ColumnCollection};

/// A view into a [`Column`]
#[derive(Debug)]
pub struct ReadColumn<'a, T> {
    column_guard: RwLockReadGuard<'a, ColumnCollection<T>>,
}

impl<'a, T> ReadColumn<'a, T> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(table)))]
    pub async fn new<DB>(table: &'a DB) -> ReadColumn<'a, T>
    where
        T: 'a,
        DB: BorrowColumn<T>,
    {
        let column = table.borrow();
        let column_guard = column.read().await;
        ReadColumn { column_guard }
    }

    pub fn column(&'a self) -> &'a ColumnCollection<T> {
        self.column_guard.deref()
    }
}

impl<'a, T> Deref for ReadColumn<'a, T> {
    type Target = ColumnCollection<T>;

    fn deref(&self) -> &Self::Target {
        self.column()
    }
}
