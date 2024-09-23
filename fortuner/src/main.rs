use clap::Parser;
use regex::{Regex, RegexBuilder};
use anyhow::{anyhow, Result};


#[derive(Parser, Debug)]
#[command(author, version, about)]
/// Rust fortune
struct Args {
    /// Input files or directories
    #[arg(required = true)]
    sources: Vec<String>,
    /// Pattern
    #[arg(short('m'), long)]
    pattern: Option<String>,
    /// Case-insensitive pattern matching
    #[arg(short, long)]
    insensitive: bool,
    /// Random seed
    #[arg(short, long, value_parser(clap::value_parser!(u64)))]
    seed: Option<u64>,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    println!("{:?}", args);
    let pattern = args.pattern
        .map(|p| RegexBuilder::new(&p)
            .case_insensitive(args.insensitive)
            .build()
            .map_err(|_| anyhow!(r#"Invalid --pattern "{p}""#))
        ).transpose()?;
    Ok(())
}
