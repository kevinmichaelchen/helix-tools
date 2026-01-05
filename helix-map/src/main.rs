use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use helix_map::{IndexStore, Indexer, JsonStore, RenderOptions, RustExtractor, SkeletonRenderer};
use std::fs;
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "helix-map",
    version,
    about = "HelixDB-backed codebase skeleton indexer"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Index {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        store: Option<PathBuf>,
    },
    Skeleton {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        store: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        include_private: bool,
        #[arg(long)]
        refresh: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Index { path, store } => run_index(&path, store),
        Commands::Skeleton {
            path,
            store,
            output,
            include_private,
            refresh,
        } => run_skeleton(&path, store, output, include_private, refresh),
    }
}

fn run_index(path: &Path, store_override: Option<PathBuf>) -> Result<()> {
    let root = resolve_root(path)?;
    let store_path = store_override.unwrap_or_else(|| default_store_path(&root));
    let store = JsonStore::new(store_path);

    let extractor = RustExtractor::new();
    let indexer = Indexer::new(&store, &extractor);
    let index = indexer.index(&root)?;

    let symbol_count: usize = index.files.iter().map(|file| file.symbols.len()).sum();
    println!(
        "Indexed {} files ({} symbols) into {}",
        index.files.len(),
        symbol_count,
        store.path().display()
    );
    Ok(())
}

fn run_skeleton(
    path: &Path,
    store_override: Option<PathBuf>,
    output: Option<PathBuf>,
    include_private: bool,
    refresh: bool,
) -> Result<()> {
    let root = resolve_root(path)?;
    let store_path = store_override.unwrap_or_else(|| default_store_path(&root));
    let store = JsonStore::new(store_path);

    let extractor = RustExtractor::new();
    let index = if refresh {
        let indexer = Indexer::new(&store, &extractor);
        indexer.index(&root)?
    } else if let Some(index) = store.load()? {
        index
    } else {
        let indexer = Indexer::new(&store, &extractor);
        indexer.index(&root)?
    };

    let renderer = SkeletonRenderer;
    let output_text = renderer.render(&index, RenderOptions { include_private });

    write_output(output.as_deref(), &output_text)?;
    Ok(())
}

fn resolve_root(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).with_context(|| format!("invalid root path {}", path.display()))
}

fn default_store_path(root: &Path) -> PathBuf {
    let store_root = std::env::current_dir().unwrap_or_else(|_| root.to_path_buf());
    store_root.join(".helix-map").join("index.json")
}

fn write_output(path: Option<&Path>, content: &str) -> Result<()> {
    match path {
        None => {
            print!("{}", content);
        }
        Some(path) if path.as_os_str() == "-" => {
            print!("{}", content);
        }
        Some(path) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create output directory {}", parent.display())
                })?;
            }
            fs::write(path, content)
                .with_context(|| format!("failed to write skeleton to {}", path.display()))?;
        }
    }
    Ok(())
}
