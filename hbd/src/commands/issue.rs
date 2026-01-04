use hbd::domain::graph::find_open_children;
use hbd::{CreatorType, Issue, IssueType, Priority, TicketStore};

pub fn init() -> hbd::Result<()> {
    let store = TicketStore::new(std::env::current_dir()?);
    store.init()?;
    println!("Initialized hbd in current directory");
    println!("  Created .tickets/");
    println!("  Created .helix/config.toml");
    println!("  Updated .gitignore");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    store: &TicketStore,
    title: &str,
    description: Option<&str>,
    issue_type: &str,
    priority: u8,
    labels: Option<&str>,
    assignee: Option<&str>,
    agent: Option<&str>,
    session: Option<&str>,
    parent: Option<&str>,
    json: bool,
) -> hbd::Result<()> {
    let issue_type: IssueType = issue_type.parse().map_err(hbd::HbdError::Other)?;
    let priority = Priority::from_u8(priority).unwrap_or(Priority::Medium);

    let mut builder = Issue::builder(title)
        .issue_type(issue_type)
        .priority(priority);

    if let Some(desc) = description {
        builder = builder.body(desc);
    }
    if let Some(a) = assignee {
        builder = builder.assignee(a);
    }
    if let Some(agent_id) = agent {
        builder = builder.agent(agent_id);
    }
    if let Some(sess) = session {
        builder = builder.session(sess);
    }
    if let Some(p) = parent {
        let parent_id = store.resolve_id(p)?;
        builder = builder.parent(parent_id);
    }
    if let Some(l) = labels {
        builder = builder.labels(l.split(',').map(str::trim));
    }

    let issue = builder.build();
    store.write_issue(&issue)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&issue)?);
    } else {
        println!("{}", issue.id);
    }

    Ok(())
}

pub fn show(store: &TicketStore, id: &str, json: bool) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let issue = store.read_issue(&id)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&issue)?);
    } else {
        println!("{} {}", issue.id, issue.title);
        println!(
            "Status: {} | Priority: {} | Type: {}",
            issue.status,
            issue.priority.as_u8(),
            issue.issue_type
        );
        if let Some(ref assignee) = issue.assignee {
            println!("Assignee: {assignee}");
        }
        if let Some(ref parent) = issue.parent_id {
            println!("Parent: {parent}");
        }
        if !issue.labels.is_empty() {
            println!("Labels: {}", issue.labels.join(", "));
        }
        if !issue.depends_on.is_empty() {
            println!("Depends on:");
            for dep in &issue.depends_on {
                println!("  - {} ({})", dep.id, dep.dep_type);
            }
        }
        if !issue.body.is_empty() {
            println!("\n{}", issue.body);
        }
        if !issue.comments.is_empty() {
            println!("\nComments ({}):", issue.comments.len());
            for c in &issue.comments {
                println!(
                    "  [{} — {}]",
                    c.created_at.format("%Y-%m-%d %H:%M"),
                    c.created_by
                );
                for line in c.body.lines().take(2) {
                    println!("    {line}");
                }
            }
        }
    }
    Ok(())
}

pub fn update(
    store: &TicketStore,
    id: &str,
    status: Option<&str>,
    priority: Option<u8>,
    title: Option<&str>,
    assignee: Option<&str>,
    json: bool,
) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let mut issue = store.read_issue(&id)?;

    if let Some(s) = status {
        issue.status = s.parse().map_err(hbd::HbdError::Other)?;
    }
    if let Some(p) = priority
        && let Some(p) = Priority::from_u8(p)
    {
        issue.priority = p;
    }
    if let Some(t) = title {
        issue.title = t.to_string();
    }
    if let Some(a) = assignee {
        issue.assignee = Some(a.to_string());
    }

    issue.touch();
    store.write_issue(&issue)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&issue)?);
    } else {
        println!("Updated {}", issue.id);
    }
    Ok(())
}

pub fn close(
    store: &TicketStore,
    id: &str,
    reason: Option<&str>,
    force: bool,
    json: bool,
) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let mut issue = store.read_issue(&id)?;

    let all_issues = store.read_all_issues_map()?;
    let open_children = find_open_children(&id, &all_issues);

    if !open_children.is_empty() && !force {
        if json {
            let children: Vec<_> = open_children
                .iter()
                .map(|c| serde_json::json!({"id": &c.id, "title": &c.title, "status": c.status.as_str()}))
                .collect();
            let result = serde_json::json!({
                "error": "has_open_children",
                "message": format!("Issue {id} has {} open child issue(s). Use --force to close anyway.", open_children.len()),
                "open_children": children
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            eprintln!(
                "error: issue {id} has {} open child issue(s):",
                open_children.len()
            );
            for child in &open_children {
                eprintln!("  - {} ({}): {}", child.id, child.status, child.title);
            }
            eprintln!("\nUse --force to close anyway.");
        }
        return Err(hbd::HbdError::Other(
            "cannot close issue with open children without --force".to_string(),
        ));
    }

    let user = whoami::username();
    issue.close(reason.map(String::from), &user, CreatorType::Human);
    store.write_issue(&issue)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&issue)?);
    } else {
        println!("Closed {}", issue.id);
    }
    Ok(())
}

pub fn reopen(store: &TicketStore, id: &str, json: bool) -> hbd::Result<()> {
    let id = store.resolve_id(id)?;
    let mut issue = store.read_issue(&id)?;

    issue.reopen();
    store.write_issue(&issue)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&issue)?);
    } else {
        println!("Reopened {}", issue.id);
    }
    Ok(())
}
