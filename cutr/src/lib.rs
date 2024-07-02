use std::{error::Error, ops::Range};
use clap::Parser;

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct ArgsExtract {
    /// Selected fields
    #[arg(short, long, value_name = "FIELDS")]
    fields: Option<String>,

    /// Selected bytes
    #[arg(short, long, value_name = "BYTES")]
    bytes: Option<String>,

    /// Selected chars
    #[arg(short, long, value_name = "CHARS")]
    chars: Option<String>,
}


#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of cut
pub struct Config {
    /// Input file(s)
    #[arg(default_value = "-")]
    files: Vec<String>,

    /// Field delimiter
    #[arg(short, long, value_name = "DELIMITER", default_value = "\t")]
    delimiter: String,

    #[command(flatten)]
    extract: ArgsExtract,
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    if config.delimiter.bytes().len() != 1 {
        return Err(From::from(format!(
            "--delim \"{}\" must be a single byte",
            config.delimiter
        )));
    };

    match config.extract.bytes {
        Some(ref bytes) => {
            let test = bytes.parse::<u8>();
            match test {
                Ok(_) => (),
                Err(_) => {
                    return Err(From::from(format!(
                        "illegal list value: \"{}\"",
                        bytes
                    )))
                }
            }
        }
        None => (),
    };

    match config.extract.chars {
        Some(ref chars) => {
            let test = chars.parse::<u8>();
            match test {
                Ok(_) => (),
                Err(_) => {
                    return Err(From::from(format!(
                        "illegal list value: \"{}\"",
                        chars
                    )))
                }
            }
        }
        None => (),
    };

    match config.extract.fields {
        Some(ref fields) => {
            let test = fields.parse::<u8>();
            match test {
                Ok(_) => (),
                Err(_) => {
                    return Err(From::from(format!(
                        "illegal list value: \"{}\"",
                        fields
                    )))
                }
            }
        }
        None => (),
    };

    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}
