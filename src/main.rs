//! # Rucco
//! A Docco derivative in Rust (with multiline support)

#![feature(plugin)]
#![plugin(embed)]

extern crate clap;
use clap::{Arg, App, SubCommand};
use std::collections::HashMap;

const ABOUT: &'static str = "
Rucco, a docco derivative (documentation generator).

This tool will automatically generate a 'Ruccofile.toml' conf file if lacking.
";

fn parse_args() {
    let matches = App::new("rucco")
        .version("1.0")
        .author("jojo <gall.johan@linecorp.com>")
        .about(ABOUT)
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("ruccofile")
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
             .help("Explore directories recursively (default is true)")
             .takes_value(true))
        .arg(Arg::with_name("inputs")
             .help("Files and directories to parse for documentation")
             .multiple(true)
             .value_name("FILES_AND_DIRS")
             .index(1))
        .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let inputs = matches.values_of("inputs").unwrap();
    for s in inputs {
        println!("Value for config: {}", s);
    }

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    //println!("Using input file: {}", matches.value_of("INPUT").unwrap());

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match matches.occurrences_of("v") {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }
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
    //for (name, content) in files {
    //    println!("{}: \"{}\"", String::from_utf8(name).unwrap(), String::from_utf8(content).unwrap().trim());
    //}
}
