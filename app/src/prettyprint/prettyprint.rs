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
    inner: Format, // o formatador padrão que já lida com cores
}

impl PrettyFormatter {
    pub fn new() -> Self {
        Self {
            inner: fmt::format()
                .with_ansi(true) // enables ANSI colors
                .with_target(false) // disables target (module path)
                .with_file(false) // disables file name
                .with_line_number(false) // disables line number
                .with_level(true) // enables log level
                .with_source_location(false), // disables source location
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

        self.inner.format_event(ctx, writer, event) // delega com coloração
    }
}
