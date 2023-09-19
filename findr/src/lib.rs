// use crate::EntryType;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Eq)]
enum EntryType {
    Dir,
    File,
    Link,
}

impl From<&str> for EntryType {
    fn from(value: &str) -> Self {
        match value {
            "d" => EntryType::Dir,
            "f" => EntryType::File,
            "l" => EntryType::Link,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .about("Rust find")
        .author("Krithik")
        .version("0.1.0")
        .arg(
            Arg::with_name("names")
                .short("n")
                .long("name")
                .value_name("NAME")
                .help("Name")
                .multiple(true),
        )
        .arg(
            Arg::with_name("types")
                .short("t")
                .long("type")
                .help("Entry Type")
                .value_name("TYPE")
                .possible_values(&["f", "d", "l"])
                .multiple(true),
        )
        .arg(
            Arg::with_name("paths")
                .value_name("PATH")
                .help("Search paths")
                .multiple(true)
                .default_value("."),
        )
        .get_matches();

    let paths = matches.values_of_lossy("paths").unwrap();
    let names = matches
        .values_of_lossy("names")
        .map(|vals| {
            vals.into_iter()
                .map(|name| {
                    let x = Regex::new(&name).map_err(|_| format!("Invalid --name \"{}\"", name));
                    x
                })
                .collect::<Result<Vec<Regex>, String>>()
        })
        .transpose()?
        .unwrap_or_default();
    let entry_types = matches
        .values_of_lossy("types")
        .map(|t| t.iter().map(|e| EntryType::from(e.as_str())).collect())
        .unwrap_or_default();

    Ok(Config {
        paths,
        names,
        entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // dbg!(&config);
    let find_dir = config.entry_types.len() == 0 || config.entry_types.contains(&EntryType::Dir);
    let find_file = config.entry_types.len() == 0 || config.entry_types.contains(&EntryType::File);
    let find_link = config.entry_types.len() == 0 || config.entry_types.contains(&EntryType::Link);
    let should_match_name = config.names.len() > 0;

    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(entry) => {
                    if entry.file_type().is_dir() && !find_dir {
                        continue;
                    }
                    if entry.file_type().is_symlink() && !find_link {
                        continue;
                    }
                    if entry.file_type().is_file() && !find_file {
                        continue;
                    }

                    if should_match_name {
                        let has_match = config
                            .names
                            .iter()
                            .any(|r| r.is_match(&entry.file_name().to_string_lossy()));
                        if !has_match {
                            continue;
                        }
                    }

                    println!("{}", entry.path().display())
                }
            }
        }
    }
    Ok(())
}
