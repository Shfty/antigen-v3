use crate::{CommonKeys, Table};
use async_std::sync::RwLock;
use futures::StreamExt;
use std::{borrow::Borrow, collections::BTreeSet, marker::PhantomData};

use crate::Key;

/// A thread-safe set of valid keys for a given [`Row`] type.
#[derive(Debug)]
pub struct View<R> {
    pub keys: RwLock<BTreeSet<Key>>,
    _phantom: PhantomData<R>
}

unsafe impl<T> Send for View<T> {}
unsafe impl<T> Sync for View<T> {}

impl<R> Default for View<R> {
    fn default() -> Self {
        View {
            keys: Default::default(),
            _phantom: Default::default()
        }
    }
}

impl<R> View<R> {
    pub async fn update<T>(&self, db: &T)
    where
        T: Table,
        std::collections::BTreeSet<Key>: std::iter::Extend<<T as Table>::Key>,
        R: CommonKeys<T>,
    {
        let mut keys = self.keys.write().await;
        *keys = R::common_keys(db).await.collect().await;
    }

    pub async fn keys(
        &self,
    ) -> async_std::stream::FromIter<std::collections::btree_set::IntoIter<Key>> {
        async_std::stream::from_iter(self.keys.read().await.clone().into_iter())
    }
}

pub trait BorrowView<R>: Borrow<View<R>> {}
impl<R, T> BorrowView<R> for T where T: Borrow<View<R>> {}
