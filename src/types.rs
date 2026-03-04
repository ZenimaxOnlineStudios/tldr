use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct TldrEntry {
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub docs: Vec<String>,
}
