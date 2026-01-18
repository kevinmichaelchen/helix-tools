//! CLI for the Game of Thrones family tree demo.

use clap::{Parser, Subcommand};
use demo_got::{
    BioLoader, FamilyTree, GotStorage, House, find_ancestors, find_descendants,
    get_person_with_family,
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

        /// Skip generating embeddings (faster for development iteration)
        #[arg(long)]
        skip_embeddings: bool,
    },

    /// Semantic search across character bios
    Search {
        /// Search query (natural language)
        query: String,

        /// Maximum number of results to return
        #[arg(short, long, default_value = "5")]
        limit: usize,
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
        Commands::Ingest {
            file,
            clear,
            skip_embeddings,
        } => {
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

            // Load biographies
            let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
            let bios = BioLoader::load_all(&data_dir)?;
            println!("Loaded {} character biographies", bios.len());

            let mut storage = GotStorage::new(&db_path)?;

            if clear {
                println!("Clearing existing data...");
                storage.clear()?;
            }

            println!("Ingesting into HelixDB at {}...", db_path.display());

            if skip_embeddings {
                // Use the original ingest without embeddings
                let stats = storage.ingest(&tree)?;
                println!(
                    "Ingested {} nodes and {} edges (no embeddings)",
                    stats.nodes_inserted, stats.edges_inserted
                );
            } else {
                // Generate embeddings and ingest with vectors
                println!("Initializing embedding model...");
                let embedder = helix_embeddings::Embedder::new()
                    .map_err(|e| anyhow::anyhow!("Failed to create embedder: {e}"))?;

                // Build composite texts for people with bios
                let mut texts_with_ids: Vec<(String, String)> = Vec::new();
                for person in &tree.people {
                    if let Some(bio) = bios.get(&person.id) {
                        let composite = bio.composite_text(person);
                        texts_with_ids.push((person.id.clone(), composite));
                    }
                }

                if texts_with_ids.is_empty() {
                    println!("Warning: No biographies found, falling back to basic ingest");
                    let stats = storage.ingest(&tree)?;
                    println!(
                        "Ingested {} nodes and {} edges (no embeddings)",
                        stats.nodes_inserted, stats.edges_inserted
                    );
                } else {
                    println!(
                        "Generating embeddings for {} characters...",
                        texts_with_ids.len()
                    );
                    let texts: Vec<&str> = texts_with_ids.iter().map(|(_, t)| t.as_str()).collect();
                    let embeddings = embedder
                        .embed_batch(&texts)
                        .map_err(|e| anyhow::anyhow!("Failed to generate embeddings: {e}"))?;

                    // Create a map of person_id -> embedding
                    let embedding_map: std::collections::HashMap<String, Vec<f32>> = texts_with_ids
                        .iter()
                        .zip(embeddings)
                        .map(|((id, _), emb)| (id.clone(), emb))
                        .collect();

                    // Ingest with embeddings
                    let stats = ingest_with_embeddings(&mut storage, &tree, &embedding_map)?;
                    println!(
                        "Ingested {} nodes and {} edges ({} with embeddings)",
                        stats.nodes_inserted, stats.edges_inserted, stats.embeddings_inserted
                    );
                }
            }

            // Verify persistence
            let db_stats = storage.get_stats()?;
            println!("\nDatabase verification:");
            println!("  Nodes in DB: {}", db_stats.node_count);
            println!("  Edges in DB: {}", db_stats.edge_count);
            println!("  Houses: {:?}", db_stats.house_counts);
        }

        Commands::Search { query, limit } => {
            if !GotStorage::exists(&db_path) {
                anyhow::bail!(
                    "Database not found at {}. Run 'demo-got ingest' first.",
                    db_path.display()
                );
            }

            let storage = GotStorage::new(&db_path)?;

            // Generate query embedding
            let embedder = helix_embeddings::Embedder::new()
                .map_err(|e| anyhow::anyhow!("Failed to create embedder: {e}"))?;
            let query_embedding = embedder
                .embed(&query)
                .map_err(|e| anyhow::anyhow!("Failed to embed query: {e}"))?;

            // Perform semantic search
            let results = storage.search_semantic(&query_embedding, limit)?;

            if cli.json {
                let output: Vec<_> = results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "id": r.person.id,
                            "name": r.person.name,
                            "house": r.person.house.to_string(),
                            "alias": r.person.alias,
                            "score": r.score,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Search results for: \"{}\"", query);
                println!();
                if results.is_empty() {
                    println!("  (no results found)");
                } else {
                    for (i, result) in results.iter().enumerate() {
                        let alias = result
                            .person
                            .alias
                            .as_ref()
                            .map(|a| format!(" \"{}\"", a))
                            .unwrap_or_default();
                        println!(
                            "{}. {}{} (House {}) - score: {:.3}",
                            i + 1,
                            result.person.name,
                            alias,
                            result.person.house,
                            result.score
                        );
                    }
                }
            }
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

/// Statistics from an ingest operation with embeddings.
struct IngestWithEmbeddingsStats {
    nodes_inserted: usize,
    edges_inserted: usize,
    embeddings_inserted: usize,
}

/// Ingest family tree data with embeddings for semantic search.
fn ingest_with_embeddings(
    storage: &mut GotStorage,
    tree: &FamilyTree,
    embeddings: &std::collections::HashMap<String, Vec<f32>>,
) -> anyhow::Result<IngestWithEmbeddingsStats> {
    use demo_got::{RelationType, RelationshipDef};

    let mut stats = IngestWithEmbeddingsStats {
        nodes_inserted: 0,
        edges_inserted: 0,
        embeddings_inserted: 0,
    };

    // Track person ID -> node ID mapping for relationship creation
    let mut id_to_node: std::collections::HashMap<String, u128> = std::collections::HashMap::new();

    // First pass: insert all people as nodes
    for person in &tree.people {
        let node_id = if let Some(embedding) = embeddings.get(&person.id) {
            // Insert with embedding
            let (node_id, _vector_id) = storage.insert_person_with_embedding(person, embedding)?;
            stats.embeddings_inserted += 1;
            node_id
        } else {
            // Fall back to basic insert (calls the internal method via ingest)
            // For now, we'll skip people without embeddings in the embedding path
            // They can be ingested via the regular path
            continue;
        };
        id_to_node.insert(person.id.clone(), node_id);
        stats.nodes_inserted += 1;
    }

    // Second pass: create all relationship edges
    for rel in &tree.relationships {
        match rel {
            RelationshipDef::ParentOf { from, to } => {
                let Some(&from_node) = id_to_node.get(from) else {
                    continue;
                };

                for child_id in to {
                    let Some(&to_node) = id_to_node.get(child_id) else {
                        continue;
                    };
                    create_edge_internal(storage, from_node, to_node, RelationType::ParentOf)?;
                    stats.edges_inserted += 1;
                }
            }
            RelationshipDef::SpouseOf { between } => {
                if between.len() >= 2 {
                    let Some(&a) = id_to_node.get(&between[0]) else {
                        continue;
                    };
                    let Some(&b) = id_to_node.get(&between[1]) else {
                        continue;
                    };
                    // Bidirectional: create edges in both directions
                    create_edge_internal(storage, a, b, RelationType::SpouseOf)?;
                    create_edge_internal(storage, b, a, RelationType::SpouseOf)?;
                    stats.edges_inserted += 2;
                }
            }
            RelationshipDef::SiblingOf { between } => {
                // Create edges between all pairs (bidirectional)
                for i in 0..between.len() {
                    for j in (i + 1)..between.len() {
                        let Some(&a) = id_to_node.get(&between[i]) else {
                            continue;
                        };
                        let Some(&b) = id_to_node.get(&between[j]) else {
                            continue;
                        };
                        create_edge_internal(storage, a, b, RelationType::SiblingOf)?;
                        create_edge_internal(storage, b, a, RelationType::SiblingOf)?;
                        stats.edges_inserted += 2;
                    }
                }
            }
        }
    }

    Ok(stats)
}

/// Helper to create an edge using the public storage method.
fn create_edge_internal(
    storage: &GotStorage,
    from_node_id: u128,
    to_node_id: u128,
    relation_type: demo_got::RelationType,
) -> anyhow::Result<()> {
    storage.create_edge_public(from_node_id, to_node_id, relation_type)?;
    Ok(())
}
