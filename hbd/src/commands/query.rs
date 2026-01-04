use std::collections::HashMap;

use tabled::{Table, Tabled, settings::Style};

use hbd::domain::filters::{blocked_issues, ready_issues, stale_issues};
use hbd::{IssueType, Priority, Status, TicketStore};

pub fn info(store: &TicketStore, json: bool) -> hbd::Result<()> {
    let ids = store.list_issue_ids()?;
    let open_count = ids
        .iter()
        .filter_map(|id| store.read_issue(id).ok())
        .filter(|i| i.status != Status::Closed)
        .count();

    if json {
        let info = serde_json::json!({
            "initialized": store.is_initialized(),
            "tickets_dir": store.tickets_dir(),
            "total_issues": ids.len(),
            "open_issues": open_count,
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("hbd status:");
        println!("  Initialized: {}", store.is_initialized());
        println!("  Tickets dir: {}", store.tickets_dir().display());
        println!("  Total issues: {}", ids.len());
        println!("  Open issues: {open_count}");
    }
    Ok(())
}

pub fn list(
    store: &TicketStore,
    status: Option<&str>,
    issue_type: Option<&str>,
    priority: Option<u8>,
    label: Option<&str>,
    assignee: Option<&str>,
    json: bool,
) -> hbd::Result<()> {
    let mut issues = store.read_all_issues()?;

    if let Some(s) = status {
        let s: Status = s.parse().map_err(hbd::HbdError::Other)?;
        issues.retain(|i| i.status == s);
    } else {
        issues.retain(|i| i.status != Status::Closed);
    }
    if let Some(t) = issue_type {
        let t: IssueType = t.parse().map_err(hbd::HbdError::Other)?;
        issues.retain(|i| i.issue_type == t);
    }
    if let Some(p) = priority
        && let Some(p) = Priority::from_u8(p)
    {
        issues.retain(|i| i.priority == p);
    }
    if let Some(l) = label {
        let l = l.trim().to_lowercase();
        issues.retain(|i| i.labels.iter().any(|lbl| lbl.to_lowercase() == l));
    }
    if let Some(a) = assignee {
        issues.retain(|i| i.assignee.as_deref() == Some(a));
    }

    issues.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| b.created_at.cmp(&a.created_at))
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&issues)?);
    } else {
        if issues.is_empty() {
            println!("No issues found.");
            return Ok(());
        }

        #[derive(Tabled)]
        struct Row {
            #[tabled(rename = "ID")]
            id: String,
            #[tabled(rename = "P")]
            priority: u8,
            #[tabled(rename = "Status")]
            status: String,
            #[tabled(rename = "Type")]
            issue_type: String,
            #[tabled(rename = "Title")]
            title: String,
        }

        let rows: Vec<Row> = issues
            .iter()
            .map(|i| Row {
                id: i.id.clone(),
                priority: i.priority.as_u8(),
                status: i.status.to_string(),
                issue_type: i.issue_type.to_string(),
                title: truncate(&i.title, 40),
            })
            .collect();

        let mut table = Table::new(rows);
        table.with(Style::rounded());
        println!("{table}");
        println!("\n{} issue(s)", issues.len());
    }
    Ok(())
}

pub fn ready(store: &TicketStore, json: bool) -> hbd::Result<()> {
    let issues = store.read_all_issues()?;
    let issues_map = store.read_all_issues_map()?;

    let mut ready: Vec<_> = ready_issues(&issues, &issues_map)
        .into_iter()
        .cloned()
        .collect();

    ready.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| b.created_at.cmp(&a.created_at))
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&ready)?);
    } else {
        if ready.is_empty() {
            println!("No ready issues.");
            return Ok(());
        }

        #[derive(Tabled)]
        struct Row {
            #[tabled(rename = "ID")]
            id: String,
            #[tabled(rename = "P")]
            priority: u8,
            #[tabled(rename = "Type")]
            issue_type: String,
            #[tabled(rename = "Title")]
            title: String,
        }

        let rows: Vec<Row> = ready
            .iter()
            .map(|i| Row {
                id: i.id.clone(),
                priority: i.priority.as_u8(),
                issue_type: i.issue_type.to_string(),
                title: truncate(&i.title, 40),
            })
            .collect();

        println!("Ready issues (no open blockers):\n");
        let mut table = Table::new(rows);
        table.with(Style::rounded());
        println!("{table}");
        println!("\n{} issue(s) ready", ready.len());
    }
    Ok(())
}

pub fn blocked(store: &TicketStore, json: bool) -> hbd::Result<()> {
    let issues = store.read_all_issues()?;
    let issues_map = store.read_all_issues_map()?;

    let blocked: Vec<_> = blocked_issues(&issues, &issues_map)
        .into_iter()
        .cloned()
        .collect();

    if json {
        let result: Vec<_> = blocked
            .iter()
            .map(|i| {
                let blockers: Vec<_> = i
                    .depends_on
                    .iter()
                    .filter_map(|d| issues_map.get(&d.id))
                    .filter(|b| b.status != Status::Closed)
                    .map(|b| serde_json::json!({"id": b.id, "title": b.title, "status": b.status.as_str()}))
                    .collect();
                serde_json::json!({
                    "issue": {"id": i.id, "title": i.title},
                    "blocked_by": blockers
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if blocked.is_empty() {
            println!("No blocked issues.");
            return Ok(());
        }
        println!("Blocked issues:\n");
        for i in &blocked {
            println!("{} {}", i.id, truncate(&i.title, 40));
            for dep in &i.depends_on {
                if let Some(blocker) = issues_map.get(&dep.id)
                    && blocker.status != Status::Closed
                {
                    println!("  blocked by: {} ({})", blocker.id, blocker.status);
                }
            }
            println!();
        }
    }
    Ok(())
}

pub fn stale(
    store: &TicketStore,
    days: u32,
    status: Option<&str>,
    limit: Option<usize>,
    json: bool,
) -> hbd::Result<()> {
    let issues = store.read_all_issues()?;
    let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(days));

    let mut issues = stale_issues(issues, cutoff);

    if let Some(s) = status {
        let s: Status = s.parse().map_err(hbd::HbdError::Other)?;
        issues.retain(|i| i.status == s);
    }

    issues.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));

    if let Some(limit) = limit {
        issues.truncate(limit);
    }

    if json {
        let result: Vec<_> = issues
            .iter()
            .map(|i| {
                serde_json::json!({
                    "id": i.id,
                    "title": i.title,
                    "status": i.status.as_str(),
                    "updated_at": i.updated_at.to_rfc3339(),
                    "days_stale": (chrono::Utc::now() - i.updated_at).num_days()
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if issues.is_empty() {
        println!("No stale issues (older than {days} days)");
    } else {
        #[derive(Tabled)]
        struct Row {
            #[tabled(rename = "ID")]
            id: String,
            #[tabled(rename = "Status")]
            status: String,
            #[tabled(rename = "Days")]
            days_stale: i64,
            #[tabled(rename = "Title")]
            title: String,
        }

        let rows: Vec<Row> = issues
            .iter()
            .map(|i| Row {
                id: i.id.clone(),
                status: i.status.to_string(),
                days_stale: (chrono::Utc::now() - i.updated_at).num_days(),
                title: truncate(&i.title, 40),
            })
            .collect();

        println!("Stale issues (not updated in {days}+ days):\n");
        let mut table = Table::new(rows);
        table.with(Style::rounded());
        println!("{table}");
        println!("\n{} stale issue(s)", issues.len());
    }
    Ok(())
}

pub fn stats(store: &TicketStore, json: bool) -> hbd::Result<()> {
    let issues = store.read_all_issues()?;

    let mut by_status: HashMap<String, usize> = HashMap::new();
    let mut by_type: HashMap<String, usize> = HashMap::new();
    let mut by_priority: HashMap<u8, usize> = HashMap::new();

    let week_ago = chrono::Utc::now() - chrono::Duration::days(7);
    let mut created_this_week = 0usize;
    let mut closed_this_week = 0usize;

    for issue in &issues {
        *by_status
            .entry(issue.status.as_str().to_string())
            .or_insert(0) += 1;
        *by_type
            .entry(issue.issue_type.as_str().to_string())
            .or_insert(0) += 1;
        *by_priority.entry(issue.priority.as_u8()).or_insert(0) += 1;

        if issue.created_at >= week_ago {
            created_this_week += 1;
        }
        if let Some(closed_at) = issue.closed_at
            && closed_at >= week_ago
        {
            closed_this_week += 1;
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    let net_change = created_this_week as i64 - closed_this_week as i64;

    if json {
        let result = serde_json::json!({
            "total": issues.len(),
            "by_status": by_status,
            "by_type": by_type,
            "by_priority": by_priority,
            "weekly_trends": {
                "created": created_this_week,
                "closed": closed_this_week,
                "net_change": net_change
            }
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Issue Statistics\n");
        println!("Total: {}\n", issues.len());

        println!("By Status:");
        for (status, count) in &by_status {
            println!("  {status:<12} {count}");
        }

        println!("\nBy Type:");
        for (t, count) in &by_type {
            println!("  {t:<12} {count}");
        }

        println!("\nBy Priority:");
        let mut priorities: Vec<_> = by_priority.iter().collect();
        priorities.sort_by_key(|(p, _)| *p);
        for (p, count) in priorities {
            let label = Priority::from_u8(*p).map_or("?", Priority::label);
            println!("  P{p} ({label:<8}) {count}");
        }

        println!("\nThis Week:");
        println!("  Created:    {created_this_week}");
        println!("  Closed:     {closed_this_week}");
        let sign = if net_change >= 0 { "+" } else { "" };
        println!("  Net change: {sign}{net_change}");
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
