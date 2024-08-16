use std::ops::Range;

use crate::context::Context;
use ariadne::{Report, Source};

pub(crate) fn emit<'a>(ctx: &'a Context, report: Report<(&'a str, Range<usize>)>) {
    let fname = ctx.file_name();
    let src = ctx.src();
    report.print((fname, Source::from(src))).unwrap();
}
