//! Struct-based async table-row database.

mod column;
mod guards;
mod key;
mod row;
mod singleton;
mod table;
mod view;

pub use column::*;
pub use guards::*;
pub use key::*;
pub use row::*;
pub use singleton::*;
pub use table::*;
pub use view::*;

pub use deebs_macros as macros;

use async_std::sync::RwLock;
use std::collections::HashMap;

pub type ColumnCollection<T> = HashMap<Key, RwLock<T>, fnv::FnvBuildHasher>;

pub fn slice_stream<T>(slice: &[T]) -> impl futures::Stream<Item = &T> {
    async_std::stream::from_iter(slice)
}

pub fn array_stream<T, const N: usize>(array: [T; N]) -> impl futures::Stream<Item = T> {
    async_std::stream::from_iter(std::array::IntoIter::new(array))
}
