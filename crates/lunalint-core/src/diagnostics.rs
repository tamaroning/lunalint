use crate::{location::Location, pass::Pass};
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

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

    pub fn with_label(mut self, label: LintLabel) -> Self {
        self.labels.push(label);
        self
    }
}

pub enum LintKind {
    Diagnostics,
    SyntaxError,
}

impl LintKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Diagnostics => "diagnostics",
            Self::SyntaxError => "syntax-errors",
        }
    }
}

pub enum LintLevel {
    Error,
    Warning,
}

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

/// Emit a lint report. This function is used by lint passes.
pub fn emit_report(pass: &dyn Pass, report: LintReport) {
    pass.ctx().push_report(report)
}

/// Print a lint report to stderr. Do not use this function in lint passes.
pub fn print_report(report: &LintReport) {
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
    let mut builder = Report::build(level, loc.src().path(), loc.start()).with_message(format!(
        "{} {}",
        msg,
        format!("({})", name).fg(Color::BrightBlack)
    ));

    for label in labels {
        builder = builder
            .with_label(Label::new((loc.src().path(), loc.range())).with_message(label.msg()));
    }

    builder
        .with_help(format!(
            "for further information visit https://luals.github.io/wiki/{}/#{}",
            kind.as_str(),
            name
        ))
        .finish()
        .eprint((loc.src().path(), Source::from(loc.src().content())))
        .unwrap();
}
