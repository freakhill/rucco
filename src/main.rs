//! # Rucco
//! A Docco derivative in Rust (with multiline support)
//!
//! This is a simple program that will parse through source files
//! comments and source segments, as quick'n'dirt litterate files
//! and render them in a html+css+js soup.
//!
//! Comments must be markdown-formatted.
//!
//! This program is parametered through its command line interface
//! and a *ruccofile* (typically "Ruccofile.toml).
//!
//! Command line argument priority > Ruccofile priority > Base config priority.
//! (The base config is embedded in the rucco binary).
//!
//! Concerning the source files, multiline and singleline comments
//! can generally be supported.

#![feature(plugin)]
#![plugin(embed)]

/// the embed plugin allows us to embed files into our binary!
/// (base conf, css, js etc.)

#[macro_use] extern crate log; /// for logging...
extern crate env_logger; /// makes our logger configurable by environment variable (eg. RUST_LOG=debug)
extern crate toml; /// for configuration files
extern crate clap; /// "Command Line Argument Parsing" library
extern crate walkdir; /// to... walk dirs

use clap::{Arg, ArgMatches, App};
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::io;
use std::fs;
use std::env;
use walkdir::{WalkDir};

/// ## Static data

/// A *ruccofile* (toml-formated) is a configuration file for this program.
const RUCCOFILE_NAME: &'static str = "Ruccofile.toml";

const MAX_DEPTH: u8 = 8;

/// This will be used for the command line interface.
const ABOUT: &'static str = "
Rucco, a docco derivative (documentation generator).

This tool will automatically generate a 'Ruccofile.toml' conf file if lacking.

Command line argument priority > Ruccofile priority > Base config priority.
(The base config is embedded in the rucco binary).
";

/// ## Structures

/// This will hold the data retrieved through clap.
struct Args<'a> {
    conf: Option<&'a str>,
    output: Option<&'a str>,
    nonrecursive: bool,
    inputs: Vec<&'a str>
}

/// This will hold our final configuration (after merging clap data and ruccofile data).
struct Config<'a> {
    recursive: bool,
    entries: Vec<&'a str>,
    output_dir: &'a str,
    languages: &'a toml::Table
}

/// ## CLI

/// We segragate the generation of the CLI in its own function.
/// It is not too easy to add to much more processing here because
/// of lifetime concerns.
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

/// Given `cli().get-matches() -> ArgMatches`, we choose to create this simple
/// function.
impl<'a> Args<'a> {
    fn new(matches: &'a ArgMatches<'a>) -> Args<'a> {
        let inputs : Vec<&str> = matches.values_of("inputs")
            .map_or(vec![], &Iterator::collect);
        Args {
            conf: matches.value_of("config"),
            output: matches.value_of("output"),
            nonrecursive: matches.is_present("non-recursive"),
            inputs: inputs,
        }
    }
}

/// ## Conf files

/// This function parses a ruccofile whose path is given as parameter.
fn parse_conf_file(path: &str) -> Result<toml::Table, io::Error> {
    let mut conf_file = try![File::open(path)];
    let mut conf_string = String::new();
    try![conf_file.read_to_string(&mut conf_string)];
    Ok(toml::Parser::new(conf_string.as_str()).parse()
       .expect("failed to parse custom ruccofile"))
}

/// This function parses the base ruccofile embedded in our binary.
fn parse_default_conf(mut resources: HashMap<Vec<u8>, Vec<u8>>) -> toml::Table {
    let file_as_bytes = resources.remove("Ruccofile.toml".as_bytes())
        .expect("could not find default conf failed!??");
    let file_as_string = String::from_utf8(file_as_bytes)
        .expect("default conf not utf8!??");
    toml::Parser::new(file_as_string.as_str()).parse()
        .expect("default conf parsing failed!??")
}

/// And this is a simple recursive function to merge configurations!
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

/// Then with this function, if a ruccofile is not already present we can
/// dump our merged config in.
/// This function should be called only if a custom conf was not given through the cli.
fn ensure_ruccofile_exists(config: &Config) -> Result<(), io::Error> {
    let ruccofile_path = Path::new(RUCCOFILE_NAME);
    if !ruccofile_path.is_file() {
        info!("generating configuration file: {}", RUCCOFILE_NAME);
        let mut conf_input: toml::Table = BTreeMap::new();
        let mut conf_output: toml::Table = BTreeMap::new();
        let mut conf_languages: toml::Table = BTreeMap::new();
        let mut input: toml::Table = BTreeMap::new();
        let mut output: toml::Table = BTreeMap::new();

        input.insert("recursive".to_string(), toml::Value::Boolean(config.recursive));
        output.insert("dir".to_string(), toml::Value::String(config.output_dir.to_string()));
        input.insert("entries".to_string(), toml::Value::Array(
            config.entries.iter().map(|v| toml::Value::String(v.to_string())).collect()
        ));

        conf_input.insert("input".to_string(), toml::Value::Table(input));
        conf_output.insert("output".to_string(), toml::Value::Table(output));
        conf_languages.insert("languages".to_string(), toml::Value::Table(config.languages.clone()));

        let mut ruccofile = try![File::create(RUCCOFILE_NAME)];
        /// we do this that way only to make the final file more readable!
        try![ruccofile.write_all(toml::encode_str(&conf_input).as_bytes())];
        try![ruccofile.write_all("\n".as_bytes())];
        try![ruccofile.write_all(toml::encode_str(&conf_output).as_bytes())];
        try![ruccofile.write_all("\n".as_bytes())];
        try![ruccofile.write_all(toml::encode_str(&conf_languages).as_bytes())];
    }
    Ok(())
}

/// ## The function actually doing stuff

/// We transform quite early paths to absolute paths so unless specifically mentionned
/// we will be dealing with absolute paths!

fn ensure_dir(path: &PathBuf) -> io::Result<()> {
    if !path.is_dir() {
        try![fs::create_dir(path)];
    }
    Ok(())
}

fn process_file(config: &Config, source: &Path, target: &Path) {
    info!("from {} to {}", source.display(), target.display());
}

trait Absolute {
    fn to_absolute(&self, &PathBuf) -> PathBuf;
}

impl Absolute for Path {
    fn to_absolute(&self, pwd: &PathBuf) -> PathBuf {
        if !self.is_absolute() {
            pwd.join(self)
        } else {
            self.to_owned()
        }
    }
}

impl Absolute for PathBuf {
    fn to_absolute(&self, pwd: &PathBuf) -> PathBuf {
        if !self.is_absolute() {
            pwd.join(self)
        } else {
            self.clone()
        }
    }
}

/// ## The main function!

/// And now we put everything together.
fn main() {
    env_logger::init().unwrap();

    let matches = cli().get_matches();
    let args = Args::new(&matches);
    let resources: HashMap<Vec<u8>, Vec<u8>> = embed!("resources");

    // conf
    let base_conf = parse_default_conf(resources);
    let custom_conf_path = if let Some(conf_path) = args.conf { conf_path } else { RUCCOFILE_NAME };
    let custom_conf = parse_conf_file(custom_conf_path).unwrap_or_else(|e| {
        info!("no custom ruccofile: {}", e);
        BTreeMap::new()
    });
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
    /// using ! and || makes it hard to read, so ifs!
    let recursive = if args.nonrecursive {
        false
    } else {
        conf_input
            .get("recursive").expect("malformed conf - no input.recursive")
            .as_bool().expect("malformed conf - input.recursive is not a boolean")
    };

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

    // if a ruccofile was not given as parameter, ensure a local one exists (create if necessary).
    if let None = args.conf {
        if let Err(e) = ensure_ruccofile_exists(&config) {
            error!("failed to make sure ruccofile exists: {}", e);
        }
    }

    // and now processing!
    ensure_dir(&PathBuf::from(config.output_dir))
        .expect("failed to ensure that output directory exists.");
    let pwd = env::current_dir().expect("failed to get current dir!");
    let output_dir = fs::canonicalize(config.output_dir)
        .expect("failed to canonicalize output dir path.");

    // and now recurse files and dump shit!
    let entries = config.entries.iter()
        .filter_map(|p| fs::canonicalize(p).ok())
        .filter(|p| !p.starts_with(&output_dir))
        .filter(|p| p.starts_with(&pwd));
    if config.recursive {
        for entry in entries {
            for entry in WalkDir::new(entry)
                .follow_links(false)
                .max_depth(MAX_DEPTH as usize)
                .into_iter()
                .filter_map(|p| p.ok())
            {
                let relative = entry.path().strip_prefix(&pwd)
                    .expect("failed to generate a relative path.");
                if entry.path().is_dir() {
                    ensure_dir(&output_dir.join(&relative))
                        .expect("failed to create subdirectory in output dir.");
                } else {
                    let target = output_dir.join(&relative);
                    process_file(&config, &relative, &target);
                }
            }
        }
    } else {
        for file in entries.filter(|p| p.is_file()) {
            let relative = file.strip_prefix(&pwd)
                .expect("failed to generate a relative path.");
            let target = output_dir.join(&relative);
            process_file(&config, &relative, &target);
        }
    }
    info!("complete!");
}
