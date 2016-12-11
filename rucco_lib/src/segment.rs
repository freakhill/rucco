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
