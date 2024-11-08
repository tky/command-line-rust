use clap::Parser;
use anyhow::{anyhow, Result};


#[derive(Parser, Debug)]
#[command(author, version, about)]
/// Rust version of `ls`
struct Args {
    /// Files and/or directories [default: .]
    #[arg(
        value_name = "PATH",
        default_value = ".")
    ]
    paths: Vec<String>,
    /// Long listing
    #[arg(short, long)]
    long: bool,
    /// Show all files
    #[arg(short('a'), long("all"))]
    show_hidden: bool,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    println!("{:?}", args);
    Ok(())
}
