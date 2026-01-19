use ix_core::entity::EntityKind;
use ix_core::markdown::{parse_markdown, render_markdown};
use ix_core::repo::IxchelRepo;
use serde_yaml::Value as YamlValue;
use tempfile::TempDir;

fn init_temp_git_repo() -> (TempDir, IxchelRepo) {
    let temp = TempDir::new().expect("create tempdir");
    std::fs::create_dir_all(temp.path().join(".git")).expect("create .git marker");

    let repo = IxchelRepo::init_from(temp.path(), false).expect("init ixchel repo");
    (temp, repo)
}

#[test]
fn open_from_requires_git_repo() {
    let temp = TempDir::new().expect("create tempdir");
    let err = IxchelRepo::open_from(temp.path()).expect_err("expected failure without .git");
    let msg = format!("{err:#}");
    assert!(msg.contains("Not inside a git repository"), "{msg}");
}

#[test]
fn init_writes_ixchel_config_and_gitignore_entries() {
    let (temp, _repo) = init_temp_git_repo();

    let config_path = temp.path().join(".ixchel/config.toml");
    assert!(config_path.exists(), "missing {config_path:?}");

    let gitignore_path = temp.path().join(".gitignore");
    let gitignore = std::fs::read_to_string(&gitignore_path).expect("read .gitignore");
    assert!(gitignore.contains(".ixchel/data/"));
    assert!(gitignore.contains(".ixchel/models/"));
}

#[test]
fn link_unlink_and_check_roundtrip() {
    let (_temp, repo) = init_temp_git_repo();

    let decision = repo
        .create_entity(EntityKind::Decision, "Decision A", Some("accepted"))
        .expect("create decision");
    let issue = repo
        .create_entity(EntityKind::Issue, "Issue A", Some("open"))
        .expect("create issue");

    repo.link(&issue.id, "implements", &decision.id)
        .expect("link issue implements decision");

    let report = repo.check().expect("check");
    assert!(
        report.errors.is_empty(),
        "expected no errors, got: {:#?}",
        report.errors
    );

    assert!(
        repo.unlink(&issue.id, "implements", &decision.id)
            .expect("unlink should succeed")
    );
    assert!(
        !repo
            .unlink(&issue.id, "implements", &decision.id)
            .expect("second unlink should no-op")
    );

    let report = repo.check().expect("check");
    assert!(
        report.errors.is_empty(),
        "expected no errors after unlink, got: {:#?}",
        report.errors
    );
}

#[test]
fn check_reports_broken_and_unknown_links() {
    let (_temp, repo) = init_temp_git_repo();
    let issue = repo
        .create_entity(EntityKind::Issue, "Issue A", Some("open"))
        .expect("create issue");

    let path = repo.paths.entity_path(&issue.id).expect("issue path");
    let raw = std::fs::read_to_string(&path).expect("read issue");
    let mut doc = parse_markdown(&path, &raw).expect("parse markdown");

    doc.frontmatter.insert(
        YamlValue::String("depends_on".to_string()),
        YamlValue::Sequence(vec![YamlValue::String("dec-deadbe".to_string())]),
    );
    doc.frontmatter.insert(
        YamlValue::String("mentions".to_string()),
        YamlValue::Sequence(vec![YamlValue::String("foo-123456".to_string())]),
    );

    let rendered = render_markdown(&doc).expect("render markdown");
    std::fs::write(&path, rendered).expect("write issue");

    let report = repo.check().expect("check");
    assert_eq!(report.errors.len(), 2, "{:#?}", report.errors);

    let messages = report
        .errors
        .into_iter()
        .map(|e| e.message)
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|m| m.contains("broken link depends_on -> dec-deadbe")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|m| m.contains("unknown id prefix in mentions: foo-123456")),
        "{messages:#?}"
    );
}
