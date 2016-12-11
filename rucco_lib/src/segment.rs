#[derive(Debug,Clone)]
pub enum Segment {
    Title((u8, String)), // "## lol" -> (2, "## lol")
    Code(String),
    Doc(String)
}

pub type RenderedSegment = Segment;

impl Segment {
    // extend a segment with new data and return Empty
    // OR
    // replace a segment with new data and return the old segment
    fn push(&mut self, s: Segment) => Option<Segment> {
        match (self, s) {
            (_, Segment::Empty) => Segment::Empty,
            (Segment::Empty, _) => {
                // replace stuff
                // ...
                None
            },
            (mut ref Segment::Title(_), Segment::Title(_)) => {
                error!("two title segments with nothing in between!??");
                None
            },
            (mut ref Segment::Code(c), Segment::Code(cc)) => {
            },
            (mut ref Segment::Doc(d), Segment::Doc(dd)) => {
            },
            (mut ref a, b) => {
                // replace b by a and return a
            }
        }
    }
}

/// iterator over Option<ExtractSegments>
struct ExtractSegments<'r, 't> {
    /// our regex captures that split doc from code
    fc: regex::FindCaptures<'r, 't>,
    /// necessary for splitting multilines into titles and doc lines
    multiline_doc_fc: Option<regex::FindCaptures<'r, 't>>
}

/// iterator over segments
pub struct ExtractCompactSegments<'r, 't> {
    segments: ExtractSegments<'r, 't>
}

pub fn extract_segments<'r, 't>(r: &'r regex::Regex, source: &'t str)
                                -> ExtractCompactSegments<'r, 't> {
    ExtractCompactSegments {
        segments: ExtractSegments {
            fc: r.captures_iter(source),
            multiline_doc_fc: None
        }
    }
}

impl<'r, 't> Iterator for ExtractCompactSegments<'r, 't> {
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


impl<'r, 't> Iterator for ExtractSegments<'r, 't> {
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
