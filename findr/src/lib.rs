use clap::{ArgAction, Parser, ValueEnum};
use regex::Regex;
use walkdir::WalkDir;
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
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(entry) => {
                    if should_display(&entry, &config.names, &config.entry_types) {
                        println!("{}", entry.path().display())
                    }
                },
            }
        }
    }
    Ok(())
}


fn should_display(entry: &walkdir::DirEntry, names: &Vec<Regex>, entry_types: &Vec<EntryType>) -> bool {
    (names.is_empty() || names.iter().any(|re| re.is_match(&entry.file_name().to_string_lossy())))
        &&
    (entry_types.is_empty() || entry_types.iter().any(|entry_type| {
        match entry_type {
            EntryType::Link => entry.file_type().is_symlink(),
            EntryType::File => entry.file_type().is_file(),
            EntryType::Dir => entry.file_type().is_dir(),
        }
    }))
}
