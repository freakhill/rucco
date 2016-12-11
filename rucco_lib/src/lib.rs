#![feature(plugin)]
#![plugin(maud_macros)]

#[macro_use] extern crate maud;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate toml; // conf files
extern crate regex;
extern crate hoedown; // markdown
extern crate syntect;

mod compute_regex;
use compute_regex::compute_regex;

mod segment;
use segment::*;

pub mod templates;

use std::collections::BTreeMap;
use std::path::{Path};
use std::convert::{From,Into};
use regex::{Regex,RegexBuilder};
use hoedown::{Markdown,Html,Render};
use hoedown::renderer::html;
use syntect::parsing::SyntaxSet;
use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::html::highlighted_snippet_for_string;

/// Iterator<Item=Option<Segment>>
pub struct Segments<'r, 't> {
    /// our regex captures that split doc from code
    fc: regex::FindCaptures<'r, 't>,
    /// necessary for splitting multilines into titles and doc lines
    multiline_doc_fc: Option<regex::FindCaptures<'r, 't>>
}

/// Iterator<Item=Segment>
/// also merge segments
pub struct CompactSegments<'r, 't> {
    segments: Segments<'r, 't>
}

impl<'r, 't> From<Segments<'r, 't>> for CompactSegments<'r, 't> {
    fn from(segments: Segments<'r, 't>) -> Self {
        CompactSegments { segments: self }
    }
}

impl<'r, 't> Iterator for CompactSegments<'r, 't> {
    type Item=Segment;

    fn next(&mut self) -> Option<Segment> {
        // change all og this to use 1 buffered extensible segment
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

lazy_static! {
    static ref ML_MARGIN_L_RE: Regex =
        RegexBuilder::new(r"^[ \t\n]*[^ \t\n]+ ?([^\n]*\n?)$")
        .multi_line(true)
        .dot_matches_new_line(true)
        .compile().expect("Wrong multiline split regexp");
    static ref ML_NOMARGIN_L_RE: Regex =
        RegexBuilder::new(r"^[\n\t]*([^\n]*\n?)$")
        .multi_line(true)
        .dot_matches_new_line(true)
        .compile().expect("Wrong multiline split regexp");
    static ref TITLE_SPLIT_RE: Regex =
        RegexBuilder::new(r"^(#+).*")
        .multi_line(true)
        .dot_matches_new_line(true)
        .compile().expect("Wrong title split regexp!");
}


impl<'r, 't> Iterator for Segments<'r, 't> {
    type Item=Option<Segment>;

    fn title_or_doc_segment(&mut self, line: &'t str) -> Segment {
        if let Some(heading_capture) = TITLE_SPLIT_RE.captures_iter(line).first() {
            Segment::Title((heading_capture.length(), line.toString()))
        } else {
            Segment::Doc(line.toString())
        }
    }

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(bm) = self.multiline_doc_fc {
            /// in multiline doc context
            if let Some(c) = bm.next() {
                if let Some(l) = c.at(1) {
                    Some(title_or_doc_segment(l))
                } else {
                    Some(None)
                }
            } else {
                self.multiline_doc_fc = None;
                Some(None)
            }
        } else {
            /// primary context
            let capture = self.fc.next();
            if let Some(c) = capture {
                match (c.name("doc_sl"),c.name("doc_ml"),c.name("doc_ml_h"),c.name("doc_ml_l"),c.name("code")) {
                    (None,None,None,None,None) => Some(None), // ignore
                    (Some(ref sl),_,_,_,_) => // single comment line
                        Some(title_or_doc_segment(sl)),
                    (_,Some(ref ml),_,_,_) => { // multiline no margin
                        self.multiline_doc_fc = Some(ML_NOMARGIN_L_RE.captures_iter(ml));
                        Some(None)
                    },
                    (_,_,Some(ref ml_h),Some(ref ml_l),_) => { // multiline with margin
                        self.multiline_doc_fc = Some(ML_MARGIN_L_RE.captures_iter(ml_l));
                        Some(title_or_doc_segment(ml_h))
                    },
                    (_,_,_,_,Some(code)) =>  // code
                        Some(Segment::Code(code)),
                    (_,_,_,_,_) => {
                        error!("Something went wrong when processing Segments");
                        None
                    }
                }
            } else {
                None
            }
        }
    }
}

pub fn segments<'r, 't>(r: &'r regex::Regex, source: &'t str)
                        -> Segments<'r, 't> {
    Segments {
        fc: r.captures_iter(source),
        multiline_doc_fc: None
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

/// ----------------------------------------------------------------------------
/// Rendering a segment

thread_local! {
    static THEME_SET: ThemeSet = ThemeSet::load_defaults();
    static THEME: Theme = THEME_SET.with(|ts| ts.themes["base16-ocean.dark"].clone());
}

fn render_segment(syntax_def: &SyntaxDefinition, segment: Segment) -> RenderedSegment {
    let mut md_html = Html::new(html::Flags::empty(), 0);

    match segment {
        Segment::Title((h, title)) => {
            let md_doc = Markdown::new(title.as_str());
            let title_html = md_html.render(&md_doc).to_str().unwrap_or("<p>failed to render</p>").to_owned();
            Segment::Title((h, title_html))
        },
        Segment::Doc(doc) => {
            let md_doc = Markdown::new(doc.as_str());
            let doc_html = md_html.render(&md_doc).to_str().unwrap_or("<p>failed to render</p>").to_owned();
            Segment::Doc(doc_html)
        },
        Segment::Code(code) => {
            THEME.with(move |theme| {
                let code_html = highlighted_snippet_for_string(&code, syntax_def, theme);
                Segment::Code(code_html)
            })
        }
    }
}

/// ----------------------------------------------------------------------------
/// Rendering a source file

thread_local! {
    static SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_nonewlines();
}

pub fn render
    (languages: &mut Languages,
     extension: &str,
     source_text: &str,
     source_path: &Path,
     css_rel_path: &str) -> Option<String>
{
    SYNTAX_SET.with(|ss| {
        if let Some(syntax_def) = ss.find_syntax_by_extension(extension) {
            if let &Some(ref regex) = languages.get(extension) {
                let sections: Vec<Segment> =
                    CompactSegments::from(segments(regex, source_text))
                    .map(|s| render_segment(syntax_def,s)).collect();
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
