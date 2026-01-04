use std::collections::HashMap;

use tabled::{Table, Tabled, settings::Style};

use hbd::{CreatorType, TicketStore};

pub fn add(store: &TicketStore, id: &str, labels_input: &str, json: bool) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let mut issue = store.read_issue(&id)?;

    let labels_to_add: Vec<String> = labels_input
        .split(',')
        .map(|l| l.trim().to_lowercase())
        .filter(|l| !l.is_empty())
        .collect();

    if labels_to_add.is_empty() {
        return Err(hbd::HbdError::Other("no valid labels provided".to_string()));
    }

    let mut added = Vec::new();
    let mut skipped = Vec::new();

    for label in labels_to_add {
        if issue.labels.contains(&label) {
            skipped.push(label);
        } else {
            issue.labels.push(label.clone());
            added.push(label);
        }
    }

    if !added.is_empty() {
        issue.labels.sort();
        issue.touch();
        store.write_issue(&issue)?;
    }

    if json {
        let result = serde_json::json!({
            "id": id,
            "added": added,
            "unchanged": skipped,
            "labels": issue.labels
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if added.is_empty() {
        println!("All labels already on {id}");
    } else if skipped.is_empty() {
        println!(
            "Added {} label(s) to {id}: {}",
            added.len(),
            added.join(", ")
        );
    } else {
        println!(
            "Added {} label(s) to {id}: {} (skipped existing: {})",
            added.len(),
            added.join(", "),
            skipped.join(", ")
        );
    }
    Ok(())
}

pub fn remove(store: &TicketStore, id: &str, label: &str, json: bool) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let mut issue = store.read_issue(&id)?;

    let label = label.trim().to_lowercase();
    let original_len = issue.labels.len();
    issue.labels.retain(|l| l != &label);

    if issue.labels.len() == original_len {
        if json {
            let result = serde_json::json!({
                "id": id,
                "label": label,
                "action": "unchanged",
                "warning": format!("issue {id} does not have label '{label}'"),
                "labels": issue.labels
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            eprintln!("warning: issue {id} does not have label '{label}'");
        }
        return Ok(());
    }

    issue.touch();
    store.write_issue(&issue)?;

    if json {
        let result = serde_json::json!({
            "id": id,
            "label": label,
            "action": "removed",
            "labels": issue.labels
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Removed label '{label}' from {id}");
    }
    Ok(())
}

pub fn list(store: &TicketStore, id: &str, json: bool) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let issue = store.read_issue(&id)?;

    if json {
        let result = serde_json::json!({
            "id": id,
            "labels": issue.labels
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if issue.labels.is_empty() {
        println!("{id} has no labels");
    } else {
        println!("Labels for {id}:");
        for label in &issue.labels {
            println!("  {label}");
        }
    }
    Ok(())
}

pub fn list_all(store: &TicketStore, json: bool) -> hbd::Result<()> {
    let issues = store.read_all_issues()?;

    let mut label_counts: HashMap<String, usize> = HashMap::new();
    for issue in &issues {
        for label in &issue.labels {
            *label_counts.entry(label.clone()).or_insert(0) += 1;
        }
    }

    let mut labels: Vec<_> = label_counts.into_iter().collect();
    labels.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    if json {
        let result: Vec<_> = labels
            .iter()
            .map(|(name, count)| serde_json::json!({"name": name, "count": count}))
            .collect();
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if labels.is_empty() {
        println!("No labels in project");
    } else {
        #[derive(Tabled)]
        struct Row {
            #[tabled(rename = "Label")]
            name: String,
            #[tabled(rename = "Count")]
            count: usize,
        }

        let rows: Vec<Row> = labels
            .iter()
            .map(|(name, count)| Row {
                name: name.clone(),
                count: *count,
            })
            .collect();

        println!("Labels in project:\n");
        let mut table = Table::new(rows);
        table.with(Style::rounded());
        println!("{table}");
        println!("\n{} label(s)", labels.len());
    }
    Ok(())
}

pub fn comment(
    store: &TicketStore,
    id: &str,
    message: &str,
    agent: Option<&str>,
    json: bool,
) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let mut issue = store.read_issue(&id)?;

    let (author, creator_type) = agent.map_or_else(
        || (whoami::username(), CreatorType::Human),
        |agent_id| (agent_id.to_string(), CreatorType::Agent),
    );

    issue.add_comment(message, &author, creator_type);
    store.write_issue(&issue)?;

    if json {
        let comment = issue.comments.last().unwrap();
        println!("{}", serde_json::to_string_pretty(comment)?);
    } else {
        println!("Added comment to {}", issue.id);
    }
    Ok(())
}

pub fn comments(store: &TicketStore, id: &str, json: bool) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let issue = store.read_issue(&id)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&issue.comments)?);
    } else {
        if issue.comments.is_empty() {
            println!("No comments on {}", issue.id);
            return Ok(());
        }
        for c in &issue.comments {
            let suffix = if c.created_by_type == CreatorType::Agent {
                " (agent)"
            } else {
                ""
            };
            println!(
                "### {} — {}{}",
                c.created_at.format("%Y-%m-%d %H:%M:%S"),
                c.created_by,
                suffix
            );
            println!("{}\n", c.body);
        }
    }
    Ok(())
}
