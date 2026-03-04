use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct FoundFile {
    pub path: PathBuf,
    pub is_markdown: bool,
}

pub fn find_files(root: &Path, include_markdown: bool, max_depth: Option<usize>) -> Vec<FoundFile> {
    let walker = {
        let w = WalkDir::new(root);
        if let Some(depth) = max_depth { w.max_depth(depth) } else { w }
    };

    walker
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            let path = e.into_path();
            let name = path.file_name()?.to_str()?;
            if name == ".tldr" {
                Some(FoundFile { path, is_markdown: false })
            } else if include_markdown && name.ends_with(".md") {
                Some(FoundFile { path, is_markdown: true })
            } else {
                None
            }
        })
        .collect()
}
