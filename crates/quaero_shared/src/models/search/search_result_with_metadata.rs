use smallvec::SmallVec;

use crate::models::{engine::EngineId, search::SearchResult};

/// A search result with extra metadata.
#[derive(Debug, Clone)]
pub struct SearchResultWithMetadata {
    /// The underlying search result.
    pub search_result: SearchResult,

    /// The engines this search result was found in.
    pub engines: SmallVec<[EngineId; 1]>,

    /// How relevant this search is to the original query.
    pub relevance_score: f32,
}

impl SearchResultWithMetadata {
    /// Creates a new [SearchResultWithMetadata].
    pub fn new(search_result: SearchResult, engine_id: &EngineId) -> Self {
        Self {
            search_result,
            engines: SmallVec::from([engine_id.clone()]),
            relevance_score: 0.,
        }
    }
}
