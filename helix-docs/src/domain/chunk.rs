use serde::{Deserialize, Serialize};

use super::{ChunkId, DocId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: ChunkId,
    pub doc_id: DocId,
    pub text: String,
    pub position: ChunkPosition,
    pub metadata: ChunkMetadata,
}

impl Chunk {
    pub fn new(doc_id: DocId, text: String, position: ChunkPosition) -> Self {
        Self {
            id: ChunkId::generate(),
            doc_id,
            text,
            position,
            metadata: ChunkMetadata::default(),
        }
    }

    #[must_use]
    pub fn with_metadata(mut self, metadata: ChunkMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkPosition {
    pub index: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub section_title: Option<String>,
    pub language: Option<String>,
    pub chunk_type: ChunkType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChunkType {
    #[default]
    Prose,
    CodeBlock,
    Heading,
    List,
}
