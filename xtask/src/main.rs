mod build;
mod run;
mod download_limine;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let _ = match args.get(1).map(|s| s.as_str()) {
        Some("build") => build::build(None),
        Some("run") => run::run(args.last()),
        _ => {
            eprintln!("Usage: cargo xtask [build|run]");
            std::process::exit(1);
        }
    };
}




