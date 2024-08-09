use anyhow::Result;
use clap::{ArgAction, Parser};

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of comm
struct Args {
    /// Input file 1
    #[arg()]
    file1: String,

    /// Input file 2
    #[arg()]
    file2: String,

    /// Supress printing of column 1
    #[arg(short('1'), action(ArgAction::SetFalse))]
    show_col1: bool,

    /// Supress printing of column 2
    #[arg(short('2'), action(ArgAction::SetFalse))]
    show_col2: bool,

    /// Supress printing of column 3
    #[arg(short('3'), action(ArgAction::SetFalse))]
    show_col3: bool,

    /// Case-insensitive comparison of lines
    #[arg(short)]
    insensitive: bool,

    /// Output delimiter
    #[arg(short, long("output-delimiter"), default_value = "\t")]
    delimiter:  String,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    println!("{:?}", args);
    Ok(())
}
