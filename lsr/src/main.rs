mod owner;

use clap::Parser;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::fs::{ self, metadata};
use std::os::unix::fs::MetadataExt;
use tabular::{Row, Table};
use users::{get_user_by_uid, get_group_by_gid};
use chrono::{DateTime, Utc};
use owner::Owner;


#[derive(Parser, Debug)]
#[command(author, version, about)]
/// Rust version of `ls`
struct Args {
    /// Files and/or directories [default: .]
    #[arg(
        value_name = "PATH",
        default_value = ".")
    ]
    paths: Vec<String>,
    /// Long listing
    #[arg(short, long)]
    long: bool,
    /// Show all files
    #[arg(short('a'), long("all"))]
    show_hidden: bool,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let paths = find_files(&args.paths, args.show_hidden)?;
    if args.long {
        println!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}

fn find_files(
    paths: &[String],
    show_hidden: bool,
    ) -> Result<Vec<PathBuf>> {
    let mut entries = vec![];
    paths.iter().for_each(|path| {
        match metadata(path) {
            Ok(metadata) => {
                if metadata.is_file() {
                    entries.push(PathBuf::from(path));
                } else if metadata.is_dir() {
                    fs::read_dir(path).unwrap().for_each(|entry| {
                        let path = entry.unwrap().path();
                        if show_hidden || !path.file_name().unwrap().to_str().unwrap().starts_with(".") {
                            entries.push(path);
                        }
                    });
                } else {
                    println!("{}: not a file or directory", path);
                }
            },
            _ => eprintln!("{}: No such file or directory", path),
        }
    });
    Ok(entries)
}

fn format_mode(mode: u32) -> String {
    format!("{}{}{}",
            mk_triple(mode, Owner::User)
            , mk_triple(mode, Owner::Group)
            , mk_triple(mode, Owner::Other))
}

fn format_output(paths: &[PathBuf]) -> Result<String> {
    let fmt = "{:<}{:<} {:>} {:<} {:<} {:>} {:<} {:<}";
    let mut table = Table::new(fmt);
    for path in paths {
        let meta = metadata(path)?;
        let modified_time: DateTime<Utc> = meta.modified().unwrap().into();
        table.add_row(
            Row::new()
            .with_cell(if meta.is_dir()  {"d"} else {"-"}) // 1 "d"または"-"
            .with_cell(format_mode(meta.mode())) // 2 パーミッション
            .with_cell(meta.nlink()) // 3 リンク数
            .with_cell(get_user_by_uid(meta.uid()).unwrap().name().to_str().unwrap()) // 4 ユーザー名
            .with_cell(get_group_by_gid(meta.gid()).unwrap().name().to_str().unwrap()) // 5 グループ名
            .with_cell(meta.len()) // 6 サイズ
            .with_cell(modified_time.format("%Y-%m-%d %H:%M:%S")) // 7 更新日時
            .with_cell(path.display()) // 8 パス
        );
    }
    Ok(format!("{}", table))
}

fn mk_triple(mode: u32, owner: Owner) -> String {
    let [read, write, execute] = owner.masks();
        format!(
            "{}{}{}",
            if mode & read == 0 { "-" } else { "r"},
            if mode & write == 0 { "-" } else { "w" },
            if mode &execute == 0 { "-" } else { "x" }
        )
}

// --------------------------------------------------
#[cfg(test)]
mod test {
    use super::{find_files, format_mode, format_output, mk_triple, Owner};
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Find all non-hidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
            "tests/inputs/bustle.txt",
            "tests/inputs/dir",
            "tests/inputs/empty.txt",
            "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
            "tests/inputs/bustle.txt".to_string(),
            "tests/inputs/dir".to_string(),
            ],
            false,
            );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // Find all entries in a directory including hidden
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
            "tests/inputs/.hidden",
            "tests/inputs/bustle.txt",
            "tests/inputs/dir",
            "tests/inputs/empty.txt",
            "tests/inputs/fox.txt",
            ]
        );
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
        ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(!parts.is_empty() && parts.len() <= 10);

        let perms = parts.first().unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> =
            out.split('\n').filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> =
            out.split('\n').filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
            );

        let dir_line = lines.remove(0);
        long_match(dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }

    #[test]
    fn test_mk_triple() {
        assert_eq!(mk_triple(0o751, Owner::User), "rwx");
        assert_eq!(mk_triple(0o751, Owner::Group), "r-x");
        assert_eq!(mk_triple(0o751, Owner::Other), "--x");
        assert_eq!(mk_triple(0o600, Owner::Other), "---");
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }
}
