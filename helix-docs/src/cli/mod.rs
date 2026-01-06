#![allow(clippy::needless_pass_by_value)] // CLI args will be consumed when implemented

use clap::{Parser, Subcommand};

use crate::error::Result;

mod add;
mod get;
mod ingest;
mod library;
mod search;
mod source;
mod status;

#[derive(Parser)]
#[command(name = "helix-docs")]
#[command(about = "Local documentation cache for AI-assisted development")]
#[command(version)]
pub struct Cli {
    #[arg(long, global = true, help = "Output as JSON")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Add a documentation source")]
    Add {
        #[arg(help = "URL of the documentation source (GitHub repo or website)")]
        url: String,

        #[arg(long, help = "Path to docs within repo (e.g., 'docs')")]
        docs: Option<String>,

        #[arg(long, help = "Git ref (branch, tag, or commit)")]
        r#ref: Option<String>,

        #[arg(long, help = "Version label for filtering")]
        version: Option<String>,
    },

    #[command(about = "Manage documentation sources")]
    Source {
        #[command(subcommand)]
        command: SourceCommands,
    },

    #[command(about = "Fetch and index documentation")]
    Ingest {
        #[arg(long, help = "Re-fetch all documents regardless of cache")]
        force: bool,

        #[arg(long, default_value = "5", help = "Number of concurrent fetches")]
        concurrency: usize,

        #[arg(long, help = "Generate embeddings for semantic search")]
        embed: bool,
    },

    #[command(about = "Search documentation")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(long, help = "Library to search (e.g., 'facebook/react')")]
        library: Option<String>,

        #[arg(long, help = "Version to filter by")]
        version: Option<String>,

        #[arg(
            long,
            default_value = "hybrid",
            help = "Search mode: hybrid, word, vector"
        )]
        mode: String,

        #[arg(long, default_value = "10", help = "Maximum results to return")]
        limit: usize,
    },

    #[command(about = "Find libraries by name")]
    Library {
        #[arg(help = "Library name pattern to search")]
        name: String,
    },

    #[command(about = "Get document content")]
    Get {
        #[arg(long, help = "Library containing the document")]
        library: Option<String>,

        #[arg(help = "Document path")]
        path: Option<String>,

        #[arg(long, help = "Document ID (alternative to path)")]
        doc: Option<String>,

        #[arg(long, help = "Line range to return (e.g., '10:50')")]
        slice: Option<String>,

        #[arg(long, help = "Output raw content without formatting")]
        raw: bool,
    },

    #[command(about = "Show cache status and statistics")]
    Status,

    #[command(about = "Detect project dependencies")]
    Detect,

    #[command(about = "Initialize helix-docs in current project")]
    Init {
        #[arg(long, help = "Overwrite existing configuration")]
        force: bool,
    },

    #[command(about = "Add sources from seed list")]
    Seed {
        #[arg(long, help = "Custom seed file path")]
        file: Option<String>,

        #[arg(long, help = "Auto-ingest after seeding")]
        ingest: bool,
    },

    #[command(about = "Remove stale documentation")]
    Cleanup {
        #[arg(long, help = "Remove docs not accessed in N days")]
        older_than: Option<u32>,

        #[arg(long, help = "Actually perform cleanup (default is dry-run)")]
        force: bool,
    },

    #[command(about = "Start MCP server")]
    Mcp,
}

#[derive(Subcommand)]
pub enum SourceCommands {
    #[command(about = "List all sources")]
    List,

    #[command(about = "Remove a source")]
    Remove {
        #[arg(help = "Source ID to remove")]
        id: String,

        #[arg(long, help = "Skip confirmation prompt")]
        force: bool,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Add {
            url,
            docs,
            r#ref,
            version,
        } => add::run(url, docs, r#ref, version, cli.json),
        Commands::Source { command } => source::run(command, cli.json),
        Commands::Ingest {
            force,
            concurrency,
            embed,
        } => ingest::run(force, concurrency, embed, cli.json),
        Commands::Search {
            query,
            library,
            version,
            mode,
            limit,
        } => search::run(query, library, version, mode, limit, cli.json),
        Commands::Library { name } => library::run(name, cli.json),
        Commands::Get {
            library,
            path,
            doc,
            slice,
            raw,
        } => get::run(library, path, doc, slice, raw, cli.json),
        Commands::Status => status::run(cli.json),
        Commands::Detect => todo!("detect command not yet implemented"),
        Commands::Init { force: _ } => todo!("init command not yet implemented"),
        Commands::Seed { file: _, ingest: _ } => todo!("seed command not yet implemented"),
        Commands::Cleanup {
            older_than: _,
            force: _,
        } => todo!("cleanup command not yet implemented"),
        Commands::Mcp => todo!("mcp command not yet implemented"),
    }
}
