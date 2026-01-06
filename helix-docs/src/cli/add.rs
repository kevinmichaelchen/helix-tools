use crate::domain::SourceConfig;
use crate::error::Result;

pub fn run(
    url: String,
    docs: Option<String>,
    git_ref: Option<String>,
    version: Option<String>,
    _json: bool,
) -> Result<()> {
    let config = SourceConfig {
        docs_path: docs,
        git_ref,
        version,
        ..Default::default()
    };

    todo!("add command: create source with config {config:?} for URL {url}")
}
