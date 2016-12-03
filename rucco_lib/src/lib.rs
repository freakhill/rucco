#![feature(plugin)]
#![plugin(maud_macros)]

#[macro_use] extern crate maud;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate toml; // conf files
extern crate regex;
extern crate hoedown; // markdown
extern crate syntect;

mod section;
use section::*;

pub mod templates;

use std::collections::BTreeMap;
use std::path::{Path};
use regex::{Regex,RegexBuilder};
use hoedown::{Markdown,Html,Render};
use hoedown::renderer::html;
use syntect::parsing::SyntaxSet;
use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::html::highlighted_snippet_for_string;

#[cfg(test)]
mod tests {
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
}

pub fn compute_regex(language: &toml::Value) -> Option<Regex> {
    let table = language.as_table().unwrap();
    let singleline_mark = table.get("singleline")
        .map(|v| v.as_str().expect("MALFORMED RUCCOFILE"));
    let multiline_header_mark = table.get("multiline_header")
        .map(|v| v.as_str().expect("MALFORMED RUCCOFILE"));
    let multiline_footer_mark = table.get("multiline_footer")
        .map(|v| v.as_str().expect("MALFORMED RUCCOFILE"));
    let multiline_margin_mark = table.get("multiline_margin")
        .map(|v| v.as_str().expect("MALFORMED RUCCOFILE"));

    let mut regexp_vec: Vec<String> = Vec::new();
    regexp_vec.push("(?:".to_string()); // global group
    regexp_vec.push(r"(?:\n+)|".to_string()); // empty lines
    if let Some(sl) = singleline_mark {
        // singleline
        regexp_vec.push([r"(?:",
                         r"^[ \t]*", sl, r" ?",
                         r"(?:(?P<doc_sl>[^\n]*\n?)\n*",
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
    regexp_vec.push(r"(?:^(?P<code>[^\n]*\n?)\n*)".to_string()); // codeline
    regexp_vec.push(r")".to_string()); // global group end and repeat

    let final_regexp = regexp_vec.concat();
    match RegexBuilder::new(&final_regexp)
        .multi_line(true)
        .dot_matches_new_line(true)
        .compile() {
            Ok(regexp) => Some(regexp),
            Err(e) => {
                error!("Failed to build regex from language {:?}: {}", language, e);
                None
            }
        }
}

#[derive(Debug,Clone)]
pub enum Segment {
    Title(String,u8),
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

pub struct TitleSplitSections<T: Iterator<Item=Section>> {
    it: T,
    current_doc: Option<String>,
    current_code: Option<String>
}

impl<T: Iterator<Item=Segment>> Iterator for Sections<T> {
    type Item=Section;

    fn title_split(mut self) -> TitleSplitSections<Self> {
        TitleSplitSections {
            it: self,
            current_doc: None,
            current_code: None
        }
    }

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(segment) = self.it.next() {
                match (segment, &mut self.current_doc, &mut self.current_code) {
                    (Segment::Doc(s), cd @ &mut None, &mut None) => {
                        std::mem::replace(cd, Some(s));
                    },
                    (Segment::Doc(_), _cd, &mut None) => {
                        error!("doc and doc together should not happen.");
                        return None;
                    },
                    (Segment::Doc(_), &mut None, _cc) => {
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
                    (Segment::Code(_), &mut None, _cc) => {
                        error!("code and code should not happen.");
                        return None;
                    },
                    (_, _cd, _cc) => {
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

impl<T: Iterator<Item=Section>> Iterator for TitleSplitSections<T> {
    type Item=TitleSplitSection;

    fn next(&mut self) -> Option<Self::Item> {
        None
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
        g => {
            let mut growing_string = g.as_mut().unwrap();
            growing_string.push_str(text);
        }
    }
    match other {
        &mut None => None,
        o => Some(f(std::mem::replace(o, None).unwrap()))
    }
}

lazy_static! {
    static ref ML_L_RE: Regex =
        RegexBuilder::new(r"^[ \t\n]*[^ \t\n]+([^\n]*\n?)$")
        .multi_line(true)
        .dot_matches_new_line(true)
        .compile().expect("Wrong multiline split regexp");
    static ref TITLE_SPLIT_RE: Regex =
        RegexBuilder::new(r"^(#+).*")
        .multi_line(true)
        .dot_matches_new_line(true)
        .compile().expect("Wrong title split regexp!");
}

fn extract_title()

impl<'r, 't> Iterator for RuccoCaptures<'r, 't> {
    type Item=Segment;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let capture = self.fc.next(); // capture
            if let Some(c) = capture {
                match (c.name("doc_sl"),c.name("doc_ml"),c.name("doc_ml_h"),c.name("doc_ml_l"),c.name("code")) {
                    (None,None,None,None,None) => {}, // ignore
                    (Some(ref sl),_,_,_,_) => { // single comment line
                        //if Some(capture) = TITLE_SPLIT_RE.captures_iter(sl).first()
                        let maybe_return = append(sl, &mut self.current_doc, &mut self.current_code, |c| Segment::Code(c));
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,Some(ref ml),_,_,_) => { // multiline no margin
                        let maybe_return = append(ml, &mut self.current_doc, &mut self.current_code, |c| Segment::Code(c));
                        if maybe_return.is_some() { return maybe_return; };
                    },
                    (_,_,Some(ref ml_h),Some(ref ml_l),_) => {
                        let maybe_return = append(ml_h, &mut self.current_doc, &mut self.current_code, |c| Segment::Code(c));
                         // we remove the margin from the lines
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
pub struct Languages {
    computed: BTreeMap<String, Option<Regex>>,
    raw: toml::Table
}

impl Languages {
    pub fn new(raw: toml::Table) -> Languages {
        Languages {computed: BTreeMap::new(), raw: raw}
    }

    fn get(&mut self, l: &str) -> &Option<Regex> {
        let lang_raw_value = self.raw.get(l);
        let entry = self.computed.entry(l.to_owned());
        entry.or_insert_with(|| {
            match lang_raw_value {
                Some(lang) => compute_regex(lang),
                None => None
            }
        })
    }
}

thread_local! {
    static SS: SyntaxSet = SyntaxSet::load_defaults_nonewlines();
    static TS: ThemeSet = ThemeSet::load_defaults();
    static THEME: Theme = TS.with(|ts| ts.themes["base16-ocean.dark"].clone());
}

pub fn render
    (languages: &mut Languages,
     extension: &str,
     source_text: &str,
     source_path: &Path,
     css_rel_path: &str) -> Option<String>
{
    SS.with(|ss| {
        if let Some(syntax_def) = ss.find_syntax_by_extension(extension) {
            if let &Some(ref regex) = languages.get(extension) {
                let sections: Vec<Section> = regex
                    .rucco_captures_iter(source_text)
                    .into_sections_iter()
                    .title_split()
                    .map(|s| render_section(syntax_def,s)).collect();
                Some(templates::classic::render(vec![].iter(),
                                                css_rel_path,
                                                source_path,
                                                sections.iter()))
            } else {
                debug!("could not build section parser for extension: {}", extension);
                None
            }
        } else {
            debug!("no sublime syntax available for extension: {}", extension);
            None
        }
    })
}

fn render_section(syntax_def: &SyntaxDefinition, raw: TitleSplitSection) -> RenderedTitleSplitSection {
    let mut md_html = Html::new(html::Flags::empty(), 0);

    match raw {
        TitleSplitSection::Title(t) => {
            let md_doc = Markdown::new(t.text.as_str());
            let title_html = md_html.render(&md_doc).to_str().unwrap_or("<p>failed to render</p>").to_owned();
            RenderedTitleSplitSection::Title(Title {
                text: title_html,
                level: t.level
            })
        },
        TitleSplitSection::Section(s) => {
            let md_doc = Markdown::new(s.doc.as_str());
            let doc_html = md_html.render(&md_doc).to_str().unwrap_or("<p>failed to render</p>").to_owned();
            THEME.with(move |theme| {
                let code_html = highlighted_snippet_for_string(&s.code, syntax_def, theme);
                RenderedTitleSplitSection::Section(Section {
                    doc: doc_html,
                    code: code_html
                })
            })
        }
    }
}
