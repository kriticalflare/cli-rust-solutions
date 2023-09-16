use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse::<usize>() {
        Ok(res) if res > 0 => Ok(res),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    // 3 is an OK integer
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("Krithik - kriticalflare")
        .about("Rust head")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input File(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .help("Number of lines")
                .value_name("LINES")
                .default_value("10")
                .takes_value(true)
                .hide_default_value(false),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .value_name("BYTES")
                .help("Number of bytes")
                .conflicts_with("lines")
                .takes_value(true),
        )
        .get_matches();

    // let lines = match parse_positive_int(matches.value_of("lines").unwrap()) {
    //     Ok(line_count) => line_count,
    //     Err(err) => {
    //         return Err(From::from(format!(
    //             "illegal line count -- {}",
    //             err.to_string()
    //         )))
    //     }
    // };

    // let bytes = match matches.value_of("bytes") {
    //     Some(val) => match parse_positive_int(val) {
    //         Ok(byte) => Some(byte),
    //         Err(err) => {
    //             return Err(From::from(format!(
    //                 "illegal byte count -- {}",
    //                 err.to_string()
    //             )))
    //         }
    //     },
    //     None => None,
    // };

    let lines = matches
        .value_of("lines")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?;

    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: lines.unwrap(),
        bytes,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // dbg!(&config);
    let print_header = config.files.len() > 1;
    for (file_num, filename) in config.files.iter().enumerate() {
        match open(&filename) {
            Ok(mut file) => {
                // if multiple files print header
                // if bytes, then read x bytes from file
                // else read z lines from file
                // print the value read
                if print_header {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        &filename
                    );
                }

                if let Some(bytes) = config.bytes {
                    let mut buf = vec![0; bytes];
                    let mut handle = file.take(bytes as u64);
                    let bytes_read = handle.read(&mut buf)?;
                    print!("{}", String::from_utf8_lossy(&buf[0..bytes_read]));
                } else {
                    let mut buf = String::new();
                    for _ in 0..config.lines {
                        buf.clear();

                        let bytes = file.read_line(&mut buf)?;
                        if bytes == 0 {
                            break;
                        }
                        print!("{}", buf);
                    }
                }
            }
            Err(err) => eprintln!("{}: {}", filename, err),
        }
    }
    Ok(())
}
