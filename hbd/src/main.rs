//! hbd - Git-first issue tracker powered by HelixDB
//!
//! A distributed issue tracker designed for AI-supervised coding workflows.
//! Issues are stored as Markdown files in `.tickets/` and synced to HelixDB
//! for fast graph traversal, vector search, and BM25 text search.
//!
//! See specs/ for full documentation.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hbd")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Output format
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize hbd in the current directory
    Init,

    /// Show system info and status
    Info,

    /// Create a new issue
    Create {
        /// Issue title
        title: String,

        /// Issue description
        #[arg(short, long)]
        description: Option<String>,

        /// Issue type: bug, feature, task, epic, chore, gate
        #[arg(short = 't', long, default_value = "task")]
        r#type: String,

        /// Priority: 0 (critical) to 4 (backlog)
        #[arg(short, long, default_value = "2")]
        priority: i32,

        /// Labels (comma-separated)
        #[arg(short, long)]
        labels: Option<String>,

        /// Assignee
        #[arg(short, long)]
        assignee: Option<String>,

        /// Agent ID (for AI agents)
        #[arg(long)]
        agent: Option<String>,

        /// Session ID (for AI agents)
        #[arg(long)]
        session: Option<String>,

        /// Create as ephemeral (not synced to git)
        #[arg(long)]
        ephemeral: bool,
    },

    /// Show issue details
    Show {
        /// Issue ID
        id: String,
    },

    /// List issues
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by type
        #[arg(short = 't', long)]
        r#type: Option<String>,

        /// Filter by priority
        #[arg(short, long)]
        priority: Option<i32>,

        /// Filter by label
        #[arg(short, long)]
        label: Option<String>,

        /// Filter by assignee
        #[arg(short, long)]
        assignee: Option<String>,

        /// Filter by project
        #[arg(long)]
        project: Option<String>,

        /// Include ephemeral issues
        #[arg(long)]
        include_ephemeral: bool,
    },

    /// Update an issue
    Update {
        /// Issue ID
        id: String,

        /// New status
        #[arg(long)]
        status: Option<String>,

        /// New priority
        #[arg(long)]
        priority: Option<i32>,

        /// New title
        #[arg(long)]
        title: Option<String>,

        /// New assignee
        #[arg(long)]
        assignee: Option<String>,
    },

    /// Close an issue
    Close {
        /// Issue ID
        id: String,

        /// Reason for closing
        #[arg(short, long)]
        reason: Option<String>,

        /// Force close even with open children
        #[arg(long)]
        force: bool,
    },

    /// Reopen a closed issue
    Reopen {
        /// Issue ID
        id: String,
    },

    /// Search issues
    Search {
        /// Search query
        query: String,

        /// Use semantic (vector) search
        #[arg(long)]
        semantic: bool,

        /// Use hybrid search (BM25 + vector)
        #[arg(long)]
        hybrid: bool,

        /// Maximum results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Find similar issues
    Similar {
        /// Issue ID
        id: String,

        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Dependency management
    Dep {
        #[command(subcommand)]
        command: DepCommands,
    },

    /// Label management
    Label {
        #[command(subcommand)]
        command: LabelCommands,
    },

    /// Add a comment to an issue
    Comment {
        /// Issue ID
        id: String,

        /// Comment message
        message: String,
    },

    /// List comments on an issue
    Comments {
        /// Issue ID
        id: String,
    },

    /// Show ready (unblocked) issues
    Ready {
        /// Filter by project
        #[arg(long)]
        project: Option<String>,
    },

    /// Show blocked issues
    Blocked {
        /// Filter by project
        #[arg(long)]
        project: Option<String>,
    },

    /// Explain blockers for an issue
    Explain {
        /// Issue ID
        id: String,
    },

    /// Find critical path to an epic
    CriticalPath {
        /// Epic issue ID
        id: String,
    },

    /// Generate dependency graph visualization
    Graph {
        /// Issue ID
        id: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Max traversal depth
        #[arg(long, default_value = "5")]
        depth: usize,
    },

    /// Find stale issues not updated recently
    Stale {
        /// Days threshold (default: 14)
        #[arg(long, default_value = "14")]
        days: u32,

        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,

        /// Maximum results
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Count issues with optional filters
    Count {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by type
        #[arg(short = 't', long)]
        r#type: Option<String>,
    },

    /// Merge duplicate issues into one
    Merge {
        /// Source issue IDs to merge
        sources: Vec<String>,

        /// Target issue ID to merge into
        #[arg(long)]
        into: String,

        /// Preview without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Restore pre-compaction content from git history
    Restore {
        /// Issue ID
        id: String,
    },

    /// Administrative commands
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },

    /// Sync between git and HelixDB
    Sync {
        /// Only import from files
        #[arg(long)]
        import_only: bool,

        /// Only export to files
        #[arg(long)]
        export_only: bool,
    },

    /// Project health metrics
    Health {
        /// Show health for a specific label
        #[arg(long)]
        label: Option<String>,
    },

    /// Issue statistics
    Stats {
        /// Filter by project
        #[arg(long)]
        project: Option<String>,
    },

    /// Get context for AI agents
    Context {
        /// Search query for relevant issues
        #[arg(long)]
        query: Option<String>,

        /// Maximum tokens in output
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Compact old closed issues
    Compact {
        /// Show what would be compacted without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Daemon management
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
}

#[derive(Subcommand)]
enum DepCommands {
    /// Add a dependency
    Add {
        /// Source issue ID
        from: String,

        /// Dependency type: blocks, related, waits_for, duplicate_of
        dep_type: String,

        /// Target issue ID
        to: String,
    },

    /// Remove a dependency
    Remove {
        /// Source issue ID
        from: String,

        /// Dependency type
        dep_type: String,

        /// Target issue ID
        to: String,
    },

    /// List dependencies for an issue
    List {
        /// Issue ID
        id: String,
    },

    /// Find all cycles in the dependency graph
    Cycles,
}

#[derive(Subcommand)]
enum LabelCommands {
    /// Add a label to an issue
    Add {
        /// Issue ID
        id: String,

        /// Label name (comma-separated for multiple)
        label: String,
    },

    /// Remove a label from an issue
    Remove {
        /// Issue ID
        id: String,

        /// Label name
        label: String,
    },

    /// List labels on an issue
    List {
        /// Issue ID
        id: String,
    },

    /// List all labels in the project
    ListAll,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },
}

#[derive(Subcommand)]
enum AdminCommands {
    /// Delete old closed issues
    Cleanup {
        /// Only delete issues closed more than N days ago
        #[arg(long, default_value = "90")]
        older_than: u32,

        /// Show what would be deleted without making changes
        #[arg(long)]
        dry_run: bool,

        /// Actually perform deletion (required for safety)
        #[arg(long)]
        force: bool,

        /// Also delete orphaned dependencies and comments
        #[arg(long)]
        cascade: bool,
    },
}

#[derive(Subcommand)]
enum DaemonCommands {
    /// Start the daemon
    Start,

    /// Stop the daemon
    Stop,

    /// Check daemon status
    Status,
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    match cli.command {
        Commands::Init => {
            println!("TODO: Initialize hbd in current directory");
            println!("See specs/tasks.md for implementation details");
        }
        Commands::Info => {
            println!("TODO: Show system info (db path, daemon status, issue count)");
        }
        Commands::Create { title, .. } => {
            println!("TODO: Create issue: {}", title);
        }
        Commands::Show { id } => {
            println!("TODO: Show issue: {}", id);
        }
        Commands::List { .. } => {
            println!("TODO: List issues");
        }
        Commands::Update { id, .. } => {
            println!("TODO: Update issue: {}", id);
        }
        Commands::Close { id, .. } => {
            println!("TODO: Close issue: {}", id);
        }
        Commands::Reopen { id } => {
            println!("TODO: Reopen issue: {}", id);
        }
        Commands::Search { query, .. } => {
            println!("TODO: Search for: {}", query);
        }
        Commands::Similar { id, .. } => {
            println!("TODO: Find similar to: {}", id);
        }
        Commands::Dep { command } => match command {
            DepCommands::Add { from, dep_type, to } => {
                println!("TODO: Add dependency: {} {} {}", from, dep_type, to);
            }
            DepCommands::Remove { from, dep_type, to } => {
                println!("TODO: Remove dependency: {} {} {}", from, dep_type, to);
            }
            DepCommands::List { id } => {
                println!("TODO: List dependencies for: {}", id);
            }
            DepCommands::Cycles => {
                println!("TODO: Find all cycles in dependency graph");
            }
        },
        Commands::Label { command } => match command {
            LabelCommands::Add { id, label } => {
                println!("TODO: Add label '{}' to issue: {}", label, id);
            }
            LabelCommands::Remove { id, label } => {
                println!("TODO: Remove label '{}' from issue: {}", label, id);
            }
            LabelCommands::List { id } => {
                println!("TODO: List labels for issue: {}", id);
            }
            LabelCommands::ListAll => {
                println!("TODO: List all labels in project");
            }
        },
        Commands::Comment { id, message } => {
            println!("TODO: Add comment to {}: {}", id, message);
        }
        Commands::Comments { id } => {
            println!("TODO: List comments for: {}", id);
        }
        Commands::Ready { .. } => {
            println!("TODO: Show ready issues");
        }
        Commands::Blocked { .. } => {
            println!("TODO: Show blocked issues");
        }
        Commands::Explain { id } => {
            println!("TODO: Explain blockers for: {}", id);
        }
        Commands::CriticalPath { id } => {
            println!("TODO: Find critical path for: {}", id);
        }
        Commands::Graph { id, output, depth } => {
            println!(
                "TODO: Generate graph for {} (depth={}, output={:?})",
                id, depth, output
            );
        }
        Commands::Stale {
            days,
            status,
            limit,
        } => {
            println!(
                "TODO: Find stale issues (days={}, status={:?}, limit={:?})",
                days, status, limit
            );
        }
        Commands::Count { status, r#type } => {
            println!(
                "TODO: Count issues (status={:?}, type={:?})",
                status, r#type
            );
        }
        Commands::Merge {
            sources,
            into,
            dry_run,
        } => {
            println!(
                "TODO: Merge {:?} into {} (dry_run={})",
                sources, into, dry_run
            );
        }
        Commands::Restore { id } => {
            println!("TODO: Restore compacted issue: {}", id);
        }
        Commands::Admin { command } => match command {
            AdminCommands::Cleanup {
                older_than,
                dry_run,
                force,
                cascade,
            } => {
                println!(
                    "TODO: Cleanup issues older than {} days (dry_run={}, force={}, cascade={})",
                    older_than, dry_run, force, cascade
                );
            }
        },
        Commands::Sync { .. } => {
            println!("TODO: Sync git <-> HelixDB");
        }
        Commands::Health { .. } => {
            println!("TODO: Show health metrics");
        }
        Commands::Stats { .. } => {
            println!("TODO: Show statistics");
        }
        Commands::Context { .. } => {
            println!("TODO: Get AI context");
        }
        Commands::Compact { dry_run } => {
            println!(
                "TODO: Compact old issues{}",
                if dry_run { " (dry run)" } else { "" }
            );
        }
        Commands::Config { command } => match command {
            ConfigCommands::Show => {
                println!("TODO: Show configuration");
            }
            ConfigCommands::Set { key, value } => {
                println!("TODO: Set {} = {}", key, value);
            }
        },
        Commands::Daemon { command } => match command {
            DaemonCommands::Start => {
                println!("TODO: Start daemon");
            }
            DaemonCommands::Stop => {
                println!("TODO: Stop daemon");
            }
            DaemonCommands::Status => {
                println!("TODO: Show daemon status");
            }
        },
    }
}
