use std::ops::Deref;
use std::{marker::PhantomPinned, pin::Pin, ptr::NonNull};

use crate::{BorrowColumn, ColumnCollection, ReadColumn, Key};

/// A view into one of the [`Cell`]s of a [`Column`]
#[derive(Debug)]
pub struct ReadCell<'a, T>(Pin<Box<ReadCellInner<'a, T>>>);

impl<'a, T> ReadCell<'a, T> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(table)))]
    pub async fn new<DB>(table: &'a DB, key: &Key) -> Option<ReadCell<'a, T>>
    where
        T: 'a,
        DB: BorrowColumn<T>,
    {
        let inner = ReadCellInner::new(table, key).await;
        inner.map(ReadCell)
    }

    pub fn cell(&'a self) -> &'a T {
        self.0.deref().deref()
    }

    #[allow(dead_code)]
    pub fn column(&'a self) -> &'a ColumnCollection<T> {
        self.0.deref().column_guard.deref()
    }
}

impl<'a, T> Deref for ReadCell<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.cell()
    }
}

/// Inner workings of [`CellView`].
/// Self-referential struct that holds both the column and cell read guards
#[derive(Debug)]
struct ReadCellInner<'a, T> {
    column_guard: ReadColumn<'a, T>,
    item_guard: NonNull<T>,
    _pin: PhantomPinned,
}

unsafe impl<'a, T> Send for ReadCellInner<'a, T> {}
unsafe impl<'a, T> Sync for ReadCellInner<'a, T> {}

impl<'a, T> ReadCellInner<'a, T> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(table)))]
    pub async fn new<DB>(table: &'a DB, key: &Key) -> Option<Pin<Box<ReadCellInner<'a, T>>>>
    where
        T: 'a,
        DB: BorrowColumn<T>,
    {
        let column_guard = ReadColumn::new(table).await;

        if !column_guard.contains_key(key) {
            return None;
        }

        let guard = ReadCellInner {
            column_guard,
            item_guard: NonNull::dangling(),
            _pin: PhantomPinned,
        };
        
        let mut boxed = Box::pin(guard);

        let item_guard =
            NonNull::from(boxed.column_guard.get(&key).unwrap().read().await.deref());

        unsafe {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).item_guard = item_guard;
        }

        Some(boxed)
    }
}

impl<'a, T> Deref for ReadCellInner<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.item_guard.as_ref() }
    }
}
