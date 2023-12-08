/// [`log::Log`] implementor that gates a [`tracing_log::LogTracer`] behind an [`env_logger::filter::Filter`]
pub struct EnvLogTracer {
    log_tracer: tracing_log::LogTracer,
    filter: env_logger::filter::Filter,
}

impl EnvLogTracer {
    pub fn new() -> Self {
        use env_logger::filter::Builder;
        let mut builder = Builder::new();

        // Parse a directives string from an environment variable
        if let Ok(ref filter) = std::env::var("RUST_LOG") {
            builder.parse(filter);
        }

        EnvLogTracer {
            log_tracer: tracing_log::LogTracer::new(),
            filter: builder.build(),
        }
    }
}

impl log::Log for EnvLogTracer {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if self.filter.matches(record) {
            self.log_tracer.log(record)
        }
    }

    fn flush(&self) {}
}
