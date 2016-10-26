//! # Rucco
//! A Docco derivative in Rust (with multiline support)

#![feature(plugin)]
#![plugin(embed)]

extern crate clap;
use clap::{Arg, App, SubCommand};
use std::collections::HashMap;

//#[derive(Debug)]
struct Args<'a> {
    conf: Option<&'a str>,
    output: Option<&'a str>,
    notrecursive: bool,
    inputs: Vec<&'a str>
}

const ABOUT: &'static str = "
Rucco, a docco derivative (documentation generator).

This tool will automatically generate a 'Ruccofile.toml' conf file if lacking.
";

fn parse_args<'a, 'b>(app : &'b mut App<'a, 'b>) -> Args<'a> {
    let matches = app
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
        .arg(Arg::with_name("notrecursive")
             .long("not-recursive")
             .value_name("FILE")
             .help("Do not explore directories recursively (recurse by default)")
             .takes_value(true))
        .arg(Arg::with_name("inputs")
             .help("Files and directories to parse for documentation")
             .multiple(true)
             .value_name("FILES_AND_DIRS")
             .index(1))
        .get_matches();

    let inputs : Vec<&str> = matches.values_of("inputs").unwrap().collect();

    Args {
        conf: matches.value_of("config"),
        output: matches.value_of("output"),
        notrecursive: matches.is_present("not-recursive"),
        inputs: inputs,
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
    let app = App::new("rucco");
    let args = parse_args(&mut app);
    let files: HashMap<Vec<u8>, Vec<u8>> = embed!("resources");
    //for (name, content) in files {
    //    println!("{}: \"{}\"", String::from_utf8(name).unwrap(), String::from_utf8(content).unwrap().trim());
    //}
}
