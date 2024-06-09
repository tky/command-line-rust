use clap::Parser;
use std::fs::File;
use std::error::Error;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T,Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of head
pub struct Config {
    /// Input file(s)
    #[arg(
        default_value = "-",
        value_name = "FILE"
        )]
    files: Vec<String>,
    /// Number of lines [default: 10]
    #[arg(
        short('n'),
        long,
        default_value_t = 10,
        conflicts_with("bytes"),
        value_parser = clap::value_parser!(u64).range(1..),
        value_name = "LINES",
    )]
    lines: u64,
    /// Number of bytes
    #[arg(
        short('c'),
        long,
        conflicts_with("lines"),
        value_parser = clap::value_parser!(u64).range(1..),
        value_name = "BYTES"
    )]
    bytes: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

pub fn run(config: Config) -> MyResult<()> {
    let multiple_files = config.files.len() > 1;
    for (file, is_last_file) in config.files.iter().enumerate().map(|(i, w)| (w, i == config.files.len() - 1)) {
        match open(&file) {
            Err(err) => eprintln!("{}: {}", file, err),
            Ok(mut read) => {
                if multiple_files {
                    println!("==> {} <==", file);
                }
                let mut buffer = Vec::new();
                read.read_to_end(&mut buffer)?;

                if let Some(byte) = config.bytes {
                    let buffer = &buffer[..(byte as usize).min(buffer.len())];
                    let text = String::from_utf8_lossy(buffer);
                    print!("{}", text);
                } else {
                    print!("{}", String::from_utf8_lossy(&buffer));
                }
            }
        }
        if !is_last_file {
            println!("");
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
