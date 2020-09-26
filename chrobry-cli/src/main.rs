use chrobry_core::generate;
use clap::{App, Arg};
use std::{
    collections::HashMap,
    fs::{read_to_string, write},
};

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("entry")
                .short("e")
                .long("entry")
                .value_name("FILE")
                .help("Chrobry template entry file name")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Chrobry generated file name")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("separator")
                .short("s")
                .long("separator")
                .value_name("NUMBER")
                .help("Number of new lines used as separator")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("variable")
                .short("v")
                .long("var")
                .value_name("NAME=VALUE")
                .help("Key-value pair for variable passed into generator")
                .takes_value(true)
                .multiple(true)
                .required(false),
        )
        .arg(
            Arg::with_name("variable-file")
                .short("f")
                .long("file")
                .value_name("NAME=FILE")
                .help("Key-value pair for variable content got from the file passed into generator")
                .takes_value(true)
                .multiple(true)
                .required(false),
        )
        .get_matches();
    let entry = matches.value_of("entry").unwrap();
    let output = matches.value_of("output").unwrap();
    let separator = match matches.value_of("separator") {
        Some(num) => num.parse::<usize>().unwrap(),
        None => 1,
    };
    let separator = "\n".repeat(separator);
    let mut variables = if let Some(variables) = matches.values_of("variable") {
        variables
            .filter_map(|variable| {
                let parts = variable.split("=").collect::<Vec<_>>();
                if parts.len() == 2 {
                    Some((parts[0].to_owned(), parts[1].to_owned()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>()
    } else {
        HashMap::new()
    };
    if let Some(files) = matches.values_of("variable-file") {
        variables.extend(
            files
                .filter_map(|variable| {
                    let parts = variable.split("=").collect::<Vec<_>>();
                    if parts.len() == 2 {
                        let entry = parts[1].to_owned();
                        let content = read_to_string(&entry).unwrap_or_else(|error| {
                            panic!(
                                "Could not open variable content file: {} | {:?}",
                                entry, error
                            )
                        });
                        Some((parts[0].to_owned(), content))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>(),
        )
    }
    let content = read_to_string(entry)
        .unwrap_or_else(|error| panic!("Could not open entry file: {} | {:?}", entry, error));
    let content = match generate(&content, &separator, variables, |_| Ok("".to_owned())) {
        Ok(content) => content,
        Err(error) => panic!("{}", error),
    };
    write(output, &content)
        .unwrap_or_else(|error| panic!("Could not write output file: {} | {:?}", output, error));
}
