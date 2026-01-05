//! Graph algorithms for dependency cycle detection and blocker tree construction.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::Serialize;

use crate::types::{DepType, Issue, Status};

#[derive(Debug, Clone, Serialize)]
pub struct BlockerNode {
    pub id: String,
    pub title: String,
    pub status: String,
    pub depth: usize,
    pub blockers: Vec<Self>,
}

/// Returns `true` if adding `from_id blocks to_id` would create a cycle.
pub fn would_create_cycle(issues: &HashMap<String, Issue>, from_id: &str, to_id: &str) -> bool {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(to_id.to_string());

    while let Some(current) = queue.pop_front() {
        if current == from_id {
            return true;
        }
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());

        if let Some(issue) = issues.get(&current) {
            for dep in &issue.depends_on {
                if dep.dep_type == DepType::Blocks && !visited.contains(&dep.id) {
                    queue.push_back(dep.id.clone());
                }
            }
        }
    }
    false
}

pub fn find_all_cycles(issues: &HashMap<String, Issue>) -> Vec<Vec<String>> {
    fn dfs(
        node: &str,
        issues: &HashMap<String, Issue>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if rec_stack.contains(node) {
            if let Some(pos) = path.iter().position(|n| n == node) {
                let cycle: Vec<_> = path[pos..].to_vec();
                cycles.push(cycle);
            }
            return;
        }
        if visited.contains(node) {
            return;
        }

        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(issue) = issues.get(node) {
            for dep in &issue.depends_on {
                if dep.dep_type == DepType::Blocks {
                    dfs(&dep.id, issues, visited, rec_stack, path, cycles);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();

    for id in issues.keys() {
        if !visited.contains(id) {
            dfs(
                id,
                issues,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
            );
        }
    }

    cycles
}

pub fn build_blocker_tree(issue_id: &str, issues: &HashMap<String, Issue>) -> Option<BlockerNode> {
    fn build_node(
        issue_id: &str,
        issues: &HashMap<String, Issue>,
        visited: &mut HashSet<String>,
        depth: usize,
    ) -> Option<BlockerNode> {
        if visited.contains(issue_id) {
            return None;
        }
        visited.insert(issue_id.to_string());

        let issue = issues.get(issue_id)?;
        let blockers: Vec<_> = issue
            .depends_on
            .iter()
            .filter(|d| d.dep_type == DepType::Blocks)
            .filter_map(|d| build_node(&d.id, issues, visited, depth + 1))
            .collect();

        Some(BlockerNode {
            id: issue_id.to_string(),
            title: issue.title.clone(),
            status: issue.status.as_str().to_string(),
            depth,
            blockers,
        })
    }

    let mut visited = HashSet::new();
    build_node(issue_id, issues, &mut visited, 0)
}

pub fn count_open_blockers(node: &BlockerNode) -> usize {
    fn count_children(blockers: &[BlockerNode]) -> usize {
        blockers
            .iter()
            .map(|n| {
                let self_open = usize::from(n.status != "closed");
                self_open + count_children(&n.blockers)
            })
            .sum()
    }
    count_children(&node.blockers)
}

pub fn find_open_children<'a>(
    parent_id: &str,
    issues: &'a HashMap<String, Issue>,
) -> Vec<&'a Issue> {
    issues
        .values()
        .filter(|i| i.parent_id.as_deref() == Some(parent_id) && i.status != Status::Closed)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Dependency, IssueType, Priority};
    use chrono::Utc;

    fn make_issue(id: &str, depends_on: Vec<(&str, DepType)>) -> Issue {
        Issue {
            id: id.to_string(),
            title: format!("Issue {id}"),
            body: String::new(),
            status: Status::Open,
            priority: Priority::Medium,
            issue_type: IssueType::Task,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
            created_by: "test".to_string(),
            created_by_type: crate::types::CreatorType::Human,
            assignee: None,
            agent_id: None,
            session_id: None,
            external_ref: None,
            parent_id: None,
            labels: vec![],
            depends_on: depends_on
                .into_iter()
                .map(|(dep_id, dep_type)| Dependency {
                    id: dep_id.to_string(),
                    dep_type,
                })
                .collect(),
            comments: vec![],
            estimated_minutes: None,
            content_hash: None,
        }
    }

    #[test]
    fn detects_cycle_in_a_blocks_b_blocks_c_blocks_a() {
        let mut issues = HashMap::new();
        issues.insert("A".into(), make_issue("A", vec![("B", DepType::Blocks)]));
        issues.insert("B".into(), make_issue("B", vec![("C", DepType::Blocks)]));
        issues.insert("C".into(), make_issue("C", vec![("A", DepType::Blocks)]));

        assert!(would_create_cycle(&issues, "A", "C"));
    }

    #[test]
    fn no_false_positive_when_no_path_exists() {
        let mut issues = HashMap::new();
        issues.insert("A".into(), make_issue("A", vec![("B", DepType::Blocks)]));
        issues.insert("B".into(), make_issue("B", vec![]));
        issues.insert("C".into(), make_issue("C", vec![]));

        assert!(!would_create_cycle(&issues, "C", "A"));
    }

    #[test]
    fn detects_cycle_through_diamond_path() {
        let mut issues = HashMap::new();
        issues.insert(
            "A".into(),
            make_issue("A", vec![("B", DepType::Blocks), ("C", DepType::Blocks)]),
        );
        issues.insert("B".into(), make_issue("B", vec![("D", DepType::Blocks)]));
        issues.insert("C".into(), make_issue("C", vec![("D", DepType::Blocks)]));
        issues.insert("D".into(), make_issue("D", vec![]));

        assert!(would_create_cycle(&issues, "D", "A"));
    }

    #[test]
    fn find_cycles_detects_mutual_blocking() {
        let mut issues = HashMap::new();
        issues.insert("A".into(), make_issue("A", vec![("B", DepType::Blocks)]));
        issues.insert("B".into(), make_issue("B", vec![("A", DepType::Blocks)]));

        let cycles = find_all_cycles(&issues);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn blocker_tree_includes_transitive_dependencies() {
        let mut issues = HashMap::new();
        issues.insert("A".into(), make_issue("A", vec![("B", DepType::Blocks)]));
        issues.insert("B".into(), make_issue("B", vec![("C", DepType::Blocks)]));
        issues.insert("C".into(), make_issue("C", vec![]));

        let tree = build_blocker_tree("A", &issues).unwrap();
        assert_eq!(tree.id, "A");
        assert_eq!(tree.blockers.len(), 1);
        assert_eq!(tree.blockers[0].id, "B");
        assert_eq!(tree.blockers[0].blockers.len(), 1);
        assert_eq!(tree.blockers[0].blockers[0].id, "C");
    }

    #[test]
    fn count_open_blockers_excludes_closed_issues() {
        let mut issues = HashMap::new();
        issues.insert("A".into(), make_issue("A", vec![("B", DepType::Blocks)]));

        let mut b = make_issue("B", vec![]);
        b.status = Status::Closed;
        issues.insert("B".into(), b);

        let tree = build_blocker_tree("A", &issues).unwrap();
        assert_eq!(count_open_blockers(&tree), 0);
    }
}
