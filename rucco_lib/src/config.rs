use toml;

/// This will hold our final configuration (after merging clap data and ruccofile data).
pub struct Config<'a> {
    pub recursive: bool,
    pub entries: Vec<&'a str>,
    pub output_dir: &'a str,
    pub languages: &'a toml::Table
}
