//! # Rucco
//! A Docco clone in Rust (with multiline support)

#![feature(plugin)]
#![plugin(embed)]

#[macro_use]
extern crate clap;

use clap::{Arg, App, SubCommand};
use std::collections::HashMap;

fn parse_params() {
    let matches =
        clap_app!(rucco =>
                  (version: "0.0.1")
                  (author "Jojo <gall.johan@linecorp.com>")
                  (about "Docco with multiline support in RUST")
                  (@arg languages: -L --languages +takes_value
                   "use a custom languages.json")
                  (@arg layout: -l --layout +takes_value
                   "choose a layout (parallel, linear or classic)")
                  (@arg output: -o --output +takes_value
                  "output to a given folder")
                  (@arg css: -c --css +takes_value
                  "use a custom css file")
                  (@arg template: -t --template +takes_value
                  "use a custom jst template")
                  (@arg extension: -e --extension +takes_value
                  "assume a file extension for all inputs")
                  (@arg recursive: -r --recursive "Explore folders recursively")
        ).get_matches();
}

fn read_config() {

}

fn recurse_files() {

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
    let files: HashMap<Vec<u8>, Vec<u8>> = embed!("resources");
    for (name, content) in files {
        println!("{}: \"{}\"", String::from_utf8(name).unwrap(), String::from_utf8(content).unwrap().trim());
    }
}
