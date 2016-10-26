//! # Rucco
//! A Docco derivative in Rust (with multiline support)

#![feature(plugin)]
#![plugin(embed)]

extern crate rustc_serialize;
extern crate docopt;

extern crate clap;
use clap::{Arg, App, SubCommand};
use std::collections::HashMap;

const USAGE: &'static str = "

Author: Jojo <gall.johan@linecorp.com>

Usage:
  rucco [--conf=<f>] [--output=<dir>] [--recursive] <files_and_dirs>...
  rucco (-h | --help)
  rucco --version

Options:
  -h --help          Show this screen.
  --conf=<f>
  --output=<dir>     .
  --recursive=<boolean> Explore directories recursively (default is true).
";

const ABOUT: &'static str = "
Rucco, a docco derivative (documentation generator).

This tool will automatically generate a 'Ruccofile.toml' conf file if lacking.
";

fn parse_args() -> Args {
    let matches = App::new("rucco")
        .version("1.0")
        .author("jojo <gall.johan@linecorp.com>")
        .about(ABOUT)
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("RUCCOFILE")
             .help("Conf file to use (default is \"Ruccofile.toml\")")
             .takes_value(true))
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .value_name("TARGETDIR")
             .help("Output directory (default is \"docs\")")
             .takes_value(true))
        .arg(Arg::with_name("recursive")
             .short("r")
             .long("recursive")
             .value_name("FILE")
             .help("Conf file to use (default is \"Ruccofile.toml\")")
             .takes_value(true))
        .arg(Arg::with_name("INPUT")
             .help("Sets the input file to use")
             .multiple(true)
             .index(1))
        .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config = matches.value_of("config").unwrap_or("default.conf");
    println!("Value for config: {}", config);

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    println!("Using input file: {}", matches.value_of("INPUT").unwrap());

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match matches.occurrences_of("v") {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }

    // more program logic goes here...
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
    args
}

fn main() {
    // 1. load params
    // 2. read config
    // 3. create output folder
    // 4. go recursively in input folders and for each file
    // 4.1 split in comments and code sections
    // 4.2 use syntect to colorize code (html)
    // 4.3 stick in between comment section
    // 4.4 render the whole thing with regex
    let args = parse_args();
    let files: HashMap<Vec<u8>, Vec<u8>> = embed!("resources");
    for (name, content) in files {
        println!("{}: \"{}\"", String::from_utf8(name).unwrap(), String::from_utf8(content).unwrap().trim());
    }
}
