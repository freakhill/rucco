extern crate env_logger;
extern crate rucco_lib;
extern crate toml;

use std::collections::BTreeMap;
use rucco_lib::*;
use rucco_lib::languages::compute_regex;

const C_SAMPLE: &'static str = r"
int a = 12;

/* first block */

// # mooo mooo
// moomfomfomfoof

/*** kakarot
 * - zozo
 * - dodo
 */

int b = 77;

/*
## tatarot
momo
*/

char* gororo = 'm'; // ignpre me!!!

/*
 * # moo
 * yo
*/

";

fn create_c_language() -> toml::Value {
    let mut c: toml::Table = BTreeMap::new();
    c.insert("name".to_string(), toml::Value::String("C".to_string()));
    c.insert("singleline".to_string(), toml::Value::String(r"//+".to_string()));
    c.insert("multiline_header".to_string(), toml::Value::String(r"/\*+".to_string()));
    c.insert("multiline_footer".to_string(), toml::Value::String(r"\*+/".to_string()));
    c.insert("multiline_margin".to_string(), toml::Value::String(r"\*+".to_string()));
    toml::Value::Table(c)
}

#[test]
fn regex_parse_ok() {
    let r = compute_regex(&create_c_language()).expect("failed to create c language regex");
    for capture in r.captures_iter(C_SAMPLE) {
        println!("regex_parse_ok: {:?}", capture);
    };
}

#[test]
fn segments_ok() {
    let r = compute_regex(&create_c_language()).expect("failed to create c language regex");
    for capture in rucco_lib::segment::extract_segments(&r, C_SAMPLE) {
        println!("segments_ok: {:?}", capture);
    };
}

#[test]
fn render_ok() {
    env_logger::init().unwrap();
    let mut raw: toml::Table = BTreeMap::new();
    let c = create_c_language();
    raw.insert("c".to_string(), c);
    let mut langs = Languages::new(raw);
    if let Some(rendered) = render(&mut langs, "c", C_SAMPLE, &std::path::Path::new("./source_path.c"), "../style.css") {
        println!("file: {:#?}", rendered);
    } else {
        panic!("failed to generate sections");
    }
}
