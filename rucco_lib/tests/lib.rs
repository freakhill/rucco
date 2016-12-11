extern crate env_logger;
use toml;
use std::collections::BTreeMap;
use super::*;

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
    for capture in r.segments_iter(C_SAMPLE) {
        println!("segments_ok: {:?}", capture);
    };
}

#[test]
fn into_sections_iter_ok() {
    let r = compute_regex(&create_c_language()).expect("failed to create c language regex");
    for section in r.segments_iter(C_SAMPLE).into_sections_iter() {
        println!("section: {:?}", section);
    };
}

#[test]
fn render_ok() {
    env_logger::init().unwrap();
    let mut raw: toml::Table = BTreeMap::new();
    let c = create_c_language();
    raw.insert("c".to_string(), c);
    let mut langs = Languages {
        computed: BTreeMap::new(),
        raw: raw
    };
    if let Some(rendered) = render(&mut langs, "c", C_SAMPLE, "../style.css") {
        println!("file: {:#?}", rendered);
    } else {
        panic!("failed to generate sections");
    }
}
