// # Rucco
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

#[macro_use] extern crate rust_embed; /// for embedding css, html etc.
//#[macro_use] extern crate serde_derive; /// for the config
#[macro_use] extern crate log; /// for logging...
extern crate env_logger; /// makes our logger configurable by environment variable (eg. RUST_LOG=debug)
extern crate toml; /// for configuration files
extern crate clap; /// "Command Line Argument Parsing" library
extern crate walkdir;
extern crate rayon; /// for parallelism
extern crate tar;
extern crate rucco_lib;

use clap::{Arg, ArgMatches, App};
use std::collections::HashSet;
use std::ops::DerefMut;
use std::fs::File;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::io;
use std::fs;
use std::env;
use std::cell::RefCell;
use walkdir::{WalkDir};
use tar::Archive;
use rayon::prelude::*;

use rucco_lib::{Languages, render};

// ## Static data

#[derive(RustEmbed)]
#[folder = "resources/"]
struct Resources;

/// A *ruccofile* (toml-formated) is a configuration file for this program.
const RUCCOFILE_NAME: &'static str = "Ruccofile.toml";

/// Quic'n'dirty!
const MAX_DEPTH: u8 = 8;

/// (folders to create and files to process), if you use rucco for more than
/// 256 of those you have a problem...
const ESTIMATED_MAX_ACTIONS: usize = 256;

/// This will be used for the command line interface.
const ABOUT: &'static str = "
Rucco, a docco derivative (documentation generator).

This tool will automatically generate a 'Ruccofile.toml' conf file if lacking.

Command line argument priority > Ruccofile priority > Base config priority.
(The base config is embedded in the rucco binary).
";

// ## Structures

/// This will hold the data retrieved through clap.
struct Args<'a> {
    conf: Option<&'a str>,
    output: Option<&'a str>,
    nonrecursive: bool,
    inputs: Vec<&'a str>
}

struct Config<'a> {
    recursive: bool,
    entries: Vec<&'a str>,
    output_dir: &'a str,
    languages: &'a toml::value::Table
}

// #[derive(Deserialize)]
// struct ConfigInput {
//     recursive: Option<bool>,
//     entries: Option<Vec<String>>,
// }

// #[derive(Deserialize)]
// struct ConfigOutput {
//     dir: Option<String>,
// }

// #[derive(Deserialize)]
// struct ConfigLanguage {


// #[derive(Deserialize)]
// struct PartialConfig {
//     input: Option<ConfigInput>,
//     output: Option<ConfigOutput>,
//     languages: Option<HashMap<String,ConfigLanguage>>,
// }

// ## CLI

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

// ## Conf files

/// This function parses a ruccofile whose path is given as parameter.
fn parse_conf_file(path: &str) -> Result<toml::value::Table, io::Error> {
    let mut conf_file = File::open(path)?;
    let mut conf_string = String::new();
    conf_file.read_to_string(&mut conf_string)?;
    if let Ok(toml::Value::Table(t)) = conf_string.parse::<toml::Value>() {
        Ok(t)
    } else {
        panic!("failed to parse config Ruccofile!");
    }
}

/// This function parses the base ruccofile embedded in our binary.
fn parse_embedded_conf() -> toml::value::Table {
    let file_as_bytes = Resources::get("Ruccofile.toml")
        .expect("could not find embedded default conf!");
    let file_as_string = String::from_utf8(file_as_bytes.to_vec())
        .expect("embedded conf not in ut8 format!");
    if let Ok(toml::Value::Table(t)) = file_as_string.parse::<toml::Value>() {
        t
    } else {
        panic!("failed to parse config embedded default conf!");
    }
}

/// And this is a simple recursive function to merge configurations!
fn merge_tables(base: &toml::value::Table, custom: &toml::value::Table) -> toml::value::Table {
    let mut merged: toml::value::Table = toml::map::Map::new();
    let keys: HashSet<&String> = base.keys().chain(custom.keys()).collect();
    for key in keys {
        let val = match (base.get(key), custom.get(key)) {
            (Some(&toml::Value::Table(ref basetable)),
             Some(&toml::Value::Table(ref customtable))) =>
                toml::Value::Table(merge_tables(&basetable, &customtable)),
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
        let mut conf_input: toml::value::Table = toml::map::Map::new();
        let mut conf_output: toml::value::Table = toml::map::Map::new();
        let mut conf_languages: toml::value::Table = toml::map::Map::new();
        let mut input: toml::value::Table = toml::map::Map::new();
        let mut output: toml::value::Table = toml::map::Map::new();

        input.insert("recursive".to_string(), toml::Value::Boolean(config.recursive));
        output.insert("dir".to_string(), toml::Value::String(config.output_dir.to_string()));
        input.insert("entries".to_string(), toml::Value::Array(
            config.entries.iter().map(|v| toml::Value::String(v.to_string())).collect()
        ));

        conf_input.insert("input".to_string(), toml::Value::Table(input));
        conf_output.insert("output".to_string(), toml::Value::Table(output));
        conf_languages.insert("languages".to_string(), toml::Value::Table(config.languages.clone()));

        let mut ruccofile = File::create(RUCCOFILE_NAME)?;
        /// we do this that way only to make the final file more readable!
        ruccofile.write_all(toml::to_string(&conf_input).unwrap().as_bytes())?;
        ruccofile.write_all("\n".as_bytes())?;
        ruccofile.write_all(toml::to_string(&conf_output).unwrap().as_bytes())?;
        ruccofile.write_all("\n".as_bytes())?;
        ruccofile.write_all(toml::to_string(&conf_languages).unwrap().as_bytes())?;
    }
    Ok(())
}

// ## The function actually doing stuff

fn ensure_dir(path: &PathBuf) -> io::Result<()> {
    if !path.is_dir() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn untar_resources(output_dir: &Path,
                   pack_name: &str) -> io::Result<()> {
    let tar_bytes = Resources::get(pack_name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "could not find resource tar file"))?;
    let mut tar = Archive::new(&tar_bytes as &[u8]);
    try![tar.unpack(output_dir)];
    Ok(())
}

thread_local! {
    static LANG: RefCell<Option<Languages>> = RefCell::new(None);
}

fn process_file(config: &Config, source: &Path, target: &Path) -> io::Result<()> {
    LANG.with(|l| {
        let needs_init = l.borrow().is_none();
        if needs_init {
            debug!("thread local languages struct init.");
            *l.borrow_mut() = Some(Languages::new(config.languages.clone()));
        }
        if let &mut Some(ref mut languages) = l.borrow_mut().deref_mut() {
            // source path is relative to current dir, so it's depth gives us
            // how many times the path to css. "../../ depth times /style.css"
            let mut css_path = String::new();
            for _ in source.components().skip(1) {
                css_path.push_str("../");
            }
            css_path.push_str("style.css");
            if let Some(extension) = source.extension().and_then(&OsStr::to_str) {
                let mut source_text = String::new();
                File::open(source)?.read_to_string(&mut source_text)?;
                if let Some(ref rendered) = render(languages, extension, source_text.as_str(), source, css_path.as_str()) {
                    File::create(target)?.write_all(rendered.as_bytes())?;
                    info!("rendered {} to {}", source.display(), target.display());
                } else {
                    warn!("failed to render {}!", source.display());
                }
            } else {
                debug!("skipping {}", source.display());
            }
        }
        Ok(())
    })
}

fn htmlize(mut p: PathBuf) -> PathBuf {
    let new_f = if let Some(f) = p.file_name() {
        Some([f.to_str().expect("invalid path"), ".html"].concat())
    } else {
        None
    };
    if let Some(f) = new_f {
        p.set_file_name(OsStr::new(&f));
    };
    p
}

// ## The main function!

/// And now we put everything together.
fn main() {
    env_logger::init();

    let matches = cli().get_matches();
    let args = Args::new(&matches);

    // conf
    debug!("# CONF");
    let base_conf = parse_embedded_conf();
    let custom_conf_path = if let Some(conf_path) = args.conf { conf_path } else { RUCCOFILE_NAME };
    let custom_conf = parse_conf_file(custom_conf_path).unwrap_or_else(|e| {
        info!("no custom ruccofile: {}", e);
        toml::map::Map::new()
    });
    let conf = merge_tables(&base_conf, &custom_conf);

    let conf_input = conf.get("input").expect("malformed conf - no input")
        .as_table().expect("malformed conf - input is not a table");
    let conf_output = conf.get("output").expect("malformed conf - no output")
        .as_table().expect("malformed conf - output is not a table");

    // output
    debug!("# OUTPUT");
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
    debug!("# INPUTS");
    let entries = if args.inputs.is_empty() {
        conf_input
            .get("entries").expect("malformed conf - no input.entries")
            .as_array().expect("malformed conf - input.entries is not an array")
            .iter().map(|ref v| v.as_str().expect("malformed conf - one entry in input.entries is not a string"))
            .collect()
    } else {
        args.inputs
    };

    // languages
    debug!("# LANGUAGES");
    let languages = conf.get("languages").expect("malformed conf - no languages")
        .as_table().expect("malformed conf - languages is not a table");

    let config = Config { recursive: recursive, entries: entries, output_dir: output_dir,
                          languages: &languages };

    // if a ruccofile was not given as parameter, ensure a local one exists (create if necessary).
    debug!("# RUCCOFILE");
    if let None = args.conf {
        if let Err(e) = ensure_ruccofile_exists(&config) {
            error!("failed to make sure ruccofile exists: {}", e);
        }
    }

    // checking the environment is ready to get files processed.
    debug!("# ENVIRONMENT");
    ensure_dir(&PathBuf::from(config.output_dir))
        .expect("failed to ensure that output directory exists.");

    let pwd = env::current_dir().expect("failed to get current dir!");
    let output_dir = fs::canonicalize(config.output_dir)
        .expect("failed to canonicalize output dir path.");

    // and now recurse files and dump shit!
    debug!("# PROCESSING");
    debug!("## Pushing paths");
    let mut dirs: Vec<PathBuf> = Vec::with_capacity(ESTIMATED_MAX_ACTIONS);
    let mut files: Vec<(PathBuf,PathBuf)> = Vec::with_capacity(ESTIMATED_MAX_ACTIONS);
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
                    debug!("+ dir: {}", relative.display());
                    dirs.push(output_dir.join(&relative));
                } else {
                    let target = output_dir.join(&relative);
                    debug!("+ file: {}", relative.display());
                    files.push((relative.to_owned(), htmlize(target)))
                }
            }
        }
    } else {
        for file in entries.filter(|p| p.is_file()) {
            let parent_dir = file.parent().expect("could not get parent dir of file");
            debug!("+ dir: {}", parent_dir.display());
            dirs.push(parent_dir.to_owned());

            let relative = file.strip_prefix(&pwd)
                .expect("failed to generate a relative path.");
            let target = output_dir.join(&relative);
            debug!("+ file: {}", relative.display());
            files.push((relative.to_owned(), htmlize(target)));
        }
    }

    debug!("## Processing dirs");
    for dir in dirs {
        debug!("- dir: {}", dir.display());
        ensure_dir(&dir)
            .expect("failed to create subdirectory in output dir.");
    }

    debug!("## Processing files");
    let mut res: Vec<io::Result<()>> = vec![];
    files.par_iter()
        .map(|&(ref source, ref target)|
             process_file(&config, source, target))
        .collect_into_vec(&mut res);

    debug!("## Untar resources");
    untar_resources(&output_dir, "classic.tar").unwrap_or_else(|e| {
        panic!("resource extraction failed: {:?}", e);
    });
    ;
    info!("complete!");
}
