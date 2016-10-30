#![feature(plugin)]
#![plugin(maud_macros)]

#[macro_use] extern crate maud;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate toml; // conf files
extern crate regex;
extern crate hoedown; // markdown

mod section;
use section::*;

pub mod templates;
pub mod config;

use std::collections::BTreeMap;
use std::path::{PathBuf};
use regex::{Regex,RegexBuilder};
use hoedown::{Markdown,Html,Render};
use hoedown::renderer::html;

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
        let r = compute_regex(&create_c_language()).expect("failed to create c language regex");
        for capture in r.captures_iter(C_SAMPLE) {
            println!("regex_parse_ok: {:?}", capture);
        };
    }

    #[test]
    fn rucco_captures_ok() {
        let r = compute_regex(&create_c_language()).expect("failed to create c language regex");
        for capture in r.rucco_captures_iter(C_SAMPLE) {
            println!("rucco_captures_ok: {:?}", capture);
        };
    }

    #[test]
    fn into_sections_iter_ok() {
        let r = compute_regex(&create_c_language()).expect("failed to create c language regex");
        for section in r.rucco_captures_iter(C_SAMPLE).into_sections_iter() {
            println!("section: {:?}", section);
        };
    }
}

pub fn compute_regex(language: &toml::Value) -> Option<Regex> {
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
pub enum Segment {
    Code(String),
    Doc(String),
}

pub struct RuccoCaptures<'r, 't> {
    fc: regex::FindCaptures<'r, 't>,
    current_doc: Option<String>,
    current_code: Option<String>
}

impl<'r, 't> RuccoCaptures<'r, 't> {
    pub fn into_sections_iter(self) -> Sections<RuccoCaptures<'r, 't>> {
        Sections {
            it: self,
            current_doc: None,
            current_code: None
        }
    }
}

pub struct Sections<T: Iterator<Item=Segment>> {
    it: T,
    current_doc: Option<String>,
    current_code: Option<String>
}

impl<T: Iterator<Item=Segment>> Iterator for Sections<T> {
    type Item=Section;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(segment) = self.it.next() {
                match (segment, &mut self.current_doc, &mut self.current_code) {
                    (Segment::Doc(s), cd @ &mut None, &mut None) => {
                        std::mem::replace(cd, Some(s));
                    },
                    (Segment::Doc(s), cd, &mut None) => {
                        error!("doc and doc together should not happen.");
                        return None;
                    },
                    (Segment::Doc(s), &mut None, cc) => {
                        error!("doc should not happen with code and not previous doc.");
                        return None;
                    },
                    (Segment::Code(s), &mut None, &mut None) => {
                        return Some(Section{doc: String::from(""), code: s});
                    },
                    (Segment::Code(s), cd, &mut None) => {
                        let doc = std::mem::replace(cd ,None).unwrap();
                        return Some(Section{doc: doc, code: s});
                    },
                    (Segment::Code(s), &mut None, cc) => {
                        error!("code and code should not happen.");
                        return None;
                    },
                    (_, cd, cc) => {
                        error!("previously failed to emit a section.");
                        return None;
                    },
                }
            } else {
                match (&mut self.current_doc, &mut self.current_code) {
                    (&mut None, &mut None) => { return None; },
                    (mut doc, &mut None) => {
                                             let d = std::mem::replace(doc, None);
                                             return Some(Section{doc: d.unwrap(), code: "".to_string()});
                    },
                    (_, _) => {
                        error!("this should not happen...");
                        return None;
                    }
                }
            }
        }
    }
}

fn append<F>(text: &str,
             growing: &mut Option<String>,
             other: &mut Option<String>,
             f: F) -> Option<Segment>
    where F: Fn(String) -> Segment{
    // append or create new code segment
    match growing {
        &mut None => {std::mem::replace(growing, Some(text.to_string()));},
        g => {g.as_mut().unwrap().push_str(text);}
    }
    match other {
        &mut None => None,
        o => Some(f(std::mem::replace(o, None).unwrap()))
    }
}

impl<'r, 't> Iterator for RuccoCaptures<'r, 't> {
    type Item=Segment;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let capture = self.fc.next(); // capture
            if let Some(c) = capture {
                match (c.name("doc_sl"),c.name("doc_ml"),c.name("doc_ml_h"),c.name("doc_ml_l"),c.name("code")) {
                    (None,None,None,None,None) => {}, // ignore
                    (Some(ref sl),_,_,_,_) => { // single comment line
                        let maybe_return = append(sl, &mut self.current_doc, &mut self.current_code, |c| Segment::Code(c));
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,Some(ref ml),_,_,_) => { // multiline no margin
                        let maybe_return = append(ml, &mut self.current_doc, &mut self.current_code, |c| Segment::Code(c));
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,_,Some(ref ml_h),Some(ref ml_l),_) => {
                        let maybe_return = append(ml_h, &mut self.current_doc, &mut self.current_code, |c| Segment::Code(c));
                        lazy_static! {
                            static ref ML_L_RE: Regex = Regex::new(r"(?ms)^[ \t\n]*[^ \t\n]+([^\n]*\n?)$").unwrap();
                        } // we remove the margin from the lines
                        for c in ML_L_RE.captures_iter(ml_l) {
                            if let Some(l) = c.at(1) {
                                append(l, &mut self.current_doc, &mut self.current_code, |c| Segment::Code(c));
                            }
                        }
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,_,_,_,Some(code)) => {
                        let maybe_return = append(code, &mut self.current_code, &mut self.current_doc, |d| Segment::Doc(d));
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,_,_,_,_) => {
                        error!("SOMETHING WENT WRONT WHEN AGGLOMERATING CAPTURES");
                        return None;
                    }
                }
            } else {
                if self.current_doc.is_some() {
                    let doc = std::mem::replace(&mut self.current_doc, None);
                    return Some(Segment::Doc(doc.unwrap()));
                }
                if self.current_code.is_some() {
                    let code = std::mem::replace(&mut self.current_code, None);
                    return Some(Segment::Code(code.unwrap()));
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
    fn get(&mut self, l: String) -> &Option<Regex> {
        let lang_raw_value = self.raw.get(&l);
        let entry = self.computed.entry(l);
        entry.or_insert_with(|| {
            match lang_raw_value {
                Some(ref lang) => compute_regex(lang),
                None => None
            }
        })
    }
}

fn sections
    (languages: &mut Languages, extension: &str, source: &str) -> Option<Vec<RenderedSection>> {
    if let &Some(ref regex) = languages.get(String::from(extension)) {
        Some(regex.rucco_captures_iter(source).into_sections_iter().map(render_section).collect())
    } else {
        warn!("could not find language for extension: {}", extension);
        None
    }
}

fn render_section(raw: Section) -> RenderedSection {
    let mut html = Html::new(html::Flags::empty(), 0);
    let doc = Markdown::new(raw.doc.as_str());
    RenderedSection {
        doc: html.render(&doc).to_str().unwrap_or("<p>failed to render</p>").to_owned(),
        code: raw.code
    }
}
