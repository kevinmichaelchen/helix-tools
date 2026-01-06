//! Storage abstraction for decision indexing.

use crate::types::{ChainNode, Decision, RelatedDecision, RelationType};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub trait DecisionStorage: Send + Sync {
    fn index(&mut self, decisions: Vec<Decision>) -> Result<()>;
    fn remove(&mut self, paths: Vec<String>) -> Result<()>;
    fn search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Decision, f32)>>;
    fn get_hashes(&self) -> Result<HashMap<String, String>>;
    fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>>;
    fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>>;
}

pub struct HelixDBStorage {
    decisions: Vec<Decision>,
    hashes: HashMap<String, String>,
    decision_id_to_idx: HashMap<u32, usize>,
}

impl HelixDBStorage {
    pub fn open() -> Result<Self> {
        Ok(Self {
            decisions: Vec::new(),
            hashes: HashMap::new(),
            decision_id_to_idx: HashMap::new(),
        })
    }

    #[allow(dead_code)]
    pub fn open_at(_path: &Path) -> Result<Self> {
        Self::open()
    }

    fn rebuild_id_map(&mut self) {
        self.decision_id_to_idx.clear();
        for (idx, decision) in self.decisions.iter().enumerate() {
            self.decision_id_to_idx.insert(decision.metadata.id, idx);
        }
    }
}

impl DecisionStorage for HelixDBStorage {
    fn index(&mut self, decisions: Vec<Decision>) -> Result<()> {
        for decision in decisions {
            let path = decision.file_path.to_string_lossy().to_string();
            self.hashes.insert(path, decision.content_hash.clone());

            if let Some(idx) = self.decision_id_to_idx.get(&decision.metadata.id) {
                self.decisions[*idx] = decision;
            } else {
                let idx = self.decisions.len();
                self.decision_id_to_idx.insert(decision.metadata.id, idx);
                self.decisions.push(decision);
            }
        }
        Ok(())
    }

    fn remove(&mut self, paths: Vec<String>) -> Result<()> {
        for path in &paths {
            self.hashes.remove(path);
            self.decisions
                .retain(|d| d.file_path.to_string_lossy() != *path);
        }
        self.rebuild_id_map();
        Ok(())
    }

    fn search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Decision, f32)>> {
        let mut results: Vec<(Decision, f32)> = self
            .decisions
            .iter()
            .filter_map(|decision| {
                decision.embedding.as_ref().map(|emb| {
                    let score = cosine_similarity(&embedding, emb);
                    (decision.clone(), score)
                })
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    fn get_hashes(&self) -> Result<HashMap<String, String>> {
        Ok(self.hashes.clone())
    }

    fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>> {
        let mut chain = Vec::new();
        let mut current_id = Some(decision_id);
        let mut visited = std::collections::HashSet::new();

        while let Some(id) = current_id {
            if visited.contains(&id) {
                break;
            }
            visited.insert(id);

            if let Some(&idx) = self.decision_id_to_idx.get(&id) {
                let decision = &self.decisions[idx];

                let superseded_ids: Vec<u32> = decision
                    .metadata
                    .supersedes
                    .as_ref()
                    .map(|s| s.to_vec())
                    .unwrap_or_default();

                chain.push(ChainNode {
                    id: decision.metadata.id,
                    title: decision.metadata.title.clone(),
                    status: decision.metadata.status.clone(),
                    date: decision.metadata.date,
                    is_current: false,
                });

                current_id = superseded_ids.first().copied();
            } else {
                break;
            }
        }

        if let Some(last) = chain.last_mut() {
            last.is_current = true;
        }

        Ok(chain)
    }

    fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>> {
        let mut related = Vec::new();

        let Some(&idx) = self.decision_id_to_idx.get(&decision_id) else {
            return Ok(related);
        };

        let decision = &self.decisions[idx];

        let add_relations = |ids: &Option<crate::types::OneOrMany<u32>>,
                             rel_type: RelationType,
                             related: &mut Vec<RelatedDecision>,
                             decisions: &[Decision],
                             id_map: &HashMap<u32, usize>| {
            if let Some(ids) = ids {
                for target_id in ids.to_vec() {
                    if let Some(&target_idx) = id_map.get(&target_id) {
                        related.push(RelatedDecision {
                            id: target_id,
                            title: decisions[target_idx].metadata.title.clone(),
                            relation: rel_type,
                        });
                    }
                }
            }
        };

        add_relations(
            &decision.metadata.supersedes,
            RelationType::Supersedes,
            &mut related,
            &self.decisions,
            &self.decision_id_to_idx,
        );
        add_relations(
            &decision.metadata.amends,
            RelationType::Amends,
            &mut related,
            &self.decisions,
            &self.decision_id_to_idx,
        );
        add_relations(
            &decision.metadata.depends_on,
            RelationType::DependsOn,
            &mut related,
            &self.decisions,
            &self.decision_id_to_idx,
        );
        add_relations(
            &decision.metadata.related_to,
            RelationType::RelatedTo,
            &mut related,
            &self.decisions,
            &self.decision_id_to_idx,
        );

        for (other_idx, other_decision) in self.decisions.iter().enumerate() {
            if other_idx == idx {
                continue;
            }

            if other_decision
                .metadata
                .supersedes
                .as_ref()
                .is_some_and(|s| s.to_vec().contains(&decision_id))
            {
                related.push(RelatedDecision {
                    id: other_decision.metadata.id,
                    title: other_decision.metadata.title.clone(),
                    relation: RelationType::Supersedes,
                });
            }
        }

        Ok(related)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);
    }
}
