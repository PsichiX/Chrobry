use chrobry_core::generate;
use clap::{App, Arg};
use std::fs::{read_to_string, write};

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
        .get_matches();
    let entry = matches.value_of("entry").unwrap();
    let output = matches.value_of("output").unwrap();
    let separator = match matches.value_of("separator") {
        Some(num) => num.parse::<usize>().unwrap(),
        None => 1,
    };
    let separator = "\n".repeat(separator);
    let content = read_to_string(entry)
        .unwrap_or_else(|error| panic!("Could not open entry file: {} | {:?}", entry, error));
    let content = match generate(&content, &separator, |_| Ok("".to_owned())) {
        Ok(content) => content,
        Err(error) => panic!("{}", error),
    };
    write(output, &content)
        .unwrap_or_else(|error| panic!("Could not write output file: {} | {:?}", output, error));
}
