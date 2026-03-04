use crate::types::TldrEntry;
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde::Deserialize;
use std::path::Path;

pub fn parse_tldr_file(path: &Path) -> Option<TldrEntry> {
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

/// Extracts a TldrEntry from the `tldr:` key in YAML frontmatter of a markdown file.
pub fn parse_frontmatter(path: &Path) -> Option<TldrEntry> {
    let content = std::fs::read_to_string(path).ok()?;

    #[derive(Deserialize)]
    struct Wrapper {
        tldr: TldrEntry,
    }

    let matter = Matter::<YAML>::new();
    let result = matter.parse::<Wrapper>(&content).ok()?;
    result.data.map(|w| w.tldr)
}
