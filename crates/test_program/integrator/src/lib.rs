//! Simple numerical integrator

use std::{borrow::Borrow, ops::Deref, sync::atomic::AtomicUsize};

use antigen_components::Label;
use async_std::sync::Arc;
use deebs::{
    array_stream, macros::Insert, macros::Remove, BorrowColumn, BorrowView, Insert, ReadCell,
    ReadView, Remove, Row, Table,
};
use deebs::{
    macros::{CommonKeys, Map, Row},
    WriteCell,
};
use futures::StreamExt;

/// A user-created row query result holding references to table cells.
/// Used as the output type for table queries.
#[derive(Debug, Row, Remove, CommonKeys, Map)]
pub struct DebugRow<'a> {
    pub int: WriteCell<'a, i32>,
    pub float: WriteCell<'a, f32>,
    pub char: WriteCell<'a, char>,
}

impl<'a> DebugRow<'a> {
    fn integrate(self) {
        let DebugRow {
            mut int,
            mut float,
            mut char,
        } = self;

        *int += 1;
        *float += 1.0;
        *char = (((*int) as f32 * (*float)) as u8).into();
    }
}

pub struct IntegratorSystem;

impl IntegratorSystem {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
    pub async fn run<'a, T>(table: Arc<T>)
    where
        T: Table
            + BorrowColumn<i32>
            + BorrowColumn<f32>
            + BorrowColumn<char>
            + BorrowView<DebugRow<'a>>
            + Send
            + Sync
            + 'a,
    {
        let view = ReadView::new(table.deref()).await;
        let query = view.keys().then(|key| DebugRow::new(table.deref(), key));
        futures::pin_mut!(query);
        while let Some(debug_row) = query.next().await {
            debug_row.integrate();
        }
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn assemble<T>(table: Arc<T>)
where
    T: Table
        + Borrow<AtomicUsize>
        + BorrowColumn<Label>
        + BorrowColumn<bool>
        + BorrowColumn<i32>
        + BorrowColumn<f32>
        + BorrowColumn<f32>
        + BorrowColumn<char>
        + BorrowColumn<&'static str>
        + BorrowColumn<String>
        + Send
        + Sync,
{
    #[derive(Row, Insert, Remove)]
    struct TestRow<'a> {
        pub label: ReadCell<'a, Label>,
        pub bool: ReadCell<'a, bool>,
        pub int: ReadCell<'a, i32>,
        pub float: Option<ReadCell<'a, f32>>,
        pub char: ReadCell<'a, char>,
    }
    // Insert
    let keys = TestRow::insert_auto_multi(
        table.deref(),
        std::array::IntoIter::new([
            ("Test A".into(), false, 1, '7', Some(4.0)),
            ("Test B".into(), true, 2, '8', Some(5.0)),
            ("Test C".into(), false, 2, '8', None),
            ("Test D".into(), true, 3, '9', Some(6.0)),
        ]),
    )
    .await;

    // Remove
    TestRow::remove(table.deref(), keys[1]).await;

    let key = table.next_key();
    table.insert(key, Label::from("Test E")).await;
    table.insert(key, true).await;
    table.insert(key, 4.0).await;
    table.insert(key, "hello").await;
    table.insert(key, "world".to_string()).await;
}
