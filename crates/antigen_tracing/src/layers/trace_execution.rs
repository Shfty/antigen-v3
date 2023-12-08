use async_std::sync::Arc;
use deebs::{BorrowSingleton, Table, WriteSingleton};
use std::{fmt::Debug, ops::Deref};
use tracing::{span, Event, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

use crate::{Records, TraceEvent, TracePath, TraceRoot};

#[derive(Debug)]
pub struct TraceExecution {
    sender: async_std::channel::Sender<TraceEvent>,
}

impl TraceExecution {
    pub fn new<T>(table: Arc<T>) -> Self
    where
        T: Table + BorrowSingleton<TraceRoot>,
    {
        let trace_root = async_std::task::block_on(WriteSingleton::<TraceRoot>::new(table.deref()));
        let sender = trace_root.sender();
        TraceExecution { sender }
    }
}

impl<S> Layer<S> for TraceExecution
where
    S: Subscriber + for<'a> LookupSpan<'a> + Debug,
{
    fn new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let span = ctx.span(id).unwrap();
        let metadata = span.metadata();
        let trace_path = TracePath::from_span(span);
        let mut records = Records::default();
        attrs.record(&mut records);
        self.sender
            .try_send(TraceEvent::new_span(trace_path, metadata, records))
            .unwrap();
    }

    fn on_record(&self, id: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let trace_path = TracePath::from_span(span);
        let mut records = Records::default();
        values.record(&mut records);
        self.sender
            .try_send(TraceEvent::on_record(trace_path, records))
            .unwrap();
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let span = if event.is_contextual() {
            if let Some(span) = ctx.lookup_current() {
                span
            } else {
                return;
            }
        } else {
            if let Some(id) = event.parent() {
                if let Some(span) = ctx.span(id) {
                    span
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        let trace_path = TracePath::from_span(span);
        let mut tree_event = crate::Event::new(event.metadata());
        event.record(tree_event.records_mut());
        self.sender
            .try_send(TraceEvent::on_event(trace_path, tree_event))
            .unwrap();
    }
}
