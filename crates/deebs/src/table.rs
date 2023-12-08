use std::{
    any::TypeId,
    borrow::Borrow,
    collections::BTreeSet,
    sync::atomic::{AtomicUsize, Ordering},
};

use async_std::stream::FromIter;

use crate::{BorrowColumn, Key, ReadCell, ReadColumn, WriteCell, WriteColumn};

/// A type that holds [`View`] structs.
#[async_trait::async_trait]
pub trait Table {
    type Key: Ord + Copy;

    async fn insert<T>(&self, key: Key, value: T)
    where
        Self: Sized + BorrowColumn<T>,
        T: Send + Sync + 'static,
    {
        WriteColumn::new(self).await.insert(key, value.into());
        self.update_views(&[std::any::TypeId::of::<T>()]).await;
    }

    async fn insert_auto<T>(&self, value: T) -> Key
    where
        Self: Sized + Borrow<AtomicUsize> + BorrowColumn<T>,
        T: Send + Sync + 'static,
    {
        let key = self.next_key();
        WriteColumn::new(self).await.insert(key, value.into());
        self.update_views(&[std::any::TypeId::of::<T>()]).await;
        key
    }

    async fn insert_multi<I, T>(&self, values: I)
    where
        Self: Sized + BorrowColumn<T>,
        I: Iterator<Item = (Key, T)> + Send + Sync,
        T: Send + Sync + 'static,
    {
        {
            let mut column = WriteColumn::new(self).await;
            for (key, value) in values {
                column.insert(key, value.into());
            }
        }
        self.update_views(&[std::any::TypeId::of::<T>()]).await;
    }

    async fn insert_auto_multi<I, T>(&self, values: I) -> Vec<Key>
    where
        Self: Sized + Borrow<AtomicUsize> + BorrowColumn<T>,
        I: Iterator<Item = T> + Send + Sync,
        T: Send + Sync + 'static,
    {
        let mut keys = vec![];
        {
            let mut column = WriteColumn::new(self).await;
            for value in values {
                let key = self.next_key();
                column.insert(key, value.into());
                keys.push(key);
            }
        }
        self.update_views(&[std::any::TypeId::of::<T>()]).await;
        keys
    }

    async fn remove<T>(&self, key: Key)
    where
        Self: Sized + BorrowColumn<T>,
        T: Send + Sync + 'static,
    {
        {
            let mut column = WriteColumn::new(self).await;
            column.remove(&key);
        }
        self.update_views(&[std::any::TypeId::of::<T>()]).await;
    }

    async fn remove_multi<T, I>(&self, keys: I)
    where
        Self: Sized + BorrowColumn<T>,
        I: Iterator<Item = Key> + Send + Sync,
        T: Send + Sync + 'static,
    {
        {
            let mut column = WriteColumn::new(self).await;
            for key in keys {
                column.remove(&key);
            }
        }
        self.update_views(&[std::any::TypeId::of::<T>()]).await;
    }

    async fn get<T>(&self, key: &Key) -> Option<ReadCell<T>>
    where
        Self: Sized + BorrowColumn<T>,
        T: Send + Sync,
    {
        ReadCell::new(self, key).await
    }

    async fn get_mut<T>(&self, key: &Key) -> Option<WriteCell<T>>
    where
        Self: Sized + BorrowColumn<T>,
        T: Send + Sync,
    {
        WriteCell::new(self, key).await
    }

    async fn keys<T>(&self) -> FromIter<std::collections::btree_set::IntoIter<Key>>
    where
        Self: Sized + BorrowColumn<T>,
        T: Send + Sync,
    {
        async_std::stream::from_iter(
            ReadColumn::new(self)
                .await
                .keys()
                .copied()
                .collect::<BTreeSet<_>>()
                .into_iter(),
        )
    }

    fn next_key(&self) -> Key
    where
        Self: Borrow<AtomicUsize>,
    {
        let next = self.borrow();
        let next = next.fetch_add(1, Ordering::Relaxed);
        next.into()
    }

    /// Check each held [`View`]'s type and update it if its valid keys have changed.
    async fn update_views(&self, type_ids: &[TypeId]);
}
