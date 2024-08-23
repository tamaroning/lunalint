use std::{
    fs::OpenOptions,
    io::{BufWriter, Read},
    path::PathBuf,
    sync::Arc,
};

use lunalint_core::{diagnostics::write_report, location::SourceInfo, parse, pass, Context};

pub fn run_linter(path: &PathBuf) -> String {
    let mut file = OpenOptions::new().read(true).open(&path).unwrap();
    // buffer
    let mut out = BufWriter::new(Vec::new());

    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();

    let src = Arc::new(SourceInfo::new(path.to_str().unwrap().to_string(), code));

    let ast = match parse(Arc::clone(&src)) {
        Ok(ast) => ast,
        Err(e) => {
            write_report(&e, &mut out);
            return String::from_utf8(out.into_inner().unwrap()).unwrap();
        }
    };

    let mut ctx = Context::new(src);
    ctx.resolver_mut().go(&ast);

    let ctx = Arc::new(ctx);
    let mut pass_manager = pass::PassManager::with_all_passes(Arc::clone(&ctx));
    pass_manager.run(&ast);

    for report in ctx.reports().iter() {
        write_report(report, &mut out);
    }

    String::from_utf8(out.into_inner().unwrap()).unwrap()
}
