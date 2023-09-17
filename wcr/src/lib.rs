use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
        .about("Rust wc")
        .author("Krithik")
        .version("0.1.0")
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help("Show byte count"),
        )
        .arg(
            Arg::with_name("chars")
                .short("m")
                .long("chars")
                .help("Show character count")
                .conflicts_with("bytes"),
        )
        .arg(
            Arg::with_name("lines")
                .short("l")
                .long("lines")
                .help("Show line count"),
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .help("Show word count"),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let lines = matches.is_present("lines");
    let words = matches.is_present("words");
    let bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    let default_state = !lines && !words && !bytes && !chars;

    let lines = default_state || lines;
    let words = default_state || words;
    let bytes = default_state || bytes;

    Ok(Config {
        files,
        lines,
        words,
        bytes,
        chars,
    })
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut buf = String::new();

    loop {
        let bytes = file.read_line(&mut buf)?;
        if bytes == 0 {
            break;
        }
        num_lines += 1;
        num_words += buf.split_whitespace().count();
        num_bytes += buf.len();
        num_chars += buf.chars().count();

        buf.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

pub fn format_field(number: usize, display: bool) -> String {
    if !display {
        return String::new();
    }

    return format!("{:>8}", number);
}

pub fn format_info(file_info: &FileInfo, config: &Config, filename: &str) -> String {
    let mut output = [
        (file_info.num_lines, &config.lines),
        (file_info.num_words, &config.words),
        (file_info.num_bytes, &config.bytes),
        (file_info.num_chars, &config.chars),
    ]
    .map(|(num, disp)| format_field(num, *disp))
    .join("");
    if filename != "-" {
        output = format!("{} {}", output, filename)
    }
    output
}

pub fn run(config: Config) -> MyResult<()> {
    // dbg!(config);
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    for filename in &config.files {
        match open(&filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                let file_info = count(file)?;
                // dbg!(&file_info);
                num_lines += file_info.num_lines;
                num_words += file_info.num_words;
                num_bytes += file_info.num_bytes;
                num_chars += file_info.num_chars;
                // l w m/c name(if not stdin)
                println!("{}", format_info(&file_info, &config, filename));
            }
        }
    }

    if config.files.len() > 1 {
        println!(
            "{}",
            format_info(
                &FileInfo {
                    num_lines,
                    num_words,
                    num_bytes,
                    num_chars
                },
                &config,
                "total"
            )
        )
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{count, format_field, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_format_field() {
        assert_eq!(format_field(1, false), "");
        assert_eq!(format_field(3, true), "       3");
        assert_eq!(format_field(10, true), "      10");
    }
}
