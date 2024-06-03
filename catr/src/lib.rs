use std::error::Error;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of cat
pub struct Config {
    #[arg(required(true))]
    files: Vec<String>,
    /// Number lines
    #[arg(short('n'), long, conflicts_with = "number_nonblank")]
    number: bool,
    /// Number non-blank lines
    #[arg(short('b'), long)]
    number_nonblank: bool,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(read) => {
                let mut line_number = 0;
                read.lines().for_each(|line| {
                let line = line.unwrap();
                let is_blank = line.trim().is_empty();

                if is_blank && config.number_nonblank {
                    println!("");
                } else {
                    line_number += 1;
                    if config.number || config.number_nonblank {
                        println!("{:>6}\t{}" , line_number, line);
                    } else {
                        println!("{}", line);
                    }
                }
            })},
        }
    }
    Ok(())
}


fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
