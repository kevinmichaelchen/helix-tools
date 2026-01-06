use crate::error::Result;

pub fn run(
    query: String,
    library: Option<String>,
    _version: Option<String>,
    mode: String,
    _limit: usize,
    _json: bool,
) -> Result<()> {
    todo!("search command: query={query}, library={library:?}, mode={mode}")
}
