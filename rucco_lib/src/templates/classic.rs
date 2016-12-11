use segment::RenderedSection;
use std::path::{Path,PathBuf};
use regex::Regex;
use maud::PreEscaped;

// #[cfg(test)]
// mod tests {
//     use section::Section;
//     use std::path::Path;
//     use super::*;

//     #[test]
//     fn render_is_usable() {
//         let docfiles = vec![PathBuf::from("/lol"), PathBuf::from("wut")];
//         let css_path = "csspafouuu";
//         let source_path = Path::from("sourcepafouuu");
//         let sections = vec![Section{
//             doc: "<h1>MAH TITLE</h1>docdocdoc".to_owned(),
//             code: "codecodecdoe".to_owned()
//         }];
//         let html = render(docfiles.iter(),css_path,&source_path,sections.iter());
//         println!("render result!: {}", html);
//     }
// }

pub fn render<'a,
              T: Iterator<Item=&'a RenderedSegment> + Clone + Sized,
              U: Iterator<Item=&'a PathBuf> + Clone + Sized>
    (docfiles: U,
     css_path: &'a str,
     source_path: &'a Path,
     segments: T)
     -> String
{
    let mut peek_segments = segments.clone().peekable();
    let (has_global_title, title_to_use) =
        if let Some(Rendered::Title((ref h, ref t))) = peek_segments.peek() {
            (true, t)
        } else {
            (false, source_path.to_str()
             .expect("failed to convert file path to string"))
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
                    @if !has_global_title {
                        li#title {
                            div.annotation {
                                (title_to_use)
                            }
                        }
                    }
                    @for (i, segment) in segments.enumerate() {
                        li id={ "segment-" (i) } {
                            @match segment {
                                RenderedSegment::Title((level,html)) => {
                                    div.annotation {
                                        div class={ "pilwrap for-" (level) } {
                                            a.pilcrow href={ "#segment-" (i) } { "¶" }
                                        }
                                        (PreEscaped(&html))
                                    }
                                    div.content {}
                                },
                                RenderedSegment::Code(doc) => {
                                    div.annotation {
                                        div class={ "pilwrap" } {
                                            a.pilcrow href={ "#segment-" (i) } { "¶" }
                                        }
                                    }
                                    (PreEscaped(&doc))
                                },
                                RenderedSegment::Code(code) => {
                                    div.content {
                                        (PreEscaped(&code))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    ].into_string()
}
