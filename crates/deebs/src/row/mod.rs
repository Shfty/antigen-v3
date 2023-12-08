mod insert;
mod remove;
mod common_keys;
mod map;

pub use insert::*;
pub use remove::*;
pub use common_keys::*;
pub use map::*;

use std::any::TypeId;

use crate::{Key};
use async_trait::async_trait;

/// A type that can act as a virtual table row, containing views into the underlying cell data.
#[async_trait]
pub trait Row<'a, Tbl> {
    /// Slice of string slices representing this row's type names.
    const HEADER: &'static [&'static str];

    // Return a vec of [`TypeId`]s representing this row's inner types.
    fn inner_types() -> Vec<TypeId>;

    /// Create a new row
    async fn new(db: &'a Tbl, key: &Key) -> Self;
}
