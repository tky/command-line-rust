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
    if names.len() > 0 {
        let file_name = entry.file_name().to_string_lossy();
        let re = names.iter().any(|re| re.is_match(&file_name));
        if !re {
            return false
        }
    }
    if entry_types.len() > 0 {
        if entry.file_type().is_dir() && !entry_types.contains(&EntryType::Dir) {
            return false
        }
        if entry.file_type().is_file() && !entry_types.contains(&EntryType::File) {
            return false
        }
        if entry.file_type().is_symlink() && !entry_types.contains(&EntryType::Link) {
            return false
        }
        
    }
    true
}
