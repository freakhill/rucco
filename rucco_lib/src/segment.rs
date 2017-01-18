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
                                -> //impl Iterator<Item=Segment>+'r+'t
DenseSegments<'r, 't>
{
    let sparse_segments: SparseSegments<'r, 't> =
        SparseSegments {
            code_and_doc_captures: r.captures_iter(source),
            title_and_doc_in_multiline_capture: None
        };

    let dense_segments: DenseSegments<'r, 't> =
        DenseSegments { segments: sparse_segments, cur: None };

    dense_segments
}

// -----------------------------------------------------------------------------
// ## Extracting segments

/// Iterator<Item=Option<Segment>>
struct SparseSegments<'r, 't> {
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

impl<'r, 't> Iterator for SparseSegments<'r, 't> {
    type Item=Option<Segment>;

    fn next(&mut self) -> Option<Self::Item> {

        let mut drop_current_multiline_capture = false;

        let segment =
            if let Some(ref mut bm) = self.title_and_doc_in_multiline_capture {
                // in multiline doc context
                if let Some(c) = bm.next() {
                    if let Some(l) = c.get(1) {
                        Some(Some(title_or_doc_segment(l.as_str())))
                    } else {
                        Some(None)
                    }
                } else {
                    drop_current_multiline_capture = true; // hope we can drop that when MIR is online
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
                            error!("Something went wrong when processing SparseSegments");
                            None
                        }
                    }
                } else {
                    None
                }
            };

        if drop_current_multiline_capture {
            self.title_and_doc_in_multiline_capture = None;
        }

        segment
    }
}

// -----------------------------------------------------------------------------
// ## Compacting segments

/// Iterator<Item=<Segment>>
pub struct DenseSegments<'r, 't> {
    segments: SparseSegments<'r, 't>,
    cur: Option<Segment>
}

impl<'r, 't> Iterator for DenseSegments<'r, 't> {
    type Item=Segment;

    fn next(&mut self) -> Option<Segment> {
        loop {
            match (&mut self.cur, self.segments.next()) {
                // we're done
                (cur, None) => return std::mem::replace(cur, None),
                // skip dud
                (_, Some(None)) => continue,
                // first one! (no self.cur)
                (cur @ &mut None, Some(n)) => {
                    std::mem::replace(cur, n);
                },
                // ---- ok we're left with Some(_),Some(Some(_))
                // same (=> append, except title, cannot append titles! they switch!)
                (&mut Some(Segment::Code(ref mut c)),Some(Some(Segment::Code(ref n)))) => {
                    c.push_str(n.as_str());
                },
                (&mut Some(Segment::Doc(ref mut c)),Some(Some(Segment::Doc(ref n)))) => {
                    c.push_str(n.as_str());
                },
                // different (=> switch)
                (cur, Some(n)) => {
                    let res = std::mem::replace(cur, n);
                    return res;
                },
            }
        }
    }
}
