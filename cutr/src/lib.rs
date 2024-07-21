use anyhow::{anyhow, bail, Result};
use std::{error::Error, num::NonZeroUsize, ops::Range};
use clap::Parser;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use regex::Regex;
use std::io::{self, BufRead, BufReader};
use std::fs::File;

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

    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    let delim_bytes = config.delimiter.as_bytes();
    if delim_bytes.len() != 1 {
        return Err(format!(r#"--delim "{}" must be a single byte"#, config.delimiter).into())
    }
    let delimiter:u8 = *delim_bytes.first().unwrap();

    let extract = if let Some(fields) =
        config.extract.fields.map(parse_pos).transpose()?
    {
        Extract::Fields(fields)
    } else if let Some(bytes) =
        config.extract.bytes.map(parse_pos).transpose()?
    {
        Extract::Bytes(bytes)
    } else if let Some(chars) =
        config.extract.chars.map(parse_pos).transpose()?
    {
        Extract::Chars(chars)
    } else {
        unreachable!("Must have --fields, --bytes, or --chars");
    };

    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => match &extract {
                Extract::Fields(field_pos) => {
                    let mut reader = ReaderBuilder::new()
                        .delimiter(delimiter)
                        .has_headers(false)
                        .from_reader(file);
                    let mut writer = WriterBuilder::new()
                        .delimiter(delimiter)
                        .from_writer(io::stdout());
                    for record in reader.records() {
                        let record = record?;
                        let fields = extract_fields(&record, field_pos);
                        writer.write_record(&fields)?;
                    }
                }
                Extract::Bytes(byte_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_bytes(&line?, byte_pos));
                    }
                }
                Extract::Chars(char_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_chars(&line?, char_pos));
                    }
                }
            },
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

fn test_arg(arg: &Option<String>) -> MyResult<()> {
    match arg {
        Some(ref a) => {
            let numeric = a.parse::<u8>();
            match numeric {
                Ok(_) => Ok(()),
                Err(_) => {
                    return Err(From::from(format!(
                        "illegal list value: \"{}\"",
                        a
                    )))
                }
            }
        }
        None => Ok(()),
    }
}


fn parse_index(input: &str) -> Result<usize> {
    let value_error = || anyhow!("illegal list value: \"{}\"", input);
    if input.starts_with('+') {
        Err(value_error())
    } else {
        input.parse::<NonZeroUsize>()
            .map(|n| usize::from(n) -1)
            .map_err(|_| value_error())
    }
}

// cargo test unit_tests::test_parse_pos
fn parse_pos(range: String) -> Result<PositionList> {
    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap();
    range.split(',').map(|val| {
        match parse_index(val) {
            // 数字が一つだけの場合
            // cut -f 2,3 books.tsv
            Ok(n) => Ok(n..n + 1),
            // 数字が-で区切られている場合
            // cut -c 2-3 books.tsv
            Err(e) => {
                range_re.captures(val).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        bail!(
                            "First number in range ({}) \
                            must be lower than second number ({})",
                            n1 + 1,
                            n2 + 1
                        );
                    }
                    Ok(n1..n2 + 1)
                })
            }
        }
    })
    // Iterator<Item = Result<Range<usize>, anyhow::Error>>
    // から Result<Vec<Range<usize>>, anyhow::Error> に変換
    .collect::<Result<_, _>>()
    .map_err(From::from)
}

// cargo test unit_tests::test_extract_bytes
fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes = line.as_bytes();
    let selected: Vec<_> = byte_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| bytes.get(i)).copied())
        .collect();
    String::from_utf8_lossy(&selected).into_owned()
}

// cargo test unit_tests::test_extract_chars
fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let line: Vec<char> = line.chars().collect();
    let seelcted: Vec<_> = char_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| line.get(i)).copied())
        .collect();
    seelcted.iter().collect()
}

// cargo test unit_tests::test_extract_fields
fn extract_fields<'a>(
    record: &'a StringRecord,
    field_pos: &[Range<usize>],
) -> Vec<&'a str> {
    field_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| record.get(i)))
        .collect()
}


#[cfg(test)]
mod unit_tests {
    use super::{extract_bytes, extract_chars, extract_fields, parse_pos};
    use csv::StringRecord;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("".to_string()).is_err());

        // Zero is an error
        let res = parse_pos("0".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "0""#
        );

        let res = parse_pos("0-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "0""#
        );

        // A leading "+" is an error
        let res = parse_pos("+1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1""#,
        );

        let res = parse_pos("+1-2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1-2""#,
        );

        let res = parse_pos("1-+2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-+2""#,
        );

        // Any non-number is an error
        let res = parse_pos("a".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "a""#
        );

        let res = parse_pos("1,a".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "a""#
        );

        let res = parse_pos("1-a".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-a""#,
        );

        let res = parse_pos("a-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "a-1""#,
        );

        // Wonky ranges
        let res = parse_pos("-".to_string());
        assert!(res.is_err());

        let res = parse_pos(",".to_string());
        assert!(res.is_err());

        let res = parse_pos("1,".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-1".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-a".to_string());
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(
            extract_fields(&rec, &[0..1, 2..3]),
            &["Captain", "12345"]
        );
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(
            extract_chars("ábc", &[0..1, 1..2, 4..5]),
            "áb".to_string()
        );
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }
}
