use std::collections::BTreeMap;
use regex::{Regex,RegexBuilder};
use toml;

// figure out Arc, Mutex etc. afterwards
pub struct Languages {
    computed: BTreeMap<String, Option<Regex>>,
    raw: toml::value::Table
}

impl Languages {
    pub fn new(raw: toml::value::Table) -> Languages {
        Languages {computed: BTreeMap::new(), raw: raw}
    }

    pub fn get(&mut self, l: &str) -> &Option<Regex> {
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
    regexp_vec.push(r"(?:".to_string()); // global group
    regexp_vec.push(r"(?:\n+)|".to_string()); // empty lines
    if let Some(sl) = singleline_mark {
        // singleline
        regexp_vec.push([r"(?:",
                         r"^[ \t]*", sl, r" ?",
                         r"(?:(?P<doc_sl>[^\n]*\n?)\n*)",
                         r")|"].concat());
    };
    if let (Some(mh), Some(mf), Some(mm)) = (multiline_header_mark, multiline_footer_mark, multiline_margin_mark) {
        // multiline with margin
        regexp_vec.push([r"(?:",
                         r"^[ \t]*", mh, r"(?P<doc_ml_h>[^\n]*\n?)\n*", // header and potential doc there
                         r"(?P<doc_ml_l>(?:[ \t]*", mm, r"[^\n]*\n*)*[ \t]*)", mf, // lines
                         r")|"].concat());
        // this is far from foolproof
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
    debug!("building regexp from: {}", &final_regexp);

    match RegexBuilder::new(&final_regexp)
        .multi_line(true)
        .dot_matches_new_line(true)
        .build() {
            Ok(regexp) => Some(regexp),
            Err(e) => {
                error!("Failed to build regex from language {:?}: {}", language, e);
                None
            }
        }
}
