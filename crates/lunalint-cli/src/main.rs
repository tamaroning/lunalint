use std::{fs::OpenOptions, io::Read, path::PathBuf, sync::Arc};

use clap::Parser;
use lunalint_core::{
    ariadne::{Color, Fmt},
    env_logger,
    location::SourceInfo,
    log, parse, pass, print_report, Context,
};

#[derive(Parser)]
pub struct Args {
    /// Sets a custom config file
    #[arg(value_name = "FILE")]
    input_file: PathBuf,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    log::debug!("input file: {:?}", args.input_file);

    let Ok(mut file) = OpenOptions::new()
        .read(true)
        .open(&args.input_file)
        .map_err(|e| error(format!("failed to open file: {}", e)))
    else {
        std::process::exit(1);
    };

    let mut code = String::new();
    let Ok(_) = file.read_to_string(&mut code).map_err(|e| {
        error(format!("failed to read file: {}", e));
    }) else {
        std::process::exit(1);
    };

    let src = Arc::new(SourceInfo::new(
        args.input_file.to_str().unwrap().to_string(),
        code,
    ));

    let Ok(ast) = parse(Arc::clone(&src)).map_err(|e| {
        print_report(&e);
    }) else {
        std::process::exit(1);
    };

    log::debug!("successfully parsed file");

    let mut ctx = Context::new(src);
    ctx.resolver_mut().go(&ast);

    let ctx = Arc::new(ctx);
    let mut pass_manager = pass::PassManager::with_all_passes(Arc::clone(&ctx));
    pass_manager.run(&ast);

    for report in ctx.reports().iter() {
        print_report(report);
    }

    if ctx.saw_error() {
        exit_with_error();
    }
}

fn error(msg: String) {
    eprintln!("lunalint: {} {msg}", "error:".fg(Color::Red));
}

fn exit_with_error() -> ! {
    error("exited with 1 due to previous errors".to_owned());
    std::process::exit(1);
}
