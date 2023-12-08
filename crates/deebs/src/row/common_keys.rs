use crate::{Key};
use async_trait::async_trait;

#[async_trait]
pub trait CommonKeys<Tbl> {
    /// Return the set of all keys common to the types in this [`Row`]
    async fn common_keys(
        db: &Tbl,
    ) -> async_std::stream::FromIter<std::collections::btree_set::IntoIter<Key>>;
}