use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{
        self,
        format::{Format, FormatEvent, FormatFields, Writer},
        FmtContext,
    },
    registry::LookupSpan,
};

pub struct PrettyFormatter {
    inner: Format,
}

impl PrettyFormatter {
    pub fn new() -> Self {
        Self {
            inner: fmt::format()
                .with_ansi(true)
                .with_target(false)
                .with_file(false)
                .with_line_number(false)
                .with_level(true)
                .with_source_location(false),
        }
    }
}

impl<S, N> FormatEvent<S, N> for PrettyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let depth = ctx.event_scope().map(|scope| scope.count()).unwrap_or(0);

        writeln!(writer, "-----")?;
        for _ in 0..depth / 4 {
            write!(writer, "  ")?;
        }

        if depth > 0 {
            write!(writer, "└─")?;
        } else {
            write!(writer, "  ")?;
        }

        self.inner.format_event(ctx, writer, event)
    }
}
