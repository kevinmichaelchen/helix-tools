use ix_core::entity::EntityKind;
use ix_core::markdown::{parse_markdown, render_markdown, set_string};
use ix_core::repo::{IxchelRepo, ListSort};
use serde_yaml::Value as YamlValue;
use tempfile::TempDir;

fn init_temp_git_repo() -> (TempDir, IxchelRepo) {
    let temp = TempDir::new().expect("create tempdir");
    std::fs::create_dir_all(temp.path().join(".git")).expect("create .git marker");

    let repo = IxchelRepo::init_from(temp.path(), false).expect("init ixchel repo");
    (temp, repo)
}

fn set_entity_timestamps(
    repo: &IxchelRepo,
    id: &str,
    created: Option<&str>,
    updated: Option<&str>,
) {
    let path = repo.paths.entity_path(id).expect("entity path");
    let raw = std::fs::read_to_string(&path).expect("read entity");
    let mut doc = parse_markdown(&path, &raw).expect("parse markdown");

    match created {
        Some(value) => set_string(&mut doc.frontmatter, "created_at", value),
        None => {
            doc.frontmatter
                .remove(YamlValue::String("created_at".to_string()));
        }
    }

    match updated {
        Some(value) => set_string(&mut doc.frontmatter, "updated_at", value),
        None => {
            doc.frontmatter
                .remove(YamlValue::String("updated_at".to_string()));
        }
    }

    let rendered = render_markdown(&doc).expect("render markdown");
    std::fs::write(&path, rendered).expect("write entity");
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

#[test]
fn check_reports_frontmatter_parse_errors() {
    let (_temp, repo) = init_temp_git_repo();
    let dir = repo.paths.kind_dir(EntityKind::Issue);
    let path = dir.join("iss-bad001.md");
    let raw = "---\nfoo: [\n---\n";
    std::fs::write(&path, raw).expect("write invalid frontmatter");

    let report = repo.check_with_suggestions().expect("check");
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.message.contains("frontmatter YAML parse error")),
        "{:#?}",
        report.errors
    );
    assert!(
        report.errors.iter().any(|e| e.suggestion.is_some()),
        "{:#?}",
        report.errors
    );

    let report = repo.check().expect("check");
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.message.contains("frontmatter YAML parse error")),
        "{:#?}",
        report.errors
    );
}

#[test]
fn add_remove_tags_are_idempotent() {
    let (_temp, repo) = init_temp_git_repo();
    let issue = repo
        .create_entity(EntityKind::Issue, "Issue A", Some("open"))
        .expect("create issue");

    let changed = repo
        .add_tags(
            &issue.id,
            &["cli".to_string(), "cli".to_string(), " ".to_string()],
        )
        .expect("add tags");
    assert!(changed, "expected tag add to change document");

    let changed = repo
        .add_tags(&issue.id, &["cli".to_string()])
        .expect("add tags idempotent");
    assert!(!changed, "expected idempotent add to no-op");

    let path = repo.paths.entity_path(&issue.id).expect("issue path");
    let raw = std::fs::read_to_string(&path).expect("read issue");
    let doc = parse_markdown(&path, &raw).expect("parse markdown");
    assert_eq!(
        ix_core::markdown::get_string_list(&doc.frontmatter, "tags"),
        vec!["cli"]
    );

    let changed = repo
        .remove_tags(&issue.id, &["cli".to_string(), "missing".to_string()])
        .expect("remove tags");
    assert!(changed, "expected tag remove to change document");

    let changed = repo
        .remove_tags(&issue.id, &["cli".to_string()])
        .expect("remove tags idempotent");
    assert!(!changed, "expected idempotent remove to no-op");

    let raw = std::fs::read_to_string(&path).expect("read issue");
    let doc = parse_markdown(&path, &raw).expect("parse markdown");
    assert!(ix_core::markdown::get_string_list(&doc.frontmatter, "tags").is_empty());
}

#[test]
fn list_sorts_by_timestamps_and_puts_missing_last() {
    let (_temp, repo) = init_temp_git_repo();
    let issue_a = repo
        .create_entity(EntityKind::Issue, "Issue A", Some("open"))
        .expect("create issue a");
    let issue_b = repo
        .create_entity(EntityKind::Issue, "Issue B", Some("open"))
        .expect("create issue b");
    let issue_missing = repo
        .create_entity(EntityKind::Issue, "Issue Missing", Some("open"))
        .expect("create issue missing");

    set_entity_timestamps(
        &repo,
        &issue_a.id,
        Some("2024-01-01T00:00:00Z"),
        Some("2024-02-01T00:00:00Z"),
    );
    set_entity_timestamps(
        &repo,
        &issue_b.id,
        Some("2024-03-01T00:00:00Z"),
        Some("2024-01-15T00:00:00Z"),
    );
    set_entity_timestamps(&repo, &issue_missing.id, None, None);

    let created = repo
        .list(Some(EntityKind::Issue), ListSort::CreatedDesc)
        .expect("list created desc");
    let created_ids = created
        .iter()
        .map(|item| item.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        created_ids,
        vec![
            issue_b.id.as_str(),
            issue_a.id.as_str(),
            issue_missing.id.as_str()
        ]
    );

    let updated = repo
        .list(Some(EntityKind::Issue), ListSort::UpdatedDesc)
        .expect("list updated desc");
    let updated_ids = updated
        .iter()
        .map(|item| item.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        updated_ids,
        vec![
            issue_a.id.as_str(),
            issue_b.id.as_str(),
            issue_missing.id.as_str()
        ]
    );
}
