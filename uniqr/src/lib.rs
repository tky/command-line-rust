use clap::Parser;
use std::fs::File;
use std::error::Error;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command[author, version, about]]
/// Rust version uniq
pub struct Config {
    /// Input file
    #[arg(
        value_name = "IN_FILE",
        default_value = "-"
    )]
    in_file: String,
    /// Output file
    #[arg(
        value_name = "OUT_FILE"
    )]
    out_file: Option<String>,
    /// Show counts
    #[arg(
        short,
        long,
    )]
    count: bool
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file)
        .map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        print!("{}", line);
        line.clear();
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
