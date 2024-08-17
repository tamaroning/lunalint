pub(crate) mod context;
pub(crate) mod diagnostics;
pub(crate) mod location;
mod pass;

use std::{fs::OpenOptions, io::Read, path::PathBuf, sync::Arc};

use clap::Parser;

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
        .write(true)
        .create(true)
        .open(&args.input_file)
        .map_err(|e| log::error!("failed to open file: {}", e))
    else {
        std::process::exit(1);
    };

    let mut src = String::new();
    let Ok(_) = file.read_to_string(&mut src).map_err(|e| {
        log::error!("failed to read file: {}", e);
    }) else {
        std::process::exit(1);
    };

    let Ok(ast) = full_moon::parse(&src).map_err(|e| log::error!("failed to parse file: {}", e))
    else {
        std::process::exit(1);
    };

    log::debug!("successfully parsed file");

    let ctx = context::Context::new(args.input_file, src);
    let ctx = Arc::new(ctx);

    let mut pass_manager = pass::PassManager::new();
    pass_manager.add_pass(Box::new(pass::count_down_loop::CountDownLoop::new(
        Arc::clone(&ctx),
    )));
    pass_manager.add_pass(Box::new(pass::global_in_nil_env::GlobalInNilEnv::new(
        Arc::clone(&ctx),
    )));
    pass_manager.add_pass(Box::new(pass::unicode_name::UnicodeName::new(Arc::clone(
        &ctx,
    ))));
    pass_manager.run(&ast);
}
