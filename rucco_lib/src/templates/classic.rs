use section::TitleSplitSection;
use std::path::{Path,PathBuf};
use regex::Regex;
use maud::PreEscaped;

#[cfg(test)]
mod tests {
    use section::Section;
    use std::path::Path;
    use super::*;

    #[test]
    fn render_is_usable() {
        let docfiles = vec![PathBuf::from("/lol"), PathBuf::from("wut")];
        let css_path = "csspafouuu";
        let source_path = Path::from("sourcepafouuu");
        let sections = vec![Section{
            doc: "<h1>MAH TITLE</h1>docdocdoc".to_owned(),
            code: "codecodecdoe".to_owned()
        }];
        let html = render(docfiles.iter(),css_path,&source_path,sections.iter());
        println!("render result!: {}", html);
    }
}

lazy_static! {
    static ref HEADING_RE: Regex = Regex::new(r"(?i)\A[ \t]*<h(\d)+>(.*?)</h\d+>").unwrap();
}

fn heading(s: &Section) -> Option<(&str, &str)> {
    match HEADING_RE.captures(s.doc.as_str()) {
        Some(c) => {
            let heading_level = c.at(1).unwrap();
            let heading = c.at(2).unwrap();
            Some((heading_level, heading))
        },
        None => None
    }
}

pub fn render<'a,
              T: Iterator<Item=&'a TitleSplitSection> + Clone + Sized,
              U: Iterator<Item=&'a PathBuf> + Clone + Sized>
    (docfiles: U,
     css_path: &'a str,
     source_path: &'a Path,
     sections: T)
     -> String
{
    let mut peek_sections = sections.clone().peekable();
    if let Some(TitleSplitSection::Title(ref t)) = peek_sections.peek() {
        let (has_title_in_first_section, title_to_use) = {
            if let Some((_, title)) = heading(t.text) {
                (true, title)
            } else {
                (false, source_path.to_str()
                 .expect("failed to convert file path to string"))
            }
        };

        let docfiles_count = docfiles.clone().count();

        html! [
            head {
                title { (title_to_use) }
                meta http-equiv="content-type" content="text/html; charset=UTF-8" {}
                meta name="viewport" content="width=device-width, target-densitydpi=160dpi, initial-scale=1.0, maximum-scale=1.0, user-scalable=0" {}
                link rel="stylesheet" media="all" href=(css_path) {}
            }
            body {
                div#container {
                    div#background {}
                    @if docfiles_count > 1 {
                        ul#jump_to {
                            li {
                                a.large href="javascript:void(0)" { "Jump To …" }
                                a.small href="javascript:void(0)" { "+" }
                                div#jump_wrapper {
                                    div#jump_page {
                                        @for docfile in docfiles {
                                            a.source href=(docfile.to_str().unwrap()) {
                                                (docfile.file_name().unwrap().to_str().unwrap())
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ul.sections {
                        @if !has_title_in_first_section {
                            li#title {
                                div.annotation {
                                    h1 { (title_to_use) }
                                }
                            }
                        }
                        @for (i, section) in sections.enumerate() {
                            li id={ "section-" (i) } {
                                @match section {
                                    TitleSplitSection::Title(title) => {
                                        div.annotation {
                                            div class={ "pilwrap for-" (t.level) } {
                                                a.pilcrow href={ "#section-" (i) } { "¶" }
                                            }
                                            (PreEscaped(&title.text))
                                        }
                                        div.content {}
                                    },
                                    TitleSplitSection::Section(section) => {
                                        div.annotation {
                                            div class={ "pilwrap" } {
                                                a.pilcrow href={ "#section-" (i) } { "¶" }
                                            }
                                            (PreEscaped(&section.doc))
                                        }
                                        div.content {
                                            (PreEscaped(&section.code))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        ].into_string()
    } else {
        // there are no sections...
        String::from("nothing to see here...")
    }
}
