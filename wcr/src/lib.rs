use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about)]
// Rust version of wc
pub struct Config {
    /// Input file(s)
    #[arg(
        value_name = "FILE",
        default_value = "-",
        )]
        files: Vec<String>,
        /// Show byte count
        #[arg(short('c'), long)]
        bytes: bool,
        /// Show character count
        #[arg(short('m'), long, conflicts_with("bytes"))]
        chars: bool,
        /// Show line count
        #[arg(short('l'), long)]
        lines: bool,
        /// Show word count
        #[arg(short('w'), long)]
        words: bool,
}

impl Config {
    fn finalize(&mut self) {
        if [self.lines, self.bytes, self.words, self.chars].iter().all(|v| !v) {
            self.lines = true;
            self.words = true;
            self.bytes = true;
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

impl FileInfo {
    fn add(&mut self, other: &FileInfo) {
        self.num_lines += other.num_lines;
        self.num_words += other.num_words;
        self.num_bytes += other.num_bytes;
        self.num_chars += other.num_chars;
    }
}

impl FileInfo {
    fn print(&self, filename: &str, config: &Config) {
        if config.lines {
            print!("{}", format!("{:>8}", self.num_lines));
        }
        if config.words {
            print!("{}", format!("{:>8}", self.num_words));
        }
        if config.bytes {
            print!("{}", format!("{:>8}", self.num_bytes));
        }
        if config.chars {
            print!("{}", format!("{:>8}" , self.num_chars));
        }
        if filename != "-" {
            println!(" {}", filename);
        } else {
            println!();
        }
    }
}

pub fn get_args() -> MyResult<Config> {
    let mut config = Config::parse();
    config.finalize();
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total_file_info = FileInfo {
        num_lines: 0,
        num_words: 0,
        num_bytes: 0,
        num_chars: 0,
    };
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(read) => {
                let info = count(read)?;
                total_file_info.add(&info);
                info.print(filename, &config);
            }
        }
    }
    if config.files.len() > 1 {
        total_file_info.print("total", &config);
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut buf = String::new();

    while {
        let read_bytes = file.read_line(&mut buf)?;
        num_bytes += read_bytes;
        read_bytes != 0
    } {
        num_words += buf.split_whitespace().count();
        num_chars += buf.chars().count();
        num_lines += 1;
        buf.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_bytes: 48,
            num_chars: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}

