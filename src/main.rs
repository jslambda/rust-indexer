use std::env;
use std::path::PathBuf;
use std::process;

use rust_indexer::{build_index, write_index_to};

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let program_name = args
        .next()
        .unwrap_or_else(|| String::from("rust-indexer"));

    let root_arg = args.next();
    if matches!(root_arg.as_deref(), Some("--help") | Some("-h")) {
        print_usage(&program_name);
        return Ok(());
    }

    let root = match root_arg {
        Some(path) => PathBuf::from(path),
        None => env::current_dir()?,
    };

    let entries = build_index(&root)?;
    write_index_to(&entries, std::io::stdout())?;

    Ok(())
}

fn print_usage(program_name: &str) {
    println!(
        "Usage: {program_name} [project_root]\n\nBuild a JSON search index from Rust sources in the `src/` directory. If project_root is omitted, the current working directory is used."
    );
}
