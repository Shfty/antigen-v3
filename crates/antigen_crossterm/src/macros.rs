/// Async adapter for [`crossterm::queue`].
#[macro_export]
macro_rules ! queue_async {
    ($stdout:expr, $($commands:expr),* $(,)?) => {
        async {
            let mut buf = vec![];
            crossterm::queue!(
                &mut buf,
                $($commands,)*
            )?;
            async_std::io::prelude::WriteExt::write_all($stdout, &buf).await?;

            (Ok(()) as Result<(), Box<dyn std::error::Error>>)
        }
    }
}
