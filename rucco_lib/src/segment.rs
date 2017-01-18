use regex;
use std;
use regex::{Regex,RegexBuilder};

#[derive(Debug,Clone)]
pub enum Segment {
    Title((u8, String)), // "## lol" -> (2, "## lol")
    Code(String),
    Doc(String)
}

pub use segment::Segment as RenderedSegment;

pub fn extract_segments<'r, 't>(r: &'r regex::Regex, source: &'t str)
                                -> impl Iterator<Item=Segment>+'r+'t
//ExtractCompactSegments<'r, 't>
{
    ExtractCompactSegments {
        segments: ExtractSegments {
            code_and_doc_captures: r.captures_iter(source),
            title_and_doc_in_multiline_capture: None
        }
    }
}

// -----------------------------------------------------------------------------

// impl Segment {
//     // extend a segment with new data and return Empty
//     // OR
//     // replace a segment with new data and return the old segment
//     fn push(&mut self, s: Segment) -> Option<Segment> {
//         match (self, s) {
//             (_, Segment::Empty) => Segment::Empty,
//             (Segment::Empty, _) => {
//                 // replace stuff
//                 // ...
//                 None
//             },
//             (ref mut Segment::Title(_), Segment::Title(_)) => {
//                 error!("two title segments with nothing in between!??");
//                 None
//             },
//             (ref mut Segment::Code(c), Segment::Code(cc)) => {
//             },
//             (ref mut Segment::Doc(d), Segment::Doc(dd)) => {
//             },
//             (ref mut a, b) => {
//                 panic!("lol");
//                 // replace b by a and return a
//             }
//         }
//     }
// }

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

// -----------------------------------------------------------------------------
// ## Extracting segments

/// Iterator<Item=Option<Segment>>
struct ExtractSegments<'r, 't> {
    /// our regex captures that split doc from code
    code_and_doc_captures: regex::CaptureMatches<'r, 't>,
    /// necessary for splitting multilines into titles and doc lines
    title_and_doc_in_multiline_capture: Option<regex::CaptureMatches<'r, 't>>
}

lazy_static! {
    /// regex to extract title and doc from multiline comments
    static ref ML_MARGIN_L_RE: Regex = // multiline with margin
        RegexBuilder::new(r"^[ \t\n]*[^ \t\n]+ ?([^\n]*\n?)$")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build().expect("Wrong multiline split regexp");
    static ref ML_NOMARGIN_L_RE: Regex = // multiline with no margin
        RegexBuilder::new(r"^[\n\t]*([^\n]*\n?)$")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build().expect("Wrong multiline split regexp");
    static ref TITLE_SPLIT_RE: Regex = // split title between '##' and 'actual title'
        RegexBuilder::new(r"^(#+).*")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build().expect("Wrong title split regexp!");
}

// impl<'r, 't> ExtractSegments<'r, 't> {
//     fn title_or_doc_segment(&mut self, line: &'t str) -> Segment {
//         if let Some(heading_capture) = TITLE_SPLIT_RE.captures_iter(line).first() {
//             Segment::Title((heading_capture.length(), line.toString()))
//         } else {
//             Segment::Doc(line.toString())
//         }
//     }
// }

fn title_or_doc_segment(line: &str) -> Segment {
    if let Some(heading_capture) = TITLE_SPLIT_RE.captures_iter(line).next() {
        if let Some(h) = heading_capture.get(1) {
            Segment::Title(( (h.end() - h.start()) as u8, line.to_owned()))
            // yes, we suppose '#' is 1 byte >_> we dun trappin~
        } else {
            Segment::Doc(line.to_owned())
        }
    } else {
        Segment::Doc(line.to_owned())
    }
}

impl<'r, 't> Iterator for ExtractSegments<'r, 't> {
    type Item=Option<Segment>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(bm) = self.title_and_doc_in_multiline_capture {
            // in multiline doc context
            if let Some(c) = bm.next() {
                if let Some(l) = c.get(1) {
                    Some(Some(title_or_doc_segment(l.as_str())))
                } else {
                    Some(None)
                }
            } else {
                self.title_and_doc_in_multiline_capture = None;
                Some(None)
            }
        } else {
            // primary context
            let capture = self.code_and_doc_captures.next();
            if let Some(c) = capture {
                match (c.name("doc_sl"),c.name("doc_ml"),c.name("doc_ml_h"),c.name("doc_ml_l"),c.name("code")) {
                    (None,None,None,None,None) => Some(None), // ignore
                    (Some(sl),_,_,_,_) => // single comment line
                        Some(Some(title_or_doc_segment(sl.as_str()))),
                    (_,Some(ml),_,_,_) => { // multiline no margin
                        self.title_and_doc_in_multiline_capture = Some(ML_NOMARGIN_L_RE.captures_iter(ml.as_str()));
                        Some(None)
                    },
                    (_,_,Some(ml_h),Some(ml_l),_) => { // multiline with margin
                        self.title_and_doc_in_multiline_capture = Some(ML_MARGIN_L_RE.captures_iter(ml_l.as_str()));
                        Some(Some(title_or_doc_segment(ml_h.as_str())))
                    },
                    (_,_,_,_,Some(code)) =>  // code
                        Some(Some(Segment::Code(code.as_str().to_owned()))),
                    (_,_,_,_,_) => {
                        error!("Something went wrong when processing ExtractSegments");
                        None
                    }
                }
            } else {
                None
            }
        }
    }
}

// -----------------------------------------------------------------------------
// ## Compacting segments

/// Iterator<Item=<Segment>>
pub struct ExtractCompactSegments<'r, 't> {
    segments: ExtractSegments<'r, 't>
}

impl<'r, 't> Iterator for ExtractCompactSegments<'r, 't> {
    type Item=Segment;

    fn next(&mut self) -> Option<Segment> {
        // change all og this to use 1 buffered extensible segment
        panic!("lol");
        // loop {
        //     if let Some(segment) = self.it.next() {
        //         match (segment, &mut self.current_doc, &mut self.current_code) {
        //             (Segment::Doc(s), cd @ &mut None, &mut None) => {
        //                 std::mem::replace(cd, Some(s));
        //             },
        //             (Segment::Doc(_), _cd, &mut None) => {
        //                 error!("doc and doc together should not happen.");
        //                 return None;
        //             },
        //             (Segment::Doc(_), &mut None, _cc) => {
        //                 error!("doc should not happen with code and not previous doc.");
        //                 return None;
        //             },
        //             (Segment::Code(s), &mut None, &mut None) => {
        //                 return Some(Section{doc: String::from(""), code: s});
        //             },
        //             (Segment::Code(s), cd, &mut None) => {
        //                 let doc = std::mem::replace(cd ,None).unwrap();
        //                 return Some(Section{doc: doc, code: s});
        //             },
        //             (Segment::Code(_), &mut None, _cc) => {
        //                 error!("code and code should not happen.");
        //                 return None;
        //             },
        //             (_, _cd, _cc) => {
        //                 error!("previously failed to emit a section.");
        //                 return None;
        //             },
        //         }
        //     } else {
        //         match (&mut self.current_doc, &mut self.current_code) {
        //             (&mut None, &mut None) => { return None; },
        //             (mut doc, &mut None) => {
        //                 let d = std::mem::replace(doc, None);
        //                 return Some(Section{doc: d.unwrap(), code: "".to_string()});
        //             },
        //             (_, _) => {
        //                 error!("this should not happen...");
        //                 return None;
        //             }
        //         }
        //     }
        // }
    }
}
