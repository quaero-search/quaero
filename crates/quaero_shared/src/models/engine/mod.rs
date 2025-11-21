mod engine_id;
use std::{any::type_name_of_val, sync::Arc};

use anyhttp::{HeaderMap, HeaderValue, Response};
pub use engine_id::*;
use smallvec::SmallVec;

use crate::models::search::{SearchError, SearchOptions, SearchResult};

/// Defines the interface for a search engine.
#[async_trait::async_trait]
pub trait Engine: Send + Sync {
    /// Returns the name of the engine.
    fn name(&self) -> String {
        let name = type_name_of_val(&*self)
            .rsplit("::")
            .next()
            .unwrap_or_default();

        let name = name.strip_suffix("Engine").unwrap_or(name);

        name.chars()
            .enumerate()
            .fold(String::new(), |mut acc, (i, ch)| {
                if ch.is_uppercase() && i > 0 {
                    if let Some(prev) = name.chars().nth(i - 1) {
                        if !prev.is_uppercase() {
                            acc.push(' ');
                        }
                    }
                }
                acc.push(ch);
                acc
            })
    }

    /// Gets the homepage url for this engine.
    fn homepage(&self) -> &'static str;

    /// Returns the url for a particular query.
    fn url(&self, query: &str, options: &SearchOptions) -> Result<String, SearchError>;

    /// Returns the headers for a particular query.
    fn headers(&self, _headers: &mut HeaderMap<HeaderValue>, _options: &SearchOptions) {}

    /// Validates that the response is valid.
    fn validate_response(&self, _response: &Response<Vec<u8>>) -> Result<(), SearchError> {
        Ok(())
    }

    /// Parses search results from the response text.
    fn parse<'a>(&self, response_text: String) -> Result<Vec<(String, SearchResult)>, SearchError>;
}

/// An engine tagged with a unique identifier.
/// Used to identify multiple of the same engine from each other.
pub struct TaggedEngine(pub EngineId, pub Arc<dyn Engine>);

impl TaggedEngine {
    /// Creates a new tagged engine from an engine.
    pub fn new(engine: impl Engine + 'static) -> Self {
        Self(EngineId::from_name(engine.name()), Arc::new(engine))
    }
}

impl Into<SmallVec<[TaggedEngine; 1]>> for TaggedEngine {
    fn into(self) -> SmallVec<[TaggedEngine; 1]> {
        SmallVec::from_const([self])
    }
}
