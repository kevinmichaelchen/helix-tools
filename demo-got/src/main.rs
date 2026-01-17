//! CLI for the Game of Thrones family tree demo.

use clap::{Parser, Subcommand};
use demo_got::{
    FamilyTree, GotStorage, House, find_ancestors, find_descendants, get_person_with_family,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "demo-got")]
#[command(about = "Game of Thrones family tree graph demo with HelixDB")]
#[command(version)]
struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Path to the database directory
    #[arg(long, global = true)]
    db_path: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest family tree data from a YAML file
    Ingest {
        /// Path to the YAML file (default: data/westeros.yaml relative to crate)
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Clear existing data before ingesting
        #[arg(long)]
        clear: bool,
    },

    /// Query the family tree graph
    Query {
        #[command(subcommand)]
        query_type: QueryType,
    },

    /// Show database statistics
    Stats,
}

#[derive(Subcommand)]
enum QueryType {
    /// Find all ancestors of a person
    Ancestors {
        /// Person ID (e.g., "jon-snow")
        person_id: String,

        /// Maximum depth to traverse
        #[arg(short, long, default_value = "10")]
        depth: usize,
    },

    /// Find all descendants of a person
    Descendants {
        /// Person ID (e.g., "ned-stark")
        person_id: String,

        /// Maximum depth to traverse
        #[arg(short, long, default_value = "10")]
        depth: usize,
    },

    /// Find all members of a house
    House {
        /// House name (stark, targaryen, baratheon, tully, lannister)
        house: String,
    },

    /// Show person details and immediate family
    Person {
        /// Person ID (e.g., "jon-snow")
        person_id: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    // Default database path is inside the crate directory for co-location
    let db_path = cli
        .db_path
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".data"));

    match cli.command {
        Commands::Ingest { file, clear } => {
            let yaml_path = file.unwrap_or_else(|| {
                // Default to data/westeros.yaml in the crate directory
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/westeros.yaml")
            });

            println!("Loading family tree from: {}", yaml_path.display());
            let tree = FamilyTree::load(&yaml_path)?;
            println!(
                "Loaded {} people and {} relationship definitions",
                tree.people.len(),
                tree.relationships.len()
            );

            let mut storage = GotStorage::new(&db_path)?;

            if clear {
                println!("Clearing existing data...");
                storage.clear()?;
            }

            println!("Ingesting into HelixDB at {}...", db_path.display());
            let stats = storage.ingest(&tree)?;

            println!(
                "Ingested {} nodes and {} edges",
                stats.nodes_inserted, stats.edges_inserted
            );

            // Verify persistence
            let db_stats = storage.get_stats()?;
            println!("\nDatabase verification:");
            println!("  Nodes in DB: {}", db_stats.node_count);
            println!("  Edges in DB: {}", db_stats.edge_count);
            println!("  Houses: {:?}", db_stats.house_counts);
        }

        Commands::Query { query_type } => {
            if !GotStorage::exists(&db_path) {
                anyhow::bail!(
                    "Database not found at {}. Run 'demo-got ingest' first.",
                    db_path.display()
                );
            }

            let storage = GotStorage::new(&db_path)?;

            match query_type {
                QueryType::Ancestors { person_id, depth } => {
                    let ancestors = find_ancestors(&storage, &person_id, depth)?;

                    if cli.json {
                        let output: Vec<_> = ancestors
                            .iter()
                            .map(|a| {
                                serde_json::json!({
                                    "id": a.person.id,
                                    "name": a.person.name,
                                    "house": a.person.house.to_string(),
                                    "depth": a.depth,
                                })
                            })
                            .collect();
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    } else {
                        println!("Ancestors of {}:", person_id);
                        if ancestors.is_empty() {
                            println!("  (none found)");
                        } else {
                            for ancestor in &ancestors {
                                let indent = "  ".repeat(ancestor.depth as usize);
                                let alias = ancestor
                                    .person
                                    .alias
                                    .as_ref()
                                    .map(|a| format!(" \"{}\"", a))
                                    .unwrap_or_default();
                                println!(
                                    "{}{}{} (House {})",
                                    indent, ancestor.person.name, alias, ancestor.person.house
                                );
                            }
                        }
                    }
                }

                QueryType::Descendants { person_id, depth } => {
                    let descendants = find_descendants(&storage, &person_id, depth)?;

                    if cli.json {
                        let output: Vec<_> = descendants
                            .iter()
                            .map(|d| {
                                serde_json::json!({
                                    "id": d.person.id,
                                    "name": d.person.name,
                                    "house": d.person.house.to_string(),
                                    "depth": d.depth,
                                })
                            })
                            .collect();
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    } else {
                        println!("Descendants of {}:", person_id);
                        if descendants.is_empty() {
                            println!("  (none found)");
                        } else {
                            for descendant in &descendants {
                                let indent = "  ".repeat(descendant.depth as usize);
                                println!(
                                    "{}{} (House {})",
                                    indent, descendant.person.name, descendant.person.house
                                );
                            }
                        }
                    }
                }

                QueryType::House { house } => {
                    let house: House = house
                        .parse()
                        .map_err(|e| anyhow::anyhow!("Invalid house: {}", e))?;

                    let members = storage.get_house_members(house)?;

                    if cli.json {
                        let output: Vec<_> = members
                            .iter()
                            .map(|p| {
                                serde_json::json!({
                                    "id": p.id,
                                    "name": p.name,
                                    "alias": p.alias,
                                    "titles": p.titles,
                                    "is_alive": p.is_alive,
                                })
                            })
                            .collect();
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    } else {
                        println!("Members of House {}:", house);
                        for person in &members {
                            let alias = person
                                .alias
                                .as_ref()
                                .map(|a| format!(" \"{}\"", a))
                                .unwrap_or_default();
                            let status = if person.is_alive { "" } else { " [deceased]" };
                            println!("  {}{}{}", person.name, alias, status);
                        }
                        println!("\nTotal: {} members", members.len());
                    }
                }

                QueryType::Person { person_id } => {
                    let family = get_person_with_family(&storage, &person_id)?;

                    if cli.json {
                        let output = serde_json::json!({
                            "person": {
                                "id": family.person.id,
                                "name": family.person.name,
                                "house": family.person.house.to_string(),
                                "alias": family.person.alias,
                                "titles": family.person.titles,
                                "is_alive": family.person.is_alive,
                            },
                            "parents": family.parents.iter().map(|p| {
                                serde_json::json!({ "id": p.id, "name": p.name })
                            }).collect::<Vec<_>>(),
                            "spouses": family.spouses.iter().map(|p| {
                                serde_json::json!({ "id": p.id, "name": p.name })
                            }).collect::<Vec<_>>(),
                            "children": family.children.iter().map(|p| {
                                serde_json::json!({ "id": p.id, "name": p.name })
                            }).collect::<Vec<_>>(),
                            "siblings": family.siblings.iter().map(|p| {
                                serde_json::json!({ "id": p.id, "name": p.name })
                            }).collect::<Vec<_>>(),
                        });
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    } else {
                        println!("{}", family.person.name);
                        if let Some(alias) = &family.person.alias {
                            println!("  Alias: \"{}\"", alias);
                        }
                        println!("  House: {}", family.person.house);
                        if !family.person.titles.is_empty() {
                            println!("  Titles: {}", family.person.titles.join(", "));
                        }
                        println!(
                            "  Status: {}",
                            if family.person.is_alive {
                                "Alive"
                            } else {
                                "Deceased"
                            }
                        );

                        if !family.parents.is_empty() {
                            println!("\nParents:");
                            for parent in &family.parents {
                                println!("  - {} ({})", parent.name, parent.house);
                            }
                        }

                        if !family.spouses.is_empty() {
                            println!("\nSpouse(s):");
                            for spouse in &family.spouses {
                                println!("  - {} ({})", spouse.name, spouse.house);
                            }
                        }

                        if !family.children.is_empty() {
                            println!("\nChildren:");
                            for child in &family.children {
                                println!("  - {} ({})", child.name, child.house);
                            }
                        }

                        if !family.siblings.is_empty() {
                            println!("\nSiblings:");
                            for sibling in &family.siblings {
                                println!("  - {} ({})", sibling.name, sibling.house);
                            }
                        }
                    }
                }
            }
        }

        Commands::Stats => {
            if !GotStorage::exists(&db_path) {
                anyhow::bail!(
                    "Database not found at {}. Run 'demo-got ingest' first.",
                    db_path.display()
                );
            }

            let storage = GotStorage::new(&db_path)?;
            let stats = storage.get_stats()?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                println!("Database Statistics");
                println!("==================");
                println!("Path: {}", db_path.display());
                println!("Nodes: {}", stats.node_count);
                println!("Edges: {}", stats.edge_count);
                println!("\nHouse breakdown:");
                for (house, count) in &stats.house_counts {
                    println!("  {}: {} members", house, count);
                }
            }
        }
    }

    Ok(())
}
