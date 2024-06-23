use clap::Parser;
use std::fs::File;
use std::error::Error;
use std::io::{self, BufRead, BufReader, Write};

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
    let out_file = &config.out_file;
    match out_file {
        None => 
            func(config, |line| {
              print!("{}", line);
            }),
        Some(filename) => {
            let mut out = File::create(&filename)?;
            func(config, |line| {
              out.write_all(line.as_bytes()).unwrap();
            })
        }
    }
}

fn func<F>(config: Config, mut f: F) -> MyResult<()> 
where
    F: FnMut(&str),
{
    let mut file = open(&config.in_file)
        .map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut prev = String::new();
    let mut count = 0;

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if count == 0 {
            prev = line.clone();
            count = 1;
        } else {
            if line.trim_end() == prev.trim_end() {
                count += 1;
            } else {
                let output = format!("{}{}", if config.count { format!("{:4} ", count) } else { "".to_string() }, prev);
                f(&output);
                prev = line.clone();
                count = 1;
            }
        }
        line.clear();
    }
    if count > 0 {
        let output = format!("{}{}", if config.count { format!("{:4} ", count) } else { "".to_string() }, prev);
        f(&output);
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
