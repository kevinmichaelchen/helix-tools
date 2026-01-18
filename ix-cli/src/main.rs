use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use clap::Subcommand;
use serde_json::json;
use serde_yaml::Value as YamlValue;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "ixchel", version)]
#[command(about = "Ixchel (ik-SHEL) — git-first knowledge weaving", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(long, global = true)]
    repo: Option<PathBuf>,

    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init {
        #[arg(long)]
        force: bool,
    },

    Create {
        kind: ix_core::entity::EntityKind,
        title: String,
        #[arg(long)]
        status: Option<String>,
    },

    Show {
        id: String,
    },

    List {
        kind: Option<ix_core::entity::EntityKind>,
    },

    Link {
        from: String,
        rel: String,
        to: String,
    },

    Unlink {
        from: String,
        rel: String,
        to: String,
    },

    Check,

    Sync,

    Search {
        query: String,
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },

    Graph {
        id: String,
    },

    Context {
        id: String,
    },

    Delete {
        id: String,
    },

    Edit {
        id: String,
    },

    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
    },
}

#[derive(Subcommand, Debug)]
enum MigrateCommand {
    Decisions {
        #[arg(long)]
        source: Option<PathBuf>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = cli.repo.clone().unwrap_or(std::env::current_dir()?);
    let json_output = cli.json;

    match cli.command {
        Command::Init { force } => {
            let repo = ix_core::repo::IxchelRepo::init_from(&start, force)?;
            if json_output {
                print_json(json!({ "ixchel_dir": repo.paths.ixchel_dir() }))?;
            } else {
                println!("Initialized {}", repo.paths.ixchel_dir().display());
            }
        }
        Command::Create {
            kind,
            title,
            status,
        } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let created = repo.create_entity(kind, &title, status.as_deref())?;
            if json_output {
                print_json(json!({
                    "id": created.id,
                    "kind": created.kind.as_str(),
                    "title": created.title,
                    "path": created.path,
                }))?;
            } else {
                println!("Created {} ({})", created.id, created.path.display());
            }
        }
        Command::Show { id } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let raw = repo.read_raw(&id)?;
            if json_output {
                print_json(json!({ "id": id, "raw": raw }))?;
            } else {
                print!("{raw}");
            }
        }
        Command::List { kind } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let items = repo.list(kind)?;
            if json_output {
                let items = items
                    .into_iter()
                    .map(|i| {
                        json!({
                            "id": i.id,
                            "kind": i.kind.as_str(),
                            "title": i.title,
                            "path": i.path,
                        })
                    })
                    .collect::<Vec<_>>();
                print_json(json!({ "items": items }))?;
            } else {
                for item in items {
                    println!("{}\t{}\t{}", item.id, item.kind.as_str(), item.title);
                }
            }
        }
        Command::Link { from, rel, to } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            repo.link(&from, &rel, &to)?;
            if json_output {
                print_json(json!({ "from": from, "rel": rel, "to": to, "changed": true }))?;
            } else {
                println!("Linked {from} -[{rel}]-> {to}");
            }
        }
        Command::Unlink { from, rel, to } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let removed = repo.unlink(&from, &rel, &to)?;
            if json_output {
                print_json(json!({ "from": from, "rel": rel, "to": to, "changed": removed }))?;
            } else if removed {
                println!("Unlinked {from} -[{rel}]-> {to}");
            } else {
                println!("No link found: {from} -[{rel}]-> {to}");
            }
        }
        Command::Check => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let report = repo.check()?;
            if json_output {
                let errors = report
                    .errors
                    .into_iter()
                    .map(|e| json!({ "path": e.path, "message": e.message }))
                    .collect::<Vec<_>>();
                print_json(json!({ "ok": errors.is_empty(), "errors": errors }))?;
                if !errors.is_empty() {
                    std::process::exit(1);
                }
            } else if report.errors.is_empty() {
                println!("OK");
            } else {
                for error in &report.errors {
                    eprintln!("{}: {}", error.path.display(), error.message);
                }
                std::process::exit(1);
            }
        }
        Command::Sync => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let mut index = ix_storage_helixdb::HelixDbIndex::open(&repo)?;
            let stats = ix_core::index::IndexBackend::sync(&mut index, &repo)?;
            if json_output {
                print_json(json!({
                    "scanned": stats.scanned,
                    "added": stats.added,
                    "modified": stats.modified,
                    "deleted": stats.deleted,
                    "unchanged": stats.unchanged,
                }))?;
            } else {
                println!(
                    "Synced: scanned={} added={} modified={} deleted={} unchanged={}",
                    stats.scanned, stats.added, stats.modified, stats.deleted, stats.unchanged
                );
            }
        }
        Command::Search { query, limit } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let index = ix_storage_helixdb::HelixDbIndex::open(&repo)?;
            let hits = ix_core::index::IndexBackend::search(&index, &query, limit)?;
            if json_output {
                let hits = hits
                    .into_iter()
                    .map(|h| {
                        json!({
                            "score": h.score,
                            "id": h.id,
                            "kind": h.kind.map(|k| k.as_str()),
                            "title": h.title,
                        })
                    })
                    .collect::<Vec<_>>();
                print_json(json!({ "hits": hits }))?;
            } else {
                for hit in hits {
                    let kind = hit.kind.map(|k| k.as_str()).unwrap_or("unknown");
                    println!("{:.3}\t{}\t{}\t{}", hit.score, hit.id, kind, hit.title);
                }
            }
        }
        Command::Graph { id } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            if json_output {
                print_json(build_graph_json(&repo, &id)?)?;
            } else {
                print_graph(&repo, &id)?;
            }
        }
        Command::Context { id } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            if json_output {
                print_json(build_context_json(&repo, &id)?)?;
            } else {
                print_context(&repo, &id)?;
            }
        }
        Command::Delete { id } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            repo.delete_entity(&id)?;
            if json_output {
                print_json(json!({ "id": id, "deleted": true }))?;
            } else {
                println!("Deleted {id}");
            }
        }
        Command::Edit { id } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let path = repo
                .paths
                .entity_path(&id)
                .ok_or_else(|| anyhow::anyhow!("Unknown entity id prefix: {id}"))?;

            if json_output {
                print_json(json!({ "id": id, "path": path }))?;
                return Ok(());
            }

            let editor = std::env::var("IXCHEL_EDITOR")
                .ok()
                .or_else(|| std::env::var("EDITOR").ok())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "vi".to_string());

            let status = std::process::Command::new(editor)
                .arg(&path)
                .status()
                .with_context(|| format!("Failed to launch editor for {}", path.display()))?;

            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }

        Command::Migrate { command } => match command {
            MigrateCommand::Decisions {
                source,
                force,
                dry_run,
            } => {
                let repo = open_or_init(&start, false)?;
                let options = ix_core::migrate::MigrateDecisionsOptions {
                    source_dir: source.unwrap_or_else(|| PathBuf::from(".decisions")),
                    force,
                    dry_run,
                };
                let report = ix_core::migrate::migrate_decisions(&repo, &options)?;

                if json_output {
                    print_json(json!({
                        "scanned": report.scanned,
                        "created": report.created,
                        "skipped": report.skipped,
                        "dry_run": dry_run,
                    }))?;
                } else if dry_run {
                    println!(
                        "Dry run: scanned={} would_create={} skipped={}",
                        report.scanned, report.created, report.skipped
                    );
                } else {
                    println!(
                        "Migrated: scanned={} created={} skipped={}",
                        report.scanned, report.created, report.skipped
                    );
                }
            }
        },
    }

    Ok(())
}

fn print_json(value: serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn open_or_init(start: &Path, force: bool) -> Result<ix_core::repo::IxchelRepo> {
    let Some(repo_root) = ix_core::paths::find_git_root(start) else {
        anyhow::bail!(
            "Not inside a git repository (no .git found above {})",
            start.display()
        );
    };

    let paths = ix_core::paths::IxchelPaths::new(repo_root.clone());
    if paths.ixchel_dir().exists() {
        ix_core::repo::IxchelRepo::open_from(&repo_root)
    } else {
        ix_core::repo::IxchelRepo::init_at(&repo_root, force)
    }
}

const METADATA_KEYS: &[&str] = &[
    "id",
    "type",
    "title",
    "status",
    "date",
    "created_at",
    "updated_at",
    "created_by",
    "tags",
];

fn print_graph(repo: &ix_core::repo::IxchelRepo, id: &str) -> Result<()> {
    let path = repo
        .paths
        .entity_path(id)
        .ok_or_else(|| anyhow::anyhow!("Unknown entity id prefix: {id}"))?;
    let raw = std::fs::read_to_string(&path)?;
    let doc = ix_core::markdown::parse_markdown(&path, &raw)?;

    let title = ix_core::markdown::get_string(&doc.frontmatter, "title").unwrap_or_default();
    println!("{id}: {title}");

    for (rel, targets) in extract_relationships(&doc.frontmatter) {
        println!("{rel}:");
        for target in targets {
            let target_title = repo
                .paths
                .entity_path(&target)
                .and_then(|p| std::fs::read_to_string(&p).ok().map(|raw| (p, raw)))
                .and_then(|(p, raw)| ix_core::markdown::parse_markdown(&p, &raw).ok())
                .and_then(|d| ix_core::markdown::get_string(&d.frontmatter, "title"))
                .unwrap_or_default();

            if target_title.is_empty() {
                println!("  - {target}");
            } else {
                println!("  - {target}: {target_title}");
            }
        }
    }

    Ok(())
}

fn print_context(repo: &ix_core::repo::IxchelRepo, id: &str) -> Result<()> {
    let path = repo
        .paths
        .entity_path(id)
        .ok_or_else(|| anyhow::anyhow!("Unknown entity id prefix: {id}"))?;
    let raw = std::fs::read_to_string(&path)?;
    let doc = ix_core::markdown::parse_markdown(&path, &raw)?;

    let mut ids = vec![id.to_string()];
    for (_, targets) in extract_relationships(&doc.frontmatter) {
        ids.extend(targets);
    }

    ids.sort();
    ids.dedup();

    for entity_id in ids {
        let path = repo
            .paths
            .entity_path(&entity_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown entity id prefix: {entity_id}"))?;
        let raw = std::fs::read_to_string(&path)?;
        let doc = ix_core::markdown::parse_markdown(&path, &raw)?;

        let title = ix_core::markdown::get_string(&doc.frontmatter, "title").unwrap_or_default();

        println!("---");
        println!("{entity_id}: {title}");
        println!();
        print!("{}", doc.body);
        if !doc.body.ends_with('\n') {
            println!();
        }
    }

    Ok(())
}

fn build_graph_json(repo: &ix_core::repo::IxchelRepo, id: &str) -> Result<serde_json::Value> {
    let (root_title, outgoing) = collect_graph(repo, id)?;
    Ok(json!({
        "id": id,
        "title": root_title,
        "outgoing": outgoing.into_iter().map(|(rel, targets)| {
            json!({
                "rel": rel,
                "targets": targets.into_iter().map(|(id, title)| json!({ "id": id, "title": title })).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>()
    }))
}

fn build_context_json(repo: &ix_core::repo::IxchelRepo, id: &str) -> Result<serde_json::Value> {
    let items = collect_context(repo, id)?;
    Ok(json!({
        "id": id,
        "items": items.into_iter().map(|(id, title, body)| json!({ "id": id, "title": title, "body": body })).collect::<Vec<_>>(),
    }))
}

fn collect_graph(
    repo: &ix_core::repo::IxchelRepo,
    id: &str,
) -> Result<(String, Vec<(String, Vec<(String, Option<String>)>)>)> {
    let path = repo
        .paths
        .entity_path(id)
        .ok_or_else(|| anyhow::anyhow!("Unknown entity id prefix: {id}"))?;
    let raw = std::fs::read_to_string(&path)?;
    let doc = ix_core::markdown::parse_markdown(&path, &raw)?;

    let title = ix_core::markdown::get_string(&doc.frontmatter, "title").unwrap_or_default();
    let mut outgoing = Vec::new();

    for (rel, targets) in extract_relationships(&doc.frontmatter) {
        let mut items = Vec::new();
        for target in targets {
            let target_title = repo
                .paths
                .entity_path(&target)
                .and_then(|p| std::fs::read_to_string(&p).ok().map(|raw| (p, raw)))
                .and_then(|(p, raw)| ix_core::markdown::parse_markdown(&p, &raw).ok())
                .and_then(|d| ix_core::markdown::get_string(&d.frontmatter, "title"));
            items.push((target, target_title));
        }
        outgoing.push((rel, items));
    }

    Ok((title, outgoing))
}

fn collect_context(
    repo: &ix_core::repo::IxchelRepo,
    id: &str,
) -> Result<Vec<(String, String, String)>> {
    let path = repo
        .paths
        .entity_path(id)
        .ok_or_else(|| anyhow::anyhow!("Unknown entity id prefix: {id}"))?;
    let raw = std::fs::read_to_string(&path)?;
    let doc = ix_core::markdown::parse_markdown(&path, &raw)?;

    let mut ids = vec![id.to_string()];
    for (_, targets) in extract_relationships(&doc.frontmatter) {
        ids.extend(targets);
    }

    ids.sort();
    ids.dedup();

    let mut out = Vec::new();
    for entity_id in ids {
        let path = repo
            .paths
            .entity_path(&entity_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown entity id prefix: {entity_id}"))?;
        let raw = std::fs::read_to_string(&path)?;
        let doc = ix_core::markdown::parse_markdown(&path, &raw)?;
        let title = ix_core::markdown::get_string(&doc.frontmatter, "title").unwrap_or_default();
        out.push((entity_id, title, doc.body));
    }

    Ok(out)
}

fn extract_relationships(frontmatter: &serde_yaml::Mapping) -> Vec<(String, Vec<String>)> {
    let mut rels = Vec::new();

    for (key, value) in frontmatter {
        let YamlValue::String(key) = key else {
            continue;
        };

        if METADATA_KEYS.contains(&key.as_str()) {
            continue;
        }

        let targets = match value {
            YamlValue::Sequence(seq) => seq
                .iter()
                .filter_map(|v| match v {
                    YamlValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            YamlValue::String(s) => vec![s.clone()],
            _ => Vec::new(),
        };

        if targets.is_empty() {
            continue;
        }

        rels.push((key.clone(), targets));
    }

    rels.sort_by(|a, b| a.0.cmp(&b.0));
    rels
}
