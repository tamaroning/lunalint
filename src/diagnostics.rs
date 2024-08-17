use std::ops::Range;

use crate::{context::Context, location::Location, pass::Pass};
use ariadne::{Cache, Color, Fmt, Report, ReportBuilder, ReportKind, Source, Span};

pub(crate) fn emit2<'a>(ctx: &'a Context, report: Report<(&'a str, Range<usize>)>) {
    let fname = ctx.file_name();
    let src = ctx.src();
    report.print((fname, Source::from(src))).unwrap();
}

pub(crate) fn emit<'a, S>(pass: &'a dyn Pass, report: ReportBuilder<'a, S>)
where
    S: Span,
    &'a str: Into<<<S as Span>::SourceId as ToOwned>::Owned>,
    (&'a str, Source<&'a str>): Cache<<S as Span>::SourceId>,
{
    let fname = pass.ctx().file_name();
    let src = pass.ctx().src();
    report
        .with_help(format!(
            "for further information visit https://luals.github.io/wiki/diagnostics/#{}",
            pass.name()
        ))
        .finish()
        .print((fname, Source::from(src)))
        .unwrap();
}

pub(crate) fn diag<'a, S>(
    pass: &'a dyn Pass,
    kind: ReportKind<'a>,
    loc: Location,
    msg: String,
) -> ReportBuilder<'a, S>
where
    S: Span,
    &'a str: Into<<<S as Span>::SourceId as ToOwned>::Owned>,
{
    Report::build(kind, pass.ctx().file_name(), loc.start())
        .with_code(999)
        .with_message(format!(
            "{} {}",
            msg,
            format!("({})", pass.name()).fg(Color::BrightBlack)
        ))
}
