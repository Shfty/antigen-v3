use crate::{Key, Table};
use async_trait::async_trait;
use futures::Stream;

#[async_trait]
pub trait Remove<Tbl> {
    /// Remove values of this row's types from a [`Table`]
    async fn remove(db: &Tbl, key: Key)
    where
        Tbl: Table;

    /// Remove values of this row's types from a [`Table`]
    async fn remove_multi<I>(db: &Tbl, keys: I)
    where
        Tbl: Table,
        I: Stream<Item = Key> + Send;
}
