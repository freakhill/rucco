use segment::*;
use languages::Languages;

use syntect::parsing::SyntaxSet;
use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::html::highlighted_snippet_for_string;

use hoedown::{Markdown,Html,Render};
use hoedown::renderer::html;

use std::path::{Path};

use templates;

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
            let title_html = md_html.render(&md_doc).to_str().unwrap_or("<p>failed to render title</p>").to_owned();
            Segment::Title((h, title_html))
        },
        Segment::Doc(doc) => {
            let md_doc = Markdown::new(doc.as_str());
            let doc_html = md_html.render(&md_doc).to_str().unwrap_or("<p>failed to render doc</p>").to_owned();
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
            if let &Some(ref lang) = languages.get(extension) {
                let sections: Vec<Segment> =
                    extract_segments(lang, source_text)
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
