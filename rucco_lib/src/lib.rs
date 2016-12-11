#![feature(plugin)]
#![plugin(maud_macros)]

#[macro_use] extern crate maud;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate toml; // conf files
extern crate regex;
extern crate hoedown; // markdown
extern crate syntect;

mod segment;
mod compute_regex;
mod languages;

pub mod templates;
pub mod render;

pub use languages::Languages;
pub use render::render;

use std::collections::BTreeMap;
use std::path::{Path};
use regex::{Regex,RegexBuilder};
use hoedown::{Markdown,Html,Render};
use hoedown::renderer::html;
use syntect::parsing::SyntaxSet;
use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::html::highlighted_snippet_for_string;

// fn append<F>(new_segment: Segment,
//              buffered_segment: &mut Segment
//              //growing: &mut Option<String>,
//              //other: &mut Option<String>,
//              f: F) -> Segment
//     where F: Fn(String) -> Segment{
//     // append or create new code segment
//     match growing {
//         &mut None => {std::mem::replace(growing, Some(text.to_string()));},
//         g => {
//             let mut growing_string = g.as_mut().unwrap();
//             growing_string.push_str(text);
//         }
//     }
//     match other {
//         &mut None => None,
//         o => Some(f(std::mem::replace(o, None).unwrap()))
//     }
// }
