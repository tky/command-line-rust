use clap::Parser;
use regex::RegexBuilder;
use std::path::PathBuf;
use std::fs;
use anyhow::{anyhow, bail, Result};
use walkdir::WalkDir;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
struct Fortune {
    source: String,
    text: String,
}

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
    let files = find_files(&args.sources)?;
    println!("{:?}", files);
    Ok(())
}

fn find_files(paths: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = vec![];

    for path in paths {
        match fs::metadata(path) {
            Err(e) => bail!("{}", e),
            Ok(_) => {files.extend(
                WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.into_path())
            );
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    unimplemented!();
}

fn read_fortunes(paths: &[PathBuf]) -> Result<Vec<Fortune>> {
    let mut fortunes = vec![];

    for path in paths {
        match fs::File::open(path) {
            Err(e) => bail!("{}", e),
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut vec = vec![];
                while  {
                    let read_bytes = reader.read_until(b'%', &mut vec)?;
                    read_bytes != 0
                } {
                    let text = String::from_utf8_lossy(&vec[..vec.len()-1]).trim().to_string();
                    if text.is_empty() || text == "%" {
                        continue;
                    }
                    let fortune = Fortune {
                        source: path.to_string_lossy().to_string(),
                        text,
                    };
                    fortunes.push(fortune);
                    vec.clear();
                }
            }
        }
    }
    Ok(fortunes)
}

#[cfg(test)]
mod tests {
    use super::{find_files, pick_fortune, read_fortunes, Fortune};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // Parses all the fortunes without a filter
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // Filters for matching text
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
