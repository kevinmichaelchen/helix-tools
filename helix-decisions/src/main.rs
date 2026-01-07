use anyhow::Result;
use clap::{Parser, Subcommand};
use helix_daemon::Client as DaemonClient;
use helix_decisions::{
    ChainResponse, DecisionSearcher, RelatedResponse, SearchResponse, Status, hooks,
};
use helix_discovery::{DiscoveryError, find_git_root_from_cwd, find_marker_from_cwd};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "helix-decisions")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    directory: Option<PathBuf>,

    #[arg(short, long, global = true)]
    json: bool,

    #[arg(long, global = true, help = "Block until index is up to date")]
    sync: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Search {
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        tags: Option<String>,
    },
    Chain {
        decision_id: u32,
    },
    Related {
        decision_id: u32,
    },
    InitHooks {
        #[arg(long)]
        force: bool,
        #[arg(long, short = 'y')]
        yes: bool,
    },
    RemoveHooks,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::InitHooks { force, yes } => {
            return handle_init_hooks(force, yes);
        }
        Commands::RemoveHooks => {
            return handle_remove_hooks();
        }
        _ => {}
    }

    let directory = resolve_decisions_directory(cli.directory.clone())?;
    let git_root = find_git_root_from_cwd()?;

    let daemon_result = enqueue_daemon_sync(&git_root, &directory, cli.sync);

    let mut searcher = DecisionSearcher::new(&git_root)?;
    searcher.sync(&directory)?;

    if let Err(e) = &daemon_result
        && !cli.json
    {
        eprintln!("Warning: daemon sync failed ({e}), results may be stale");
    }

    match cli.command {
        Commands::InitHooks { .. } | Commands::RemoveHooks => unreachable!(),
        Commands::Search {
            query,
            limit,
            status,
            tags,
        } => {
            let status_filter = status
                .map(|s| s.parse::<Status>())
                .transpose()
                .map_err(|e| anyhow::anyhow!(e))?;
            let tags_filter = tags.map(|t| t.split(',').map(str::trim).map(String::from).collect());

            let response = searcher.search(&query, limit, status_filter, tags_filter)?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_search(&response);
            }

            if response.results.is_empty() {
                std::process::exit(1);
            }
        }
        Commands::Chain { decision_id } => {
            let response = searcher.get_chain(decision_id)?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_chain(&response);
            }

            if response.chain.is_empty() {
                std::process::exit(1);
            }
        }
        Commands::Related { decision_id } => {
            let response = searcher.get_related(decision_id)?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                print_related(&response);
            }

            if response.related.is_empty() {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn enqueue_daemon_sync(
    git_root: &std::path::Path,
    decisions_dir: &std::path::Path,
    wait: bool,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let client = DaemonClient::new();

        let repo_root = git_root.to_string_lossy().to_string();
        let directory = decisions_dir
            .strip_prefix(git_root)
            .unwrap_or(decisions_dir)
            .to_string_lossy()
            .to_string();

        let sync_result = client.sync(&repo_root, "decisions", &directory, wait).await;

        match sync_result {
            Ok(state) => {
                if wait && state != helix_daemon::SyncState::Done {
                    anyhow::bail!("Sync did not complete successfully: {state:?}");
                }
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Daemon error: {e}")),
        }
    })
}

fn handle_init_hooks(force: bool, yes: bool) -> Result<()> {
    let git_root = find_git_root_from_cwd().map_err(|_| {
        anyhow::anyhow!("Not in a git repository. init-hooks must be run from within a git repo.")
    })?;

    if !yes {
        hooks::print_hook_warning();
        if !hooks::confirm_installation()? {
            println!("Cancelled.");
            return Ok(());
        }
    }

    hooks::install_hook(&git_root, force)?;
    println!(
        "Installed pre-commit hook at {}/.git/hooks/pre-commit",
        git_root.display()
    );
    Ok(())
}

fn handle_remove_hooks() -> Result<()> {
    let git_root = find_git_root_from_cwd().map_err(|_| {
        anyhow::anyhow!("Not in a git repository. remove-hooks must be run from within a git repo.")
    })?;

    if hooks::uninstall_hook(&git_root)? {
        println!("Removed pre-commit hook.");
    } else {
        println!("No helix-decisions hook found.");
    }
    Ok(())
}

fn resolve_decisions_directory(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(dir) = explicit {
        return Ok(dir);
    }

    match find_marker_from_cwd(".decisions") {
        Ok(path) => Ok(path),
        Err(DiscoveryError::NotInGitRepo) => {
            anyhow::bail!(
                "Not in a git repository.\n\
                 helix-decisions expects a .decisions/ directory at your repo root.\n\
                 Run from a git repository or use --directory to specify the path."
            )
        }
        Err(DiscoveryError::MarkerNotFound { searched, .. }) => {
            anyhow::bail!(
                ".decisions/ directory not found at git root: {}\n\
                 Create it with: mkdir .decisions\n\
                 Or use --directory to specify a different path.",
                searched.display()
            )
        }
        Err(e) => anyhow::bail!("Failed to find decisions directory: {e}"),
    }
}

fn print_search(response: &SearchResponse) {
    if response.results.is_empty() {
        println!("No results found for: \"{}\"", response.query);
        return;
    }

    println!();
    println!("Query: \"{}\"", response.query);
    println!("Found: {} results", response.count);
    println!();

    for (i, result) in response.results.iter().enumerate() {
        println!("[{}] {:03}: {}", i + 1, result.id, result.title);
        println!("    Status: {} | Score: {:.2}", result.status, result.score);
        if !result.tags.is_empty() {
            println!("    Tags: {}", result.tags.join(", "));
        }
        println!(
            "    Date: {} | Deciders: {}",
            result.date,
            result.deciders.join(", ")
        );
        println!("    File: {}", result.file_path.display());
        println!();
    }
}

fn print_chain(response: &ChainResponse) {
    if response.chain.is_empty() {
        println!("No chain found for decision {}", response.root_id);
        return;
    }

    println!();
    println!("Supersedes chain from decision {}:", response.root_id);
    println!();

    for (i, node) in response.chain.iter().enumerate() {
        let prefix = if i == 0 { "└" } else { "  └" };
        let current = if node.is_current { " (current)" } else { "" };
        println!(
            "{prefix} {:03}: {} [{}]{current}",
            node.id, node.title, node.status
        );
    }
    println!();
}

fn print_related(response: &RelatedResponse) {
    if response.related.is_empty() {
        println!(
            "No related decisions found for decision {}",
            response.decision_id
        );
        return;
    }

    println!();
    println!("Related decisions for decision {}:", response.decision_id);
    println!();

    for rel in &response.related {
        println!("  {} {:03}: {}", rel.relation, rel.id, rel.title);
    }
    println!();
}
