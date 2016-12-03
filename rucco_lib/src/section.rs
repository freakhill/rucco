#[derive(Debug, Clone)]
pub struct Section {
    pub doc: String,
    pub code: String,
}

pub struct Title {
    pub text: String,
    pub level: u8
}

pub enum TitleSplitSection {
    Title(Title),
    Section(Section)
}

pub type RenderedTitleSplitSection = TitleSplitSection;
