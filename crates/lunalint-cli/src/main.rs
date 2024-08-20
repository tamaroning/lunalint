use std::{fs::OpenOptions, io::Read, path::PathBuf, sync::Arc};

use clap::Parser;
use lunalint_core::{
    ariadne::{Color, Fmt},
    Context,
    env_logger, full_moon, log, pass,
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

    let mut src = String::new();
    let Ok(_) = file.read_to_string(&mut src).map_err(|e| {
        error(format!("failed to read file: {}", e));
    }) else {
        std::process::exit(1);
    };

    let Ok(ast) = full_moon::parse(&src).map_err(|e| {
        // pretty print parse errors
        error(format!("failed to parse file: {}", e))
    }) else {
        std::process::exit(1);
    };

    log::debug!("successfully parsed file");

    let mut ctx = Context::new(args.input_file, src);
    ctx.resolver_mut().go(&ast);

    let ctx = Arc::new(ctx);
    let mut pass_manager = pass::PassManager::new();
    pass_manager.add_pass(Box::new(pass::CountDownLoop::new(Arc::clone(&ctx))));
    pass_manager.add_pass(Box::new(pass::GlobalInNilEnv::new(Arc::clone(&ctx))));
    pass_manager.add_pass(Box::new(pass::UnicodeName::new(Arc::clone(&ctx))));
    pass_manager.add_pass(Box::new(pass::UndefinedGlobal::new(Arc::clone(&ctx))));
    pass_manager.add_pass(Box::new(pass::LowercaseGlobal::new(Arc::clone(&ctx))));
    pass_manager.run(&ast);

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
