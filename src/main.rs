//! # Rucco
//! A Docco derivative in Rust (with multiline support)

#![feature(plugin)]
#![plugin(embed)]

#[macro_use] extern crate log;
extern crate toml;
extern crate env_logger;
extern crate clap;
use clap::{Arg, ArgMatches, App};
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

fn cli<'a, 'b>() -> App<'a, 'b> {
    App::new("rucco")
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
        .arg(Arg::with_name("non-recursive")
             .long("non-recursive")
             .value_name("FILE")
             .help("Do non explore directories recursively (recurse by default)")
             .takes_value(true))
        .arg(Arg::with_name("inputs")
             .help("Files and directories to parse for documentation")
             .multiple(true)
             .value_name("FILES_AND_DIRS")
             .index(1))
}

impl<'a> Args<'a> {
    fn new(matches: &'a ArgMatches<'a>) -> Args<'a> {

        let inputs : Vec<&str> = matches.values_of("inputs").map_or(vec![],
                                                                    &Iterator::collect);

        Args {
            conf: matches.value_of("config"),
            output: matches.value_of("output"),
            nonrecursive: matches.is_present("non-recursive"),
            inputs: inputs,
        }
    }
}

fn parse_conf_file(path: &str) -> u32 {
    1
}

fn parse_default_conf(resources: HashMap<Vec<u8>, Vec<u8>>) -> u32 {
    1
}

fn main() {
    env_logger::init().unwrap();
    // 1. load params
    // 2. read config
    // 3. create output folder
    // 4. go recursively in input folders and for each file
    // 4.1 split in comments and code sections
    // 4.2 use syntect to colorize code (html)
    // 4.3 stick in between comment section
    // 4.4 render the whole thing with regex
    let matches = cli().get_matches();
    let args = Args::new(&matches);
    let resources: HashMap<Vec<u8>, Vec<u8>> = embed!("resources");

    let default_conf = parse_default_conf(resources);
    let conf = if let Some(conf_path) = args.conf {
        parse_conf_file(conf_path) // merge defautl conf in
    } else {
       default_conf
    };

    let output = if let Some(output) = args.output {
        output
    } else {
        // conf output
        "TODO"
    };

    // nonrecursive
    let recursive = non(args.nonrecursive) or true; // or conf.recursive

    // inputs
    let inputs = if args.inputs.is_empty() {
        vec![] // conf inputs
    } else {
        args.inputs
    }
    //for (name, content) in files {
    //    println!("{}: \"{}\"", String::from_utf8(name).unwrap(), String::from_utf8(content).unwrap().trim());
    //}

    // if ruccofile does not exist, dump conf in!

    // ...

    // and now recurse files and dump shit!
}
