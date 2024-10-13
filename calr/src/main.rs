use clap::Parser;
use anyhow::{anyhow, Result};
use chrono::{Days, Local, Months, NaiveDate};

#[derive(Parser, Debug)]
#[command(author, version, about)]
/// Rust version of `cal`
struct Args {
    /// Year (1-9999)
    #[arg(value_parser(clap::value_parser!(i32).range(1..=9999)))]
    year: Option<i32>,

    /// Month name or number (1-12)
    #[arg(short)]
    month: Option<String>,

    /// Show the whole current year
    #[arg(short('y'), long("year"), conflicts_with_all(["month", "year"]))]
    show_current_year: bool,
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let month = args.month.map(parse_month).transpose()?;
    let _today = Local::today();
    Ok(())
}

fn format_month(
    year: i32,
    month: u32,
    print_year: bool,
    today: NaiveDate,
) -> Vec<String> {
    unimplemented!()
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd(year + 1, 1, 1))
        .pred()
}

fn parse_month(month: String) -> Result<u32> {
    match month.parse::<u32>() {
        Ok(m) if m >= 1 && m <= 12 => Ok(m),
        Ok(m) => Err(anyhow!(r#"month "{m}" not in the range 1 through 12"#)),
        _ => {
            match MONTH_NAMES.iter().position(|m| m.to_lowercase().starts_with(&month)) {
                Some(m) => Ok(m as u32 + 1),
                _ => Err(anyhow!(r#"Invalid month "{month}""#)),
            }
        }
    }
}


// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month, parse_month};
    use chrono::NaiveDate;

    #[test]
    fn test_parse_month() {
        let res = parse_month("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "0" not in the range 1 through 12"#
        );

        let res = parse_month("13".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "13" not in the range 1 through 12"#
        );

        let res = parse_month("foo".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid month "foo""#);
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }
}
