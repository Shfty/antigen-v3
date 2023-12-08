use std::{borrow::Borrow, sync::atomic::AtomicUsize};

use crate::{Key, Table};
use async_trait::async_trait;

#[async_trait]
pub trait Insert<Tbl> {
    /// The value type used to insert cells of this row into a [`Table`]
    type Insert: Send;

    /// Insert values of this row's types into a [`Table`]
    async fn insert(db: &Tbl, key: Key, row: Self::Insert)
    where
        Tbl: Table;

    /// Insert values of this row's types into a [`Table`]
    async fn insert_auto(db: &Tbl, row: Self::Insert) -> Key
    where
        Tbl: Table + Borrow<AtomicUsize>;

    /// Insert values of this row's types into a [`Table`]
    async fn insert_multi<I>(db: &Tbl, rows: I)
    where
        Tbl: Table,
        I: Iterator<Item = (Key, Self::Insert)> + Send;

    /// Insert values of this row's types into a [`Table`]
    async fn insert_auto_multi<I>(db: &Tbl, rows: I) -> Vec<Key>
    where
        Tbl: Table + Borrow<AtomicUsize>,
        I: Iterator<Item = Self::Insert> + Send;
}
