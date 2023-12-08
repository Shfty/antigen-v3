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
pub struct SelfStart(Instant);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelfDuration(Duration);

impl Deref for SelfDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for SelfDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.0))
    }
}

#[derive(Debug)]
pub struct TraceSelfTime {
    sender: Sender<TraceEvent>,
}

impl TraceSelfTime {
    pub fn new<T>(table: Arc<T>) -> Self
    where
        T: Table + BorrowSingleton<TraceRoot>,
    {
        let trace_root = async_std::task::block_on(ReadSingleton::new(table.deref()));
        let sender = trace_root.sender();
        TraceSelfTime { sender }
    }
}

impl<S> Layer<S> for TraceSelfTime
where
    S: Subscriber + for<'a> LookupSpan<'a> + Debug,
{
    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();

        let time_start = SelfStart(Instant::now());
        if extensions.get_mut::<SelfStart>().is_some() {
            extensions.replace(time_start);
        } else {
            extensions.insert(time_start);
        }
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();
        if let Some(SelfStart(start)) = extensions.get_mut::<SelfStart>() {
            let start = *start;
            let delta = Instant::now() - start;
            if let Some(SelfDuration(duration)) = extensions.get_mut::<SelfDuration>() {
                let duration = *duration;
                extensions.replace(SelfDuration(duration + delta));
            } else {
                extensions.insert(SelfDuration(delta));
            }
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();
        let self_duration = span.extensions().get::<SelfDuration>().copied().unwrap();
        let trace_path = TracePath::from_span(span);
        self.sender
            .try_send(TraceEvent::self_duration(trace_path, self_duration))
            .unwrap();
    }
}
