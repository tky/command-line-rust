use std::{error::Error, ops::Range};
use clap::Parser;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};

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

    for field in [
        &config.extract.fields,
        &config.extract.bytes,
        &config.extract.chars,
    ] {
        match test_arg(field) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
    }
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
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


fn parse_pos(range: String) -> MyResult<PositionList> {
    unimplemented!();
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    unimplemented!();
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    unimplemented!();
}

fn extract_fields<'a>(
    record: &'a StringRecord,
    field_pos: &[Range<usize>],
) -> Vec<&'a str> {
    unimplemented!();
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
