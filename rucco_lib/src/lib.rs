#![feature(plugin)]
#![plugin(maud_macros)]

#[macro_use]

extern crate maud;
extern crate log; /// for logging...
extern crate toml; /// for configuration files
extern crate nom;

mod section;
pub mod templates;

use std::collections::BTreeMap;
use std::path::{PathBuf};

/// This will hold our final configuration (after merging clap data and ruccofile data).
pub struct Config<'a> {
    pub recursive: bool,
    pub entries: Vec<&'a str>,
    pub output_dir: &'a str,
    pub languages: &'a toml::Table
}

pub fn run() {
}

pub fn document() {
}

pub fn parse(config: &Config, path: &PathBuf) {
}

pub fn format() {
}

pub fn version() {
}
