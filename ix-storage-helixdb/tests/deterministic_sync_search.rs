use ix_core::entity::EntityKind;
use ix_core::index::IndexBackend;
use ix_core::markdown::{parse_markdown, render_markdown};
use ix_core::repo::IxchelRepo;
use ix_embeddings::{Embedder, EmbeddingProvider};
use tempfile::TempDir;

struct HashEmbeddingProvider {
    dimension: usize,
    model_name: String,
}

impl HashEmbeddingProvider {
    fn new(dimension: usize) -> Self {
        Self {
            dimension,
            model_name: "hash-v1".to_string(),
        }
    }

    fn embed_impl(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; self.dimension];

        for token in text
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|t| !t.is_empty())
        {
            let token = token.to_ascii_lowercase();
            let hash = blake3::hash(token.as_bytes());
            let idx = usize::from(hash.as_bytes()[0]) % self.dimension;
            vector[idx] += 1.0;
        }

        let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vector {
                *v /= norm;
            }
        }

        vector
    }
}

impl EmbeddingProvider for HashEmbeddingProvider {
    fn embed(&self, text: &str) -> ix_embeddings::Result<Vec<f32>> {
        Ok(self.embed_impl(text))
    }

    fn embed_batch(&self, texts: &[&str]) -> ix_embeddings::Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|t| self.embed_impl(t)).collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn provider_name(&self) -> &'static str {
        "hash"
    }
}

fn replace_entity_body(repo: &IxchelRepo, id: &str, body: &str) {
    let path = repo.paths.entity_path(id).expect("entity path");
    let raw = std::fs::read_to_string(&path).expect("read entity");
    let mut doc = parse_markdown(&path, &raw).expect("parse markdown");
    doc.body = body.to_string();
    let out = render_markdown(&doc).expect("render markdown");
    std::fs::write(&path, out).expect("write entity");
}

#[test]
fn deterministic_sync_indexes_graph_and_vectors() {
    let temp = TempDir::new().expect("tempdir");
    let repo = IxchelRepo::init_at(temp.path(), false).expect("init ixchel repo");

    let alpha = repo
        .create_entity(EntityKind::Source, "Alpha Source", None)
        .expect("create alpha source");
    replace_entity_body(&repo, &alpha.id, "alpha");

    let beta = repo
        .create_entity(EntityKind::Source, "Beta Source", None)
        .expect("create beta source");
    replace_entity_body(&repo, &beta.id, "beta");

    let decision = repo
        .create_entity(EntityKind::Decision, "Decision One", Some("accepted"))
        .expect("create decision");
    replace_entity_body(&repo, &decision.id, "decision");

    let issue = repo
        .create_entity(EntityKind::Issue, "Issue One", Some("open"))
        .expect("create issue");
    replace_entity_body(&repo, &issue.id, "issue");
    repo.link(&issue.id, "implements", &decision.id)
        .expect("link issue implements decision");

    let embedder = Embedder::from_provider(Box::new(HashEmbeddingProvider::new(32)));
    let mut index =
        ix_storage_helixdb::HelixDbIndex::open_with_embedder(&repo, embedder).expect("open index");

    let stats = index.sync(&repo).expect("sync");
    assert_eq!(stats.scanned, 4);
    assert_eq!(stats.added, 4);

    let hits = index.search("alpha", 5).expect("search alpha");
    assert!(!hits.is_empty(), "expected search hits");
    assert_eq!(hits[0].id, alpha.id, "{hits:#?}");

    let hits = index.search("beta", 5).expect("search beta");
    assert!(!hits.is_empty(), "expected search hits");
    assert_eq!(hits[0].id, beta.id, "{hits:#?}");

    let outgoing = index
        .outgoing(&issue.id, "implements")
        .expect("outgoing implements");
    assert_eq!(outgoing, vec![decision.id]);
}
