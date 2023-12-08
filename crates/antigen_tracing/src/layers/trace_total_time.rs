use async_std::{channel::Sender, sync::Arc};
use deebs::{BorrowSingleton, ReadSingleton, Table};
use std::{
    fmt::{Debug, Display},
    ops::Deref,
    time::{Duration, Instant},
};
use tracing::{span, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

use crate::{TraceEvent, TracePath, TraceRoot};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TotalStart(Instant);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TotalDuration(Duration);

impl Deref for TotalDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for TotalDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.0))
    }
}

#[derive(Debug)]
pub struct TraceTotalTime {
    sender: Sender<TraceEvent>,
}

impl TraceTotalTime {
    pub fn new<T>(table: Arc<T>) -> Self
    where
        T: Table + BorrowSingleton<TraceRoot>,
    {
        let trace_root = async_std::task::block_on(ReadSingleton::new(table.deref()));
        let sender = trace_root.sender();
        TraceTotalTime { sender }
    }
}

impl<S> Layer<S> for TraceTotalTime
where
    S: Subscriber + for<'a> LookupSpan<'a> + Debug,
{
    fn new_span(&self, _attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();

        let time_start = TotalStart(Instant::now());
        extensions.insert(time_start);
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();
        let total_duration = {
            let mut extensions = span.extensions_mut();
            let TotalStart(start) = extensions.get_mut::<TotalStart>().unwrap();
            TotalDuration(Instant::now() - *start)
        };
        let trace_path = TracePath::from_span(span);
        self.sender
            .try_send(TraceEvent::total_duration(trace_path, total_duration))
            .unwrap();
    }
}
