use std::{collections::BTreeSet, ops::Deref};

use async_std::sync::RwLockReadGuard;
use futures::Stream;

use crate::{BorrowView, Key};

/// A read guard over a [`View`].
#[derive(Debug)]
pub struct ReadView<'a> {
    keys_guard: RwLockReadGuard<'a, BTreeSet<Key>>,
}

impl<'a> ReadView<'a> {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(table)))]
    pub async fn new<DB, R>(table: &'a DB) -> ReadView<'a>
    where
        DB: BorrowView<R>,
        R: 'a,
    {
        let column = table.borrow();
        let keys_guard = column.keys.read().await;
        ReadView { keys_guard }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
    pub fn keys_cloned(&self) -> impl Stream<Item = Key> {
        async_std::stream::from_iter(self.keys_guard.deref().clone().into_iter())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
    pub fn keys(&self) -> impl Stream<Item = &Key> {
        async_std::stream::from_iter(self.keys_guard.iter())
    }
}

impl<'a> Deref for ReadView<'a> {
    type Target = BTreeSet<Key>;

    fn deref(&self) -> &Self::Target {
        self.keys_guard.deref()
    }
}
