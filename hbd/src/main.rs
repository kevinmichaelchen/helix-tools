use std::process::ExitCode;

use clap::{Parser, Subcommand};
use hbd::TicketStore;

mod commands;

#[derive(Parser)]
#[command(name = "hbd")]
#[command(author, version, about = "Git-first issue tracker powered by HelixDB")]
#[command(propagate_version = true)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,

    Info,

    Create {
        title: String,

        #[arg(short, long)]
        description: Option<String>,

        #[arg(short = 't', long, default_value = "task")]
        r#type: String,

        #[arg(short, long, default_value = "2")]
        priority: u8,

        #[arg(short, long)]
        labels: Option<String>,

        #[arg(short, long)]
        assignee: Option<String>,

        #[arg(long)]
        agent: Option<String>,

        #[arg(long)]
        session: Option<String>,

        #[arg(long)]
        parent: Option<String>,
    },

    Show {
        id: String,
    },

    List {
        #[arg(short, long)]
        status: Option<String>,

        #[arg(short = 't', long)]
        r#type: Option<String>,

        #[arg(short, long)]
        priority: Option<u8>,

        #[arg(short, long)]
        label: Option<String>,

        #[arg(short, long)]
        assignee: Option<String>,
    },

    Update {
        id: String,

        #[arg(long)]
        status: Option<String>,

        #[arg(long)]
        priority: Option<u8>,

        #[arg(long)]
        title: Option<String>,

        #[arg(long)]
        assignee: Option<String>,
    },

    Close {
        id: String,

        #[arg(short, long)]
        reason: Option<String>,

        #[arg(long)]
        force: bool,
    },

    Reopen {
        id: String,
    },

    Search {
        query: String,

        #[arg(long)]
        semantic: bool,

        #[arg(long)]
        hybrid: bool,

        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    Similar {
        id: String,

        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    Dep {
        #[command(subcommand)]
        command: DepCommands,
    },

    Label {
        #[command(subcommand)]
        command: LabelCommands,
    },

    Comment {
        id: String,
        message: String,

        #[arg(long)]
        agent: Option<String>,
    },

    Comments {
        id: String,
    },

    Ready,

    Blocked,

    Explain {
        id: String,
    },

    CriticalPath {
        id: String,
    },

    Graph {
        id: String,

        #[arg(short, long)]
        output: Option<String>,

        #[arg(long, default_value = "5")]
        depth: usize,
    },

    Stale {
        #[arg(long, default_value = "14")]
        days: u32,

        #[arg(short, long)]
        status: Option<String>,

        #[arg(short, long)]
        limit: Option<usize>,
    },

    Count {
        #[arg(short, long)]
        status: Option<String>,

        #[arg(short = 't', long)]
        r#type: Option<String>,
    },

    Merge {
        sources: Vec<String>,

        #[arg(long)]
        into: String,

        #[arg(long)]
        dry_run: bool,
    },

    Restore {
        id: String,
    },

    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },

    Sync {
        #[arg(long)]
        import_only: bool,

        #[arg(long)]
        export_only: bool,
    },

    Health {
        #[arg(long)]
        label: Option<String>,
    },

    Stats,

    Context {
        #[arg(long)]
        query: Option<String>,

        #[arg(long)]
        limit: Option<usize>,
    },

    Compact {
        #[arg(long)]
        dry_run: bool,
    },

    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
}

#[derive(Subcommand)]
enum DepCommands {
    Add {
        from: String,
        dep_type: String,
        to: String,
    },
    Remove {
        from: String,
        dep_type: String,
        to: String,
    },
    List {
        id: String,
    },
    Cycles,
}

#[derive(Subcommand)]
enum LabelCommands {
    Add { id: String, label: String },
    Remove { id: String, label: String },
    List { id: String },
    ListAll,
}

#[derive(Subcommand)]
enum ConfigCommands {
    Show,
    Set { key: String, value: String },
}

#[derive(Subcommand)]
enum AdminCommands {
    Cleanup {
        #[arg(long, default_value = "90")]
        older_than: u32,

        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        force: bool,

        #[arg(long)]
        cascade: bool,
    },
}

#[derive(Subcommand, Clone, Copy)]
enum DaemonCommands {
    Start,
    Stop,
    Status,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            ExitCode::from(e.exit_code() as u8)
        }
    }
}

#[allow(clippy::too_many_lines)]
fn run(cli: Cli) -> hbd::Result<()> {
    match cli.command {
        Commands::Init => commands::issue::init(),
        Commands::Info => {
            let store = TicketStore::from_current_dir()?;
            commands::query::info(&store, cli.json)
        }
        Commands::Create {
            title,
            description,
            r#type,
            priority,
            labels,
            assignee,
            agent,
            session,
            parent,
        } => {
            let store = TicketStore::from_current_dir()?;
            commands::issue::create(
                &store,
                &title,
                description.as_deref(),
                &r#type,
                priority,
                labels.as_deref(),
                assignee.as_deref(),
                agent.as_deref(),
                session.as_deref(),
                parent.as_deref(),
                cli.json,
            )
        }
        Commands::Show { id } => {
            let store = TicketStore::from_current_dir()?;
            commands::issue::show(&store, &id, cli.json)
        }
        Commands::List {
            status,
            r#type,
            priority,
            label,
            assignee,
        } => {
            let store = TicketStore::from_current_dir()?;
            commands::query::list(
                &store,
                status.as_deref(),
                r#type.as_deref(),
                priority,
                label.as_deref(),
                assignee.as_deref(),
                cli.json,
            )
        }
        Commands::Update {
            id,
            status,
            priority,
            title,
            assignee,
        } => {
            let store = TicketStore::from_current_dir()?;
            commands::issue::update(
                &store,
                &id,
                status.as_deref(),
                priority,
                title.as_deref(),
                assignee.as_deref(),
                cli.json,
            )
        }
        Commands::Close { id, reason, force } => {
            let store = TicketStore::from_current_dir()?;
            commands::issue::close(&store, &id, reason.as_deref(), force, cli.json)
        }
        Commands::Reopen { id } => {
            let store = TicketStore::from_current_dir()?;
            commands::issue::reopen(&store, &id, cli.json)
        }
        Commands::Comment { id, message, agent } => {
            let store = TicketStore::from_current_dir()?;
            commands::labels::comment(&store, &id, &message, agent.as_deref(), cli.json)
        }
        Commands::Comments { id } => {
            let store = TicketStore::from_current_dir()?;
            commands::labels::comments(&store, &id, cli.json)
        }
        Commands::Ready => {
            let store = TicketStore::from_current_dir()?;
            commands::query::ready(&store, cli.json)
        }
        Commands::Blocked => {
            let store = TicketStore::from_current_dir()?;
            commands::query::blocked(&store, cli.json)
        }
        Commands::Dep { command } => {
            let store = TicketStore::from_current_dir()?;
            match command {
                DepCommands::Add { from, dep_type, to } => {
                    commands::deps::add(&store, &from, &dep_type, &to, cli.json)
                }
                DepCommands::Remove { from, dep_type, to } => {
                    commands::deps::remove(&store, &from, &dep_type, &to, cli.json)
                }
                DepCommands::List { id } => commands::deps::list(&store, &id, cli.json),
                DepCommands::Cycles => commands::deps::cycles(&store, cli.json),
            }
        }
        Commands::Label { command } => {
            let store = TicketStore::from_current_dir()?;
            match command {
                LabelCommands::Add { id, label } => {
                    commands::labels::add(&store, &id, &label, cli.json)
                }
                LabelCommands::Remove { id, label } => {
                    commands::labels::remove(&store, &id, &label, cli.json)
                }
                LabelCommands::List { id } => commands::labels::list(&store, &id, cli.json),
                LabelCommands::ListAll => commands::labels::list_all(&store, cli.json),
            }
        }
        Commands::Explain { id } => {
            let store = TicketStore::from_current_dir()?;
            commands::deps::explain(&store, &id, cli.json)
        }
        Commands::Stale {
            days,
            status,
            limit,
        } => {
            let store = TicketStore::from_current_dir()?;
            commands::query::stale(&store, days, status.as_deref(), limit, cli.json)
        }
        Commands::Stats => {
            let store = TicketStore::from_current_dir()?;
            commands::query::stats(&store, cli.json)
        }
        _ => {
            eprintln!("Command not yet implemented. See specs/tasks.md for roadmap.");
            Ok(())
        }
    }
}
