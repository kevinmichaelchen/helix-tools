use std::path::PathBuf;

use ix_core::entity::EntityKind;
use ix_core::index::IndexBackend;
use ix_core::markdown::{parse_markdown, render_markdown};
use ix_core::repo::IxchelRepo;
use tempfile::TempDir;

fn read_fixture(name: &str) -> String {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = crate_dir.join("tests/fixtures").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn replace_entity_body(repo: &IxchelRepo, id: &str, body: &str) {
    let path = repo.paths.entity_path(id).expect("entity path");
    let raw = std::fs::read_to_string(&path).expect("read entity");
    let mut doc = parse_markdown(&path, &raw).expect("parse markdown");
    doc.body = body.to_string();
    let out = render_markdown(&doc).expect("render markdown");
    std::fs::write(&path, out).expect("write entity");
}

#[test]
#[ignore = "E2E: downloads embedding model + builds HelixDB index"]
fn e2e_sync_then_search_returns_expected_source() {
    let temp = TempDir::new().expect("tempdir");
    let repo = IxchelRepo::init_at(temp.path(), false).expect("init ixchel repo");

    let primary = repo
        .create_entity(EntityKind::Source, "LMDB internals report", None)
        .expect("create source");
    replace_entity_body(
        &repo,
        &primary.id,
        &read_fixture("lmdb-internals-report.md"),
    );

    let secondary = repo
        .create_entity(EntityKind::Source, "HNSW tuning report", None)
        .expect("create source");
    replace_entity_body(
        &repo,
        &secondary.id,
        &read_fixture("hnsw-parameter-tuning-report.md"),
    );

    let mut index = ix_storage_helixdb::HelixDbIndex::open(&repo).expect("open index");
    let stats = index.sync(&repo).expect("sync");
    assert_eq!(stats.scanned, 2);
    assert_eq!(stats.added, 2);

    let hits = index.search("reader lock table", 5).expect("search");
    assert!(!hits.is_empty(), "expected search hits");
    assert_eq!(hits[0].id, primary.id, "{hits:#?}");
}
