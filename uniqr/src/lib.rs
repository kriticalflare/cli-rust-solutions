use clap::{App, Arg};
use core::num;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .about("Rust uniq")
        .author("Krithik")
        .version("0.1.0")
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Show counts"),
        )
        .arg(
            Arg::with_name("in_file")
                .value_name("IN_FILE")
                .help("Input file")
                .default_value("-"),
        )
        .arg(
            Arg::with_name("out_file")
                .value_name("OUT_FILE")
                .help("Output file"),
        )
        .get_matches();

    let in_file = String::from(matches.value_of_lossy("in_file").unwrap());
    let out_file = matches.value_of("out_file").map(|s| s.to_string());
    let count = matches.is_present("count");

    Ok(Config {
        in_file,
        out_file,
        count,
    })
}

pub fn output(
    print_count: bool,
    out_file: &mut Option<File>,
    num_lines: usize,
    prev_line: String,
) -> MyResult<()> {
    match out_file {
        Some(ref mut w) if print_count == true => write!(w, "{:>4} {}", num_lines, prev_line)?,
        Some(ref mut w) if print_count == false => write!(w, "{}", prev_line)?,
        _ => {
            if print_count {
                print!("{:>4} {}", num_lines, prev_line);
            } else {
                print!("{}", prev_line);
            }
        }
    }

    Ok(())
}

pub fn run(config: Config) -> MyResult<()> {
    let mut num_lines = 0;
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", &config.in_file, e))?;

    let mut out_file = config.out_file.map(File::create).transpose()?;

    let mut line = String::new();
    let mut prev_line = String::new();
    let mut first_line = true;
    loop {
        let bytes_read = file.read_line(&mut line)?;

        if bytes_read == 0 {
            if prev_line.len() > 0 {
                output(config.count, &mut out_file, num_lines, prev_line)?;
            }
            break;
        }

        let line_without_newline = if let Some(striped_line) = line.strip_suffix("\n") {
            String::from(striped_line)
        } else {
            line.clone()
        };

        let prev_without_newline = if let Some(striped_line) = prev_line.strip_suffix("\n") {
            String::from(striped_line)
        } else {
            prev_line.clone()
        };

        if first_line {
            first_line = !first_line;
            num_lines = 1;
            prev_line = line.clone();
            line.clear();
            continue;
        }

        if prev_without_newline == line_without_newline {
            num_lines += 1;
            line.clear();
            continue;
        } else {
            output(config.count, &mut out_file, num_lines, prev_line)?;

            prev_line = line.clone();
            num_lines = 1;
            line.clear();
        }
    }

    Ok(())
}
