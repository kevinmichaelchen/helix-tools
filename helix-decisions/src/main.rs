//! helix-decisions CLI - Decision graph infrastructure with semantic search.

use anyhow::Result;
use clap::{Parser, Subcommand};
use helix_decisions::{ChainResponse, DecisionSearcher, RelatedResponse, SearchResponse, Status};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "helix-decisions")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = ".decisions", global = true)]
    directory: PathBuf,

    #[arg(short, long, global = true)]
    json: bool,
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut searcher = DecisionSearcher::new()?;
    searcher.sync(&cli.directory)?;

    match cli.command {
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
            "{} {:03}: {} [{}]{}",
            prefix, node.id, node.title, node.status, current
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
