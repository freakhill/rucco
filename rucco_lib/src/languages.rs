use compute_regex::compute_regex;

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
