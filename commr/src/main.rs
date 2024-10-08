use std::io;
use std::io::{BufRead, BufReader};
use std::fs::File;
use anyhow::{anyhow, bail, Result};
use std::cmp::Ordering;
use clap::{ArgAction, Parser};
use crate::Column::*;


enum Column<'a> {
    Col1(&'a str),
    Col2(&'a str),
    Col3(&'a str),
}

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
    let file1 = &args.file1;
    let file2 = &args.file2;

    if file1 == "-" && file2 == "-" {
        bail!("Both input files cannot be STDIN (\"-\")");
    }

    let print = |col: Column| {
        let mut columns = vec![];
        match col {
            Col1(val) => {
                if args.show_col1 {
                    columns.push(val);
                }
            }
            Col2(val) => {
                if args.show_col2 {
                    if args.show_col1 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
            Col3(val) => {
                if args.show_col3 {
                    if args.show_col1 {
                        columns.push("");
                    }
                    if args.show_col2 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
        }

        if !columns.is_empty() {
            println!("{}", columns.join(&args.delimiter));
        }
    };

    match (open(file1), open(file2))  {
        (Ok(read1), Ok(read2)) => {
            let mut lines1 = read1.lines().filter_map(|line| line.ok());
            let mut lines2 = read2.lines().filter_map(|line| line.ok());
            let mut line1 = lines1.next();
            let mut line2 = lines2.next();

            while line1.is_some() || line2.is_some() {
                match (&line1, &line2) {
                    (Some(l1), None) => {
                        print(Col1(l1));
                        line1 = lines1.next();
                    },
                    (None, Some(l2)) => {
                        print(Col2(l2));
                        line2 = lines2.next();
                    },
                    (Some(l1), Some(l2)) => {
                        let l1 = if args.insensitive { l1.to_lowercase() } else { l1.to_string() };
                        let l2 = if args.insensitive { l2.to_lowercase() } else { l2.to_string() };
                        match l1.cmp(&l2) {
                            Ordering::Equal => {
                                print(Col3(&l1));
                                line1 = lines1.next();
                                line2 = lines2.next();
                            },
                            Ordering::Less => {
                                print(Col1(&l1));
                                line1 = lines1.next();
                            },
                            Ordering::Greater => {
                                print(Col2(&l2));
                                line2 = lines2.next();
                            },
                        }
                    }
                    _ => break,
                }
            }
        },
        (Err(e), _) => bail!("{}: {}", file1, e),
        (_, Err(e)) => bail!("{}: {}", file2, e),
    }
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
                File::open(filename).map_err(|e| anyhow!("{}: {}", filename, e))?))),
    }
}
