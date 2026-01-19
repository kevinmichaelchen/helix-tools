use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use clap::Subcommand;
use serde_json::json;
use serde_yaml::Value as YamlValue;

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = cli.repo.clone().unwrap_or(std::env::current_dir()?);
    run(cli.command, &start, cli.json)
}

fn run(command: Command, start: &Path, json_output: bool) -> Result<()> {
    match command {
        Command::Init { force } => cmd_init(start, force, json_output),
        Command::Create {
            kind,
            title,
            status,
        } => cmd_create(start, kind, &title, status.as_deref(), json_output),
        Command::Show { id } => cmd_show(start, &id, json_output),
        Command::List { kind } => cmd_list(start, kind, json_output),
        Command::Link { from, rel, to } => cmd_link(start, &from, &rel, &to, json_output),
        Command::Unlink { from, rel, to } => cmd_unlink(start, &from, &rel, &to, json_output),
        Command::Check => cmd_check(start, json_output),
        Command::Sync => cmd_sync(start, json_output),
        Command::Search { query, limit } => cmd_search(start, &query, limit, json_output),
        Command::Graph { id } => cmd_graph(start, &id, json_output),
        Command::Context { id } => cmd_context(start, &id, json_output),
        Command::Delete { id } => cmd_delete(start, &id, json_output),
        Command::Edit { id } => cmd_edit(start, &id, json_output),
    }
}

fn cmd_init(start: &Path, force: bool, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::init_from(start, force)?;
    if json_output {
        print_json(&json!({ "ixchel_dir": repo.paths.ixchel_dir() }))?;
    } else {
        println!("Initialized {}", repo.paths.ixchel_dir().display());
    }
    Ok(())
}

fn cmd_create(
    start: &Path,
    kind: ix_core::entity::EntityKind,
    title: &str,
    status: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    let created = repo.create_entity(kind, title, status)?;
    if json_output {
        print_json(&json!({
            "id": created.id,
            "kind": created.kind.as_str(),
            "title": created.title,
            "path": created.path,
        }))?;
    } else {
        println!("Created {} ({})", created.id, created.path.display());
    }
    Ok(())
}

fn cmd_show(start: &Path, id: &str, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    let raw = repo.read_raw(id)?;
    if json_output {
        print_json(&json!({ "id": id, "raw": raw }))?;
    } else {
        print!("{raw}");
    }
    Ok(())
}

fn cmd_list(
    start: &Path,
    kind: Option<ix_core::entity::EntityKind>,
    json_output: bool,
) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
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
        print_json(&json!({ "items": items }))?;
    } else {
        for item in items {
            println!("{}\t{}\t{}", item.id, item.kind.as_str(), item.title);
        }
    }
    Ok(())
}

fn cmd_link(start: &Path, from: &str, rel: &str, to: &str, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    repo.link(from, rel, to)?;
    if json_output {
        print_json(&json!({ "from": from, "rel": rel, "to": to, "changed": true }))?;
    } else {
        println!("Linked {from} -[{rel}]-> {to}");
    }
    Ok(())
}

fn cmd_unlink(start: &Path, from: &str, rel: &str, to: &str, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    let removed = repo.unlink(from, rel, to)?;
    if json_output {
        print_json(&json!({ "from": from, "rel": rel, "to": to, "changed": removed }))?;
    } else if removed {
        println!("Unlinked {from} -[{rel}]-> {to}");
    } else {
        println!("No link found: {from} -[{rel}]-> {to}");
    }
    Ok(())
}

fn cmd_check(start: &Path, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    let report = repo.check()?;
    if json_output {
        let errors = report
            .errors
            .into_iter()
            .map(|e| json!({ "path": e.path, "message": e.message }))
            .collect::<Vec<_>>();
        print_json(&json!({ "ok": errors.is_empty(), "errors": errors }))?;
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
    Ok(())
}

fn cmd_sync(start: &Path, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    let stats = ix_app::sync(&repo)?;
    if json_output {
        print_json(&json!({
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
    Ok(())
}

fn cmd_search(start: &Path, query: &str, limit: usize, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    let hits = ix_app::search(&repo, query, limit)?;

    if json_output {
        let hits = hits
            .into_iter()
            .map(|h| {
                json!({
                    "score": h.score,
                    "id": h.id,
                    "kind": h.kind.map(ix_core::entity::EntityKind::as_str),
                    "title": h.title,
                })
            })
            .collect::<Vec<_>>();
        print_json(&json!({ "hits": hits }))?;
        return Ok(());
    }

    for hit in hits {
        let kind = hit
            .kind
            .map_or("unknown", ix_core::entity::EntityKind::as_str);
        println!("{:.3}\t{}\t{}\t{}", hit.score, hit.id, kind, hit.title);
    }

    Ok(())
}

fn cmd_graph(start: &Path, id: &str, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    if json_output {
        let graph = build_graph_json(&repo, id)?;
        print_json(&graph)?;
        return Ok(());
    }

    print_graph(&repo, id)
}

fn cmd_context(start: &Path, id: &str, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    if json_output {
        let context = build_context_json(&repo, id)?;
        print_json(&context)?;
        return Ok(());
    }

    print_context(&repo, id)
}

fn cmd_delete(start: &Path, id: &str, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    repo.delete_entity(id)?;
    if json_output {
        print_json(&json!({ "id": id, "deleted": true }))?;
    } else {
        println!("Deleted {id}");
    }
    Ok(())
}

fn cmd_edit(start: &Path, id: &str, json_output: bool) -> Result<()> {
    let repo = ix_core::repo::IxchelRepo::open_from(start)?;
    let path = repo
        .paths
        .entity_path(id)
        .ok_or_else(|| anyhow::anyhow!("Unknown entity id prefix: {id}"))?;

    if json_output {
        print_json(&json!({ "id": id, "path": path }))?;
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

    Ok(())
}

fn print_json(value: &serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
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

type GraphEdgeTarget = (String, Option<String>);
type GraphOutgoing = Vec<(String, Vec<GraphEdgeTarget>)>;
type CollectedGraph = (String, GraphOutgoing);

fn collect_graph(repo: &ix_core::repo::IxchelRepo, id: &str) -> Result<CollectedGraph> {
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

        let targets = targets
            .into_iter()
            .filter(|t| ix_core::entity::looks_like_entity_id(t))
            .collect::<Vec<_>>();

        if targets.is_empty() {
            continue;
        }

        rels.push((key.clone(), targets));
    }

    rels.sort_by(|a, b| a.0.cmp(&b.0));
    rels
}
