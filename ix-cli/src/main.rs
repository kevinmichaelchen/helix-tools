use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use serde_yaml::Value as YamlValue;

#[derive(Parser, Debug)]
#[command(name = "ixchel", version)]
#[command(about = "Ixchel (ik-SHEL) — git-first knowledge weaving", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(long, global = true)]
    repo: Option<PathBuf>,
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = cli.repo.clone().unwrap_or(std::env::current_dir()?);

    match cli.command {
        Command::Init { force } => {
            let repo = ix_core::repo::IxchelRepo::init_from(&start, force)?;
            println!("Initialized {}", repo.paths.ixchel_dir().display());
        }
        Command::Create {
            kind,
            title,
            status,
        } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let created = repo.create_entity(kind, &title, status.as_deref())?;
            println!("Created {} ({})", created.id, created.path.display());
        }
        Command::Show { id } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let raw = repo.read_raw(&id)?;
            print!("{raw}");
        }
        Command::List { kind } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            for item in repo.list(kind)? {
                println!("{}\t{}\t{}", item.id, item.kind.as_str(), item.title);
            }
        }
        Command::Link { from, rel, to } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            repo.link(&from, &rel, &to)?;
            println!("Linked {from} -[{rel}]-> {to}");
        }
        Command::Unlink { from, rel, to } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let removed = repo.unlink(&from, &rel, &to)?;
            if removed {
                println!("Unlinked {from} -[{rel}]-> {to}");
            } else {
                println!("No link found: {from} -[{rel}]-> {to}");
            }
        }
        Command::Check => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let report = repo.check()?;
            if report.errors.is_empty() {
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
            println!(
                "Synced: scanned={} added={} modified={} deleted={} unchanged={}",
                stats.scanned, stats.added, stats.modified, stats.deleted, stats.unchanged
            );
        }
        Command::Search { query, limit } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            let index = ix_storage_helixdb::HelixDbIndex::open(&repo)?;
            let hits = ix_core::index::IndexBackend::search(&index, &query, limit)?;
            for hit in hits {
                let kind = hit.kind.map(|k| k.as_str()).unwrap_or("unknown");
                println!("{:.3}\t{}\t{}\t{}", hit.score, hit.id, kind, hit.title);
            }
        }
        Command::Graph { id } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            print_graph(&repo, &id)?;
        }
        Command::Context { id } => {
            let repo = ix_core::repo::IxchelRepo::open_from(&start)?;
            print_context(&repo, &id)?;
        }
    }

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
