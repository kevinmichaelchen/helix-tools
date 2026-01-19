use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    let mut lines = BufReader::new(stdin).lines();
    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: RpcRequest = match serde_json::from_str(line) {
            Ok(req) => req,
            Err(err) => {
                tracing::warn!("Invalid JSON-RPC request: {err}");
                continue;
            }
        };

        if request.jsonrpc != "2.0" {
            tracing::warn!("Ignoring non-JSON-RPC-2.0 request: {}", request.jsonrpc);
            continue;
        }

        let Some(id) = request.id.clone() else {
            continue;
        };

        let response = match request.method.as_str() {
            "initialize" => ok(id, initialize_result()),
            "tools/list" => ok(id, tools_list_result()),
            "tools/call" => match handle_tools_call(request.params) {
                Ok(result) => ok(id, result),
                Err(err) => error(id, -32000, err.to_string(), None),
            },
            _ => error(
                id,
                -32601,
                format!("Method not found: {}", request.method),
                None,
            ),
        };

        let out = serde_json::to_string(&response)?;
        stdout.write_all(out.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    Ok(())
}

const fn ok(id: Value, result: Value) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    }
}

const fn error(id: Value, code: i32, message: String, data: Option<Value>) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(RpcError {
            code,
            message,
            data,
        }),
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "ixchel-mcp",
            "version": ix_core::VERSION
        }
    })
}

fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "ixchel_sync",
                "description": "Sync .ixchel Markdown into the local HelixDB cache",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "repo": { "type": "string", "description": "Path inside the target git repository (defaults to CWD)" }
                    }
                }
            },
            {
                "name": "ixchel_search",
                "description": "Semantic search over Ixchel entities",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "repo": { "type": "string", "description": "Path inside the target git repository (defaults to CWD)" },
                        "query": { "type": "string" },
                        "limit": { "type": "integer", "minimum": 1, "default": 10 }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "ixchel_show",
                "description": "Read an entity Markdown file by id",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "repo": { "type": "string", "description": "Path inside the target git repository (defaults to CWD)" },
                        "id": { "type": "string" }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "ixchel_graph",
                "description": "Return outgoing relationships for an entity",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "repo": { "type": "string", "description": "Path inside the target git repository (defaults to CWD)" },
                        "id": { "type": "string" }
                    },
                    "required": ["id"]
                }
            },
            {
                "name": "ixchel_context",
                "description": "Return a basic 1-hop context pack for an entity",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "repo": { "type": "string", "description": "Path inside the target git repository (defaults to CWD)" },
                        "id": { "type": "string" }
                    },
                    "required": ["id"]
                }
            }
        ]
    })
}

fn handle_tools_call(params: Option<Value>) -> Result<Value> {
    let params = params.unwrap_or_else(|| json!({}));
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("tools/call missing params.name"))?;
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match name {
        "ixchel_sync" => tool_sync(&args),
        "ixchel_search" => tool_search(&args),
        "ixchel_show" => tool_show(&args),
        "ixchel_graph" => tool_graph(&args),
        "ixchel_context" => tool_context(&args),
        _ => anyhow::bail!("Unknown tool: {name}"),
    }
}

fn resolve_repo_path(args: &Value) -> Result<PathBuf> {
    if let Some(path) = args.get("repo").and_then(Value::as_str) {
        return Ok(PathBuf::from(path));
    }
    Ok(std::env::current_dir()?)
}

fn tool_sync(args: &Value) -> Result<Value> {
    let repo_path = resolve_repo_path(args)?;
    let repo = ix_core::repo::IxchelRepo::open_from(&repo_path)?;
    let stats = ix_app::sync(&repo)?;

    tool_text(&json!({
        "scanned": stats.scanned,
        "added": stats.added,
        "modified": stats.modified,
        "deleted": stats.deleted,
        "unchanged": stats.unchanged
    }))
}

fn tool_search(args: &Value) -> Result<Value> {
    let repo_path = resolve_repo_path(args)?;
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("ixchel_search missing arguments.query"))?;
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(10);

    let repo = ix_core::repo::IxchelRepo::open_from(&repo_path)?;
    let hits = ix_app::search(&repo, query, limit)?;

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

    tool_text(&json!({ "hits": hits }))
}

fn tool_show(args: &Value) -> Result<Value> {
    let repo_path = resolve_repo_path(args)?;
    let id = args
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("ixchel_show missing arguments.id"))?;

    let repo = ix_core::repo::IxchelRepo::open_from(&repo_path)?;
    let raw = repo.read_raw(id)?;

    tool_text(&json!({ "id": id, "raw": raw }))
}

fn tool_graph(args: &Value) -> Result<Value> {
    let repo_path = resolve_repo_path(args)?;
    let id = args
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("ixchel_graph missing arguments.id"))?;

    let repo = ix_core::repo::IxchelRepo::open_from(&repo_path)?;
    let graph = build_graph_json(&repo, id)?;

    tool_text(&graph)
}

fn tool_context(args: &Value) -> Result<Value> {
    let repo_path = resolve_repo_path(args)?;
    let id = args
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("ixchel_context missing arguments.id"))?;

    let repo = ix_core::repo::IxchelRepo::open_from(&repo_path)?;
    let context = build_context_json(&repo, id)?;

    tool_text(&context)
}

fn tool_text(payload: &Value) -> Result<Value> {
    let text = serde_json::to_string_pretty(payload)?;
    Ok(json!({
        "content": [{ "type": "text", "text": text }],
    }))
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

fn build_graph_json(repo: &ix_core::repo::IxchelRepo, id: &str) -> Result<Value> {
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

fn build_context_json(repo: &ix_core::repo::IxchelRepo, id: &str) -> Result<Value> {
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
        let serde_yaml::Value::String(key) = key else {
            continue;
        };

        if METADATA_KEYS.contains(&key.as_str()) {
            continue;
        }

        let targets = match value {
            serde_yaml::Value::Sequence(seq) => seq
                .iter()
                .filter_map(|v| match v {
                    serde_yaml::Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            serde_yaml::Value::String(s) => vec![s.clone()],
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
