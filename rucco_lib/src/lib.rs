#![feature(plugin)]
#![plugin(maud_macros)]

#[macro_use] extern crate maud;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log; /// for logging...
extern crate toml; /// for configuration files
extern crate regex;

mod section;

pub mod templates;
pub mod config;

use std::collections::BTreeMap;
use std::path::{PathBuf};
use regex::{Regex,RegexBuilder};

#[cfg(test)]
mod tests {
    use toml;
    use std::collections::BTreeMap;
    use super::*;

    const C_SAMPLE: &'static str = r"
int a = 12;

/* first block */

// mooo mooo
// moomfomfomfoof

/*** kakarot
 * zozo
 * dodo
 */

int b = 77;

/*
tatarot
momo
*/

char* gororo = 'm'; // ignpre me!!!

/*
 * moo
*/

";

    fn create_c_language() -> toml::Value {
        let mut c: toml::Table = BTreeMap::new();
        c.insert("singleline".to_string(), toml::Value::String(r"//+".to_string()));
        c.insert("multiline_header".to_string(), toml::Value::String(r"/\*+".to_string()));
        c.insert("multiline_footer".to_string(), toml::Value::String(r"\*+/".to_string()));
        c.insert("multiline_margin".to_string(), toml::Value::String(r"\*+".to_string()));
        toml::Value::Table(c)
    }

    #[test]
    fn regex_parse_ok() {
        let r = compute_regex(create_c_language()).expect("failed to create c language regex");
        for capture in r.captures_iter(C_SAMPLE) {
            println!("regex_parse_ok: {:?}", capture);
        };
    }

    #[test]
    fn rucco_captures_ok() {
        let r = compute_regex(create_c_language()).expect("failed to create c language regex");
        for capture in r.rucco_captures_iter(C_SAMPLE) {
            println!("rucco_captures_ok: {:?}", capture);
        };
    }
}

pub fn compute_regex(language: toml::Value) -> Option<Regex> {
    let table = language.as_table().unwrap();
    let singleline_mark = table.get("singleline").map(|v| v.as_str().unwrap());
    let multiline_header_mark = table.get("multiline_header").map(|v| v.as_str().unwrap());
    let multiline_footer_mark = table.get("multiline_footer").map(|v| v.as_str().unwrap());
    let multiline_margin_mark = table.get("multiline_margin").map(|v| v.as_str().unwrap());

    let mut regexp_vec: Vec<String> = Vec::new();
    regexp_vec.push("(?:".to_string()); // global group
    regexp_vec.push(r"(?:\n+)|".to_string()); // empty lines
    if let Some(sl) = singleline_mark {
        // singleline
        regexp_vec.push([r"(?:",
                         r"^[ \t]*", sl, r"(?P<doc_sl>[^\n]*\n?)\n*",
                         r")|"].concat());
    };
    if let (Some(mh), Some(mf), Some(mm)) = (multiline_header_mark, multiline_footer_mark, multiline_margin_mark) {
        // multiline with margin
        regexp_vec.push([r"(?:",
                         r"^[ \t]*", mh, r"(?P<doc_ml_h>[^\n]*\n?)\n*", // header and potential doc there
                         r"(?P<doc_ml_l>(?:[ \t]*", mm, r"[^\n]*\n*)*[ \t]*)", mf, // lines
                         r")|"].concat());
        // this is far from foolproof but i do not want to support code nasty enough to break
    };
    if let (Some(mh), Some(mf)) = (multiline_header_mark, multiline_footer_mark) {
        // multiline without margin
        regexp_vec.push([r"(?:",
                         r"^[ \t]*", mh, r"(?P<doc_ml>.*?)", mf,
                         r")|"].concat());
    };
    regexp_vec.push(r"(?:^(?P<code>[^\n]*)\n*)".to_string()); // codeline
    regexp_vec.push(r")".to_string()); // global group end and repeat

    let final_regexp = regexp_vec.concat();
    match RegexBuilder::new(&final_regexp)
        .multi_line(true)
        .dot_matches_new_line(true)
        .compile() {
        Ok(regexp) => {
            Some(regexp)
        },
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

#[derive(Debug,Clone)]
pub enum RawSegment {
    Code(String),
    Doc(String),
}

pub struct RuccoCaptures<'r, 't> {
    fc: regex::FindCaptures<'r, 't>,
    current_doc: Option<String>,
    current_code: Option<String>
}

fn append<F>(text: &str,
             growing: &mut Option<String>,
             other: &mut Option<String>,
             f: F) -> Option<RawSegment>
    where F: Fn(String) -> RawSegment{
    // append or create new code segment
    if growing.is_some() { // wait for non lexical lifetimes....
        growing.as_mut().unwrap().push_str(text);
    } else {
        std::mem::replace(growing, Some(text.to_string()));
    }
        // if we had doc in, push it out
    if other.is_some() { // wait for non lexical lifetimes....
            let out = std::mem::replace(other, None);
        return Some(f(out.unwrap()));
    }
    None
}

impl<'r, 't> Iterator for RuccoCaptures<'r, 't> {
    type Item=RawSegment;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let capture = self.fc.next(); // capture
            if let Some(c) = capture {
                match (c.name("doc_sl"),c.name("doc_ml"),c.name("doc_ml_h"),c.name("doc_ml_l"),c.name("code")) {
                    (None,None,None,None,None) => {}, // ignore
                    (Some(ref sl),_,_,_,_) => { // single comment line
                        let maybe_return = append(sl, &mut self.current_doc, &mut self.current_code, |c| RawSegment::Code(c));
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,Some(ref ml),_,_,_) => { // multiline no margin
                        let maybe_return = append(ml, &mut self.current_doc, &mut self.current_code, |c| RawSegment::Code(c));
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,_,Some(ref ml_h),Some(ref ml_l),_) => {
                        let maybe_return = append(ml_h, &mut self.current_doc, &mut self.current_code, |c| RawSegment::Code(c));
                        lazy_static! {
                            static ref ML_L_RE: Regex = Regex::new(r"(?ms)^[ \t\n]*[^ \t\n]+([^\n]*\n?)$").unwrap();
                        } // we remove the margin from the lines
                        for c in ML_L_RE.captures_iter(ml_l) {
                            if let Some(l) = c.at(1) {
                                append(l, &mut self.current_doc, &mut self.current_code, |c| RawSegment::Code(c));
                            }
                        }
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,_,_,_,Some(code)) => {
                        let maybe_return = append(code, &mut self.current_code, &mut self.current_doc, |d| RawSegment::Doc(d));
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,_,_,_,_) => panic!("NYI")
                }
            } else {
                if self.current_doc.is_some() {
                    let doc = std::mem::replace(&mut self.current_doc, None);
                    return Some(RawSegment::Doc(doc.unwrap()));
                }
                if self.current_code.is_some() {
                    let code = std::mem::replace(&mut self.current_code, None);
                    return Some(RawSegment::Code(code.unwrap()));
                }
                return None;
            }
        }
    }
}

pub trait IntoRuccoCaptures<'r, 't> {
    fn rucco_captures_iter(&'r self, &'t str) -> RuccoCaptures<'r, 't>;
}

impl<'r, 't> IntoRuccoCaptures<'r, 't> for regex::Regex {
    fn rucco_captures_iter(&'r self, source: &'t str) -> RuccoCaptures<'r, 't> {
        RuccoCaptures {
            fc: self.captures_iter(source),
            current_doc: None,
            current_code: None
        }
    }
}


// figure out Arc, Mutex etc. afterwards
struct Languages {
    computed: BTreeMap<String, Option<Regex>>,
    raw: BTreeMap<String, toml::Value>
}

impl Languages {
    fn get(&mut self, l: String) -> Option<Regex> {
        None
    }
}

fn extract_segments(languages: &mut Languages, extension: &str, source: &str) -> Option<()> {
    if let Some(regex) = languages.get(String::from(extension)) {
        for capture in regex.rucco_captures_iter(source) {
            println!("capture: {:?}", capture);
        };
        Some(())
    } else {
        None
    }
}

// fn normalize_segments(raw_segments: Iter<&RawSegments>) {
//     let mut current_doc_segment: Option<DocSegment> = None;
//     let mut current_code_segment: Option<CodeSegment> = None;

//     let mut output: Vec<(DocSegment,CodeSegment)> = vec![];

//     // instead of for create a new "normalizing iterator"
//     for seg in raw_segments {
//         match (seg, current_doc_segment, current_code_segment) {
//             (DocSegment, None, None) => current_doc_segment <- seg,
//             (DocSegment, Some(doc_segment), None) => mergedocsegments,
//             (DocSegment, _, _) => panic!(""),
//             (CodeSegment, None, None) => output.push(empty_doc.clone(), seg),
//             (CodeSegment, Some(doc_segment), None) => output.push(doc_segment, seg),
//             (_, _, _) => panic!(""),
//         }
//     }

//     if let Some(doc) = current_doc_segment {
//         output.push(doc, empty_code.clone());
//     }
//     output
// }

// fn render_segments(sections: Iterator<&(DocSegment, CodeSegment)>)
//                        -> Vec<Section> {

//     let sections: Vec<Section> = sections.map(|(doc, code)| {
//         let (heading_level, heading) = extract_heading(doc);
//         let doc_html = render_doc(doc);
//         let code_html = render_code(code);
//         Section{...};
//     }).collect(); // need to count stuff...

//     render_file(sections) // and use that to write a file
// }
