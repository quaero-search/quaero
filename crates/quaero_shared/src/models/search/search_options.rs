use std::sync::Arc;

use crate::models::search::{DateTimeRange, SafeSearch};

#[derive(Default)]
/// Options for customizing a search query.
pub struct SearchOptions {
    /// The page number each engine will fetch data from.
    pub page_num: usize,

    /// How explicit the search results are allowed to be.
    pub safe_search: SafeSearch,

    /// Restricts results to ones created between the specified range.
    pub date_time_range: Option<DateTimeRange>,
}

impl SearchOptions {
    /// Sets the page number for the search query.
    pub fn page_num(mut self, page_num: usize) -> Self {
        self.page_num = page_num;
        self
    }

    /// Sets the safe search for the search query.
    pub fn safe_search(mut self, safe_search: impl Into<SafeSearch>) -> Self {
        self.safe_search = safe_search.into();
        self
    }

    /// Sets the date time range for the search query.
    /// Engines which don't support custom ranges will pick it's nearest supported preset instead.
    pub fn date_time_range(mut self, range: impl Into<DateTimeRange>) -> Self {
        self.date_time_range = Some(range.into());
        self
    }

    /// Convenience helper to wrap the SearchOptions with an Arc.
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}
