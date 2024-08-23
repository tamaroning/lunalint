use crate::{location::Location, pass::Pass};
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

#[derive(Clone)]
pub struct LintReport {
    name: String,
    kind: LintKind,
    level: LintLevel,
    loc: Location,
    msg: String,
    labels: Vec<LintLabel>,
}

impl LintReport {
    pub fn new(pass: &dyn Pass, loc: Location, msg: String) -> Self {
        Self {
            name: pass.name().to_string(),
            kind: pass.kind(),
            level: pass.level(),
            loc,
            msg,
            labels: Vec::new(),
        }
    }

    pub fn new_parse_error(loc: Location, msg: String) -> Self {
        LintReport {
            name: "parse-error".to_owned(),
            kind: LintKind::ParseError,
            level: LintLevel::Error,
            loc,
            msg,
            labels: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: LintLabel) -> Self {
        self.labels.push(label);
        self
    }

    pub fn kind(&self) -> LintKind {
        self.kind
    }

    pub fn level(&self) -> LintLevel {
        self.level
    }

    pub fn loc(&self) -> Location {
        self.loc.clone()
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn labels(&self) -> &[LintLabel] {
        &self.labels
    }
}

#[derive(Clone, Copy)]
pub enum LintKind {
    Diagnostics,
    SyntaxError,
    ParseError,
}

impl LintKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Diagnostics => "diagnostics",
            Self::SyntaxError => "syntax-errors",
            Self::ParseError => "parse-errors",
        }
    }
}

#[derive(Clone, Copy)]
pub enum LintLevel {
    Error,
    Warning,
}

#[derive(Clone)]
pub enum LintLabel {
    Label { msg: String, loc: Location },
}

impl LintLabel {
    pub fn new(loc: Location, msg: String) -> Self {
        Self::Label { msg, loc }
    }

    pub fn loc(&self) -> Location {
        match self {
            Self::Label { loc, .. } => loc.clone(),
        }
    }

    pub fn msg(&self) -> &str {
        match self {
            Self::Label { msg, .. } => msg,
        }
    }
}

/// Emit a lint report from lint passes, which is internally stored in the context.
pub(crate) fn emit_report(pass: &dyn Pass, report: LintReport) {
    pass.ctx().push_report(report)
}

/// Print a lint report to stderr. Do not use this from lint passes.
pub fn eprint_report(report: &LintReport) {
    let loc = report.loc();
    convert_report(report)
        .eprint((loc.src().path(), Source::from(loc.src().content())))
        .unwrap();
}

/// Write a lint report to the given writer.
pub fn write_report<W: std::io::Write>(report: &LintReport, w: W) {
    let loc = report.loc();
    convert_report(report)
        .write((loc.src().path(), Source::from(loc.src().content())), w)
        .unwrap();
}

/// Convert a lint report to an ariadne's report.
fn convert_report<'a>(report: &'a LintReport) -> Report<'a, (&'a str, std::ops::Range<usize>)>
where
{
    let LintReport {
        name,
        kind,
        level,
        loc,
        msg,
        labels,
    } = report;

    let level = match level {
        LintLevel::Error => ReportKind::Error,
        LintLevel::Warning => ReportKind::Warning,
    };
    let mut builder = Report::build(level, loc.src().path(), loc.start().bytes()).with_message(
        format!("{} {}", msg, format!("({})", name).fg(Color::BrightBlack)),
    );

    for label in labels {
        builder = builder
            .with_label(Label::new((loc.src().path(), loc.range())).with_message(label.msg()));
    }

    if matches!(report.kind(), LintKind::Diagnostics | LintKind::SyntaxError) {
        builder = builder.with_help(format!(
            "for further information visit https://luals.github.io/wiki/{}/#{}",
            kind.as_str(),
            name
        ));
    }

    builder.finish()
}
