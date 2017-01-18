#![feature(plugin)]
#![feature(conservative_impl_trait)]
#![plugin(maud_macros)]

#[macro_use] extern crate maud;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate toml;
extern crate regex;
extern crate hoedown;
extern crate syntect;

pub mod segment;
pub mod languages;
pub mod templates;
pub mod render;

pub use languages::Languages;
pub use render::render;
