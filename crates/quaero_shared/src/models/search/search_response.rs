use crate::models::{
    engine::EngineId,
    search::{SearchError, SearchResultWithMetadata},
};

/// The response gotten at the end of the search query.
#[derive(Debug)]
pub struct SearchResponse {
    /// The aggregated search results across all engines.
    pub results: Box<[SearchResultWithMetadata]>,

    /// The statuses for each individual engine which specifies
    /// if any issues occured when fetching results.
    pub statuses: Vec<(EngineId, Result<(), SearchError>)>,
}
