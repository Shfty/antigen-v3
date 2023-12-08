#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u64)]
pub enum Method {
    Block,
    Spawn,
    Serial,
    Concurrent,
    Parallel,
}

/// Block on a future, instrumenting it with the current tracing span
#[macro_export]
macro_rules! block {
    ($fut:expr) => {
        let span = tracing::info_span!("block", method = $crate::Method::Block as u64);
        async_std::task::block(($fut).instrument(span))
    };
}

/// Spawn a future into a background task that may outlive the calling code, instrumenting it with the current tracing span
#[macro_export]
macro_rules! spawn {
    ($fut:expr) => {{
        let span = tracing::info_span!("spawn", method = $crate::Method::Spawn as u64);
        span.follows_from(tracing::span::Span::current());
        async_std::task::spawn(($fut).instrument(span))
    }};
}

/// Await a set of futures in sequence, instrumenting each with the current tracing span
#[macro_export]
macro_rules ! serial {
    ($($fut:expr),* $(,)?) => {
        {
            let span = tracing::info_span!("serial", method = $crate::Method::Serial as u64);
            $(
                ($fut).instrument(span.clone()).await;
            )*
        }
    };
}

/// Await a set of futures concurrently, instrumenting each with the current tracing span
#[macro_export]
macro_rules ! concurrent {
    ($($fut:expr),* $(,)?) => {
        {
            let span = tracing::info_span!("concurrent", method = $crate::Method::Concurrent as u64);
            futures::join!($(
                ($fut).instrument(span.clone()),
            )*);
        }
    };
}

/// Spawn a set of futures into parallel tasks, instrumenting each with the current tracing span
#[macro_export]
macro_rules ! parallel {
    ($($fut:expr),* $(,)?) => {
        {
            let span = tracing::info_span!("parallel", method = $crate::Method::Parallel as u64);
            futures::join!($(
                async_std::task::spawn(($fut).instrument(span.clone())),
            )*);
        }
    };
}
