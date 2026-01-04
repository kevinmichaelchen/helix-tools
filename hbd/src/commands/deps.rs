use hbd::domain::graph::{
    BlockerNode, build_blocker_tree, count_open_blockers, find_all_cycles, would_create_cycle,
};
use hbd::{DepType, TicketStore};

pub fn add(
    store: &TicketStore,
    from: &str,
    dep_type: &str,
    to: &str,
    json: bool,
) -> hbd::Result<()> {
    let from_id = store.resolve_id(from)?;
    let to_id = store.resolve_id(to)?;
    let dep_type: DepType = dep_type.parse().map_err(hbd::HbdError::Other)?;

    if from_id == to_id {
        return Err(hbd::HbdError::Other(
            "cannot add self-dependency".to_string(),
        ));
    }

    if dep_type == DepType::Blocks {
        let issues_map = store.read_all_issues_map()?;
        if would_create_cycle(&issues_map, &to_id, &from_id) {
            return Err(hbd::HbdError::Other(format!(
                "adding this dependency would create a cycle: {to_id} -> {from_id} -> ... -> {to_id}"
            )));
        }
    }

    let mut issue = store.read_issue(&to_id)?;
    issue.add_dependency(&from_id, dep_type);
    store.write_issue(&issue)?;

    if json {
        let result = serde_json::json!({
            "blocker": from_id,
            "blocked": to_id,
            "dep_type": dep_type.as_str(),
            "action": "added"
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{to_id} now depends on {from_id}");
    }
    Ok(())
}

pub fn remove(
    store: &TicketStore,
    from: &str,
    dep_type: &str,
    to: &str,
    json: bool,
) -> hbd::Result<()> {
    let from_id = store.resolve_id(from)?;
    let to_id = store.resolve_id(to)?;
    let _: DepType = dep_type.parse().map_err(hbd::HbdError::Other)?;

    let mut issue = store.read_issue(&to_id)?;
    let removed = issue.remove_dependency(&from_id);

    if !removed {
        return Err(hbd::HbdError::Other(format!(
            "{to_id} does not depend on {from_id}"
        )));
    }

    store.write_issue(&issue)?;

    if json {
        let result = serde_json::json!({
            "blocker": from_id,
            "blocked": to_id,
            "action": "removed"
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{to_id} no longer depends on {from_id}");
    }
    Ok(())
}

pub fn list(store: &TicketStore, id: &str, json: bool) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let issue = store.read_issue(&id)?;
    let issues_map = store.read_all_issues_map()?;
    let blocks: Vec<_> = issues_map
        .values()
        .filter(|i| i.depends_on.iter().any(|d| d.id == id))
        .collect();

    if json {
        let depends_on: Vec<_> = issue
            .depends_on
            .iter()
            .map(|d| {
                let title = issues_map
                    .get(&d.id)
                    .map_or("(unknown)", |i| i.title.as_str());
                serde_json::json!({
                    "id": d.id,
                    "dep_type": d.dep_type.as_str(),
                    "title": title
                })
            })
            .collect();
        let blocked_by_this: Vec<_> = blocks
            .iter()
            .map(|i| serde_json::json!({"id": i.id, "title": i.title}))
            .collect();
        let result = serde_json::json!({
            "id": id,
            "depends_on": depends_on,
            "blocks": blocked_by_this
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Dependencies for {id}:\n");
        if issue.depends_on.is_empty() {
            println!("  Depends on: (none)");
        } else {
            println!("  Depends on:");
            for dep in &issue.depends_on {
                let title = issues_map
                    .get(&dep.id)
                    .map_or_else(|| "(unknown)".to_string(), |i| truncate(&i.title, 30));
                println!("    {} ({}) - {}", dep.id, dep.dep_type, title);
            }
        }
        println!();
        if blocks.is_empty() {
            println!("  Blocks: (none)");
        } else {
            println!("  Blocks:");
            for i in &blocks {
                println!("    {} - {}", i.id, truncate(&i.title, 30));
            }
        }
    }
    Ok(())
}

pub fn cycles(store: &TicketStore, json: bool) -> hbd::Result<()> {
    let issues_map = store.read_all_issues_map()?;
    let cycles = find_all_cycles(&issues_map);

    if json {
        let result = serde_json::json!({
            "cycles": cycles,
            "count": cycles.len()
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if cycles.is_empty() {
        println!("No dependency cycles found.");
    } else {
        println!("Found {} dependency cycle(s):\n", cycles.len());
        for (i, cycle) in cycles.iter().enumerate() {
            println!("  Cycle {}: {}", i + 1, cycle.join(" -> "));
        }
    }
    Ok(())
}

pub fn explain(store: &TicketStore, id: &str, json: bool) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let issues_map = store.read_all_issues_map()?;

    if !issues_map.contains_key(&id) {
        return Err(hbd::HbdError::IssueNotFound(id));
    }

    let tree = build_blocker_tree(&id, &issues_map);

    if json {
        println!("{}", serde_json::to_string_pretty(&tree)?);
    } else {
        fn print_tree(node: &BlockerNode, prefix: &str, is_last: bool) {
            let connector = if node.depth == 0 {
                ""
            } else if is_last {
                "└── "
            } else {
                "├── "
            };
            let status_icon = match node.status.as_str() {
                "closed" => "✓",
                "blocked" => "⊘",
                "in_progress" => "◐",
                _ => "○",
            };
            println!(
                "{prefix}{connector}{status_icon} {} {}",
                node.id,
                truncate(&node.title, 40)
            );

            let child_prefix = if node.depth == 0 {
                String::new()
            } else if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };

            for (i, blocker) in node.blockers.iter().enumerate() {
                let is_last_child = i == node.blockers.len() - 1;
                print_tree(blocker, &child_prefix, is_last_child);
            }
        }

        if let Some(ref tree) = tree {
            println!("Blocker tree for {id}:\n");
            print_tree(tree, "", true);

            let open_blockers = count_open_blockers(tree);
            if open_blockers > 0 {
                println!("\n{open_blockers} open blocker(s) in chain");
            } else {
                println!("\nNo open blockers - ready to work!");
            }
        }
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
