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
use std::collections::HashSet;
use std::collections::BTreeMap;

//#[derive(Debug)]
struct Args<'a> {
    conf: Option<&'a str>,
    output: Option<&'a str>,
    nonrecursive: bool,
    inputs: Vec<&'a str>
}

const RUCCOFILE_NAME: &'static str = "Ruccofile.toml";

const ABOUT: &'static str = "
Rucco, a docco derivative (documentation generator).

This tool will automatically generate a 'Ruccofile.toml' conf file if lacking.

Command line argument priority > Ruccofile priority > Base config priority.
(The base config is embedded in the rucco binary).
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

fn parse_conf_file(path: &str) -> toml::Table {
    BTreeMap::new()
}

fn parse_default_conf(mut resources: HashMap<Vec<u8>, Vec<u8>>) -> toml::Table {
    let file_as_bytes = resources.remove("Ruccofile.toml".as_bytes())
        .expect("could not find default conf failed!??");
    let file_as_string = String::from_utf8(file_as_bytes)
        .expect("default conf not utf8!??");
    toml::Parser::new(file_as_string.as_str()).parse()
        .expect("default conf parsing failed!??")
}

fn merge_confs(base: &toml::Table, custom: &toml::Table) -> toml::Table {
    let mut merged: toml::Table = BTreeMap::new();
    let keys: HashSet<&String> = base.keys().chain(custom.keys()).collect();
    for key in keys {
        let val = match (base.get(key), custom.get(key)) {
            (Some(&toml::Value::Table(ref basetable)),
             Some(&toml::Value::Table(ref customtable))) =>
                toml::Value::Table(merge_confs(&basetable, &customtable)),
            (_, Some(customval)) => customval.clone(),
            (Some(baseval),_) => baseval.clone(),
            (_,_) => panic!("wat!???")
        };
        merged.insert(key.clone(), val);
    };
    merged
}

struct Config<'a> {
    recursive: bool,
    entries: Vec<&'a str>,
    output_dir: &'a str,
    languages: &'a toml::Table
}

fn main() {
    env_logger::init().unwrap();

    let matches = cli().get_matches();
    let args = Args::new(&matches);
    let resources: HashMap<Vec<u8>, Vec<u8>> = embed!("resources");

    // conf
    let base_conf = parse_default_conf(resources);
    let custom_conf_path = if let Some(conf_path) = args.conf { conf_path } else { RUCCOFILE_NAME };
    let custom_conf = parse_conf_file(custom_conf_path);
    let conf = merge_confs(&base_conf, &custom_conf);

    let conf_input = conf.get("input").expect("malformed conf - no input")
        .as_table().expect("malformed conf - input is not a table");
    let conf_output = conf.get("output").expect("malformed conf - no output")
        .as_table().expect("malformed conf - output is not a table");

    // output
    let output_dir = if let Some(output) = args.output {
        output
    } else {
        conf_output.get("dir").expect("malformed conf - no output.dir")
            .as_str().expect("malformed conf - output.dir is not a string")
    };

    // nonrecursive
    let recursive = !args.nonrecursive ||
        conf_input
        .get("recursive").expect("malformed conf - no input.recursive")
        .as_bool().expect("malformed conf - input.recursive is not a boolean");

    // inputs
    let entries = if args.inputs.is_empty() {
        conf_input
            .get("entries").expect("malformed conf - no input.entries")
            .as_slice().expect("malformed conf - input.entries is not an array")
            .iter().map(|ref v| v.as_str().expect("malformed conf - one entry in input.entries is not a string"))
            .collect()
    } else {
        args.inputs
    };

    // languages
    let languages = conf.get("languages").expect("malformed conf - no languages")
        .as_table().expect("malformed conf - languages is not a table");

    let config = Config { recursive: recursive, entries: entries, output_dir: output_dir,
                          languages: &languages };

    // if ruccofile does not exist, dump final config in!

    // ...

    // and now recurse files and dump shit!
}
