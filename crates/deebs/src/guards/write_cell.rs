use std::ops::{Deref, DerefMut};
use std::{marker::PhantomPinned, pin::Pin, ptr::NonNull};

use crate::{BorrowColumn, ColumnCollection, ReadColumn, Key};

/// A view into one of the [`Cell`]s of a [`Column`]
#[derive(Debug)]
pub struct WriteCell<'a, T>(Pin<Box<WriteCellInner<'a, T>>>);

impl<'a, T> WriteCell<'a, T> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(table)))]
    pub async fn new<DB>(table: &'a DB, key: &Key) -> Option<WriteCell<'a, T>>
    where
        T: 'a,
        DB: BorrowColumn<T>,
    {
        let inner = WriteCellInner::new(table, key).await;
        inner.map(WriteCell)
    }

    pub fn cell(&self) -> &T {
        self.0.deref().deref()
    }

    pub fn cell_mut(&mut self) -> &mut T {
        unsafe {
            let mut_ref = Pin::as_mut(&mut self.0);
            Pin::get_unchecked_mut(mut_ref).deref_mut()
        }
    }

    #[allow(dead_code)]
    pub fn column(&'a self) -> &'a ColumnCollection<T> {
        self.0.deref().column_guard.deref()
    }
}

impl<'a, T> Deref for WriteCell<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.cell()
    }
}

impl<'a, T> DerefMut for WriteCell<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.cell_mut()
    }
}

/// Inner workings of [`CellView`].
/// Self-referential struct that holds both the column and cell read guards
#[derive(Debug)]
struct WriteCellInner<'a, T> {
    column_guard: ReadColumn<'a, T>,
    item_guard: NonNull<T>,
    _pin: PhantomPinned,
}

unsafe impl<'a, T> Send for WriteCellInner<'a, T> {}
unsafe impl<'a, T> Sync for WriteCellInner<'a, T> {}

impl<'a, T> WriteCellInner<'a, T> {
    pub async fn new<DB>(db: &'a DB, key: &Key) -> Option<Pin<Box<WriteCellInner<'a, T>>>>
    where
        T: 'a,
        DB: BorrowColumn<T>,
    {
        let column_guard = ReadColumn::new(db).await;

        if !column_guard.contains_key(key) {
            return None
        }

        let guard = WriteCellInner {
            column_guard,
            item_guard: NonNull::dangling(),
            _pin: PhantomPinned,
        };
        
        let mut boxed = Box::pin(guard);

        let item_guard = NonNull::from(
            boxed
                .column_guard
                .get(key)
                .unwrap()
                .write()
                .await
                .deref(),
        );

        unsafe {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).item_guard = item_guard;
        }

        Some(boxed)
    }
}

impl<'a, T> Deref for WriteCellInner<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.item_guard.as_ref() }
    }
}

impl<'a, T> DerefMut for WriteCellInner<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.item_guard.as_mut() }
    }
}
