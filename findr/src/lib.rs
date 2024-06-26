use clap::{ArgAction, Parser, ValueEnum};
use regex::Regex;
use walkdir::{WalkDir, DirEntry};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;


#[derive(Debug, Eq, PartialEq, Clone, ValueEnum)]
enum EntryType {
    #[clap(name = "d")]
    Dir,
    #[clap(name = "f")]
    File,
    #[clap(name = "l")]   
    Link,
}

#[derive(Debug, Parser)]
#[command[author, version, about]]
/// Rust version find
pub struct Config {
    /// Search path(s)
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,

    /// Names
    #[arg(
        short('n'),
        long("name"),
        value_name = "NAME",
        value_parser(Regex::new),
        action(ArgAction::Append),
        num_args(0..)
    )]
    names: Vec<Regex>,

    /// Entry types
    #[arg(
        short('t'),
        long("type"),
        value_name = "TYPE",
        value_parser(clap::value_parser!(EntryType)),
        action(ArgAction::Append),
        num_args(0..)
    )]
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    for path in config.paths {
        let name_filter = |entry: &DirEntry| -> bool {
            config.names.is_empty() || config.names.iter().any(|re| re.is_match(&entry.file_name().to_string_lossy()))
        };

        let type_filter = |entry: &DirEntry| -> bool {
            config.entry_types.is_empty() || config.entry_types.iter().any(|entry_type| {
                match entry_type {
                    EntryType::Link => entry.file_type().is_symlink(),
                    EntryType::File => entry.file_type().is_file(),
                    EntryType::Dir => entry.file_type().is_dir(),
                }
            })
        };

        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| match e {
                Err(e)=> {
                    eprintln!("{}", e);
                    None
                }
                Ok(entry) => Some(entry),
            }).filter(type_filter)
        .filter(name_filter)
        .map(|entry| entry.path().display().to_string())
        .for_each(|entry| println!("{}", entry));
    }
    Ok(())
}
