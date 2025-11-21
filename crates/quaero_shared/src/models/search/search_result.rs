use crate::models::sanitized_url::SanitizedUrl;

/// Contains data pertaining to an individual search result fetched from a particular engine.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The title of the search result.
    pub title: String,

    /// The url of the search result.
    pub url: String,

    /// The short description / summary typically found underneath the search result.
    pub summary: String,
}

impl SearchResult {
    /// Creates a new search result.
    pub fn new(title: String, url: String, summary: String) -> (String, Self) {
        let sanitized_url = SanitizedUrl::new(&url, |_k, _v| true);

        Self::new_from_sanitized_url(title, sanitized_url, summary)
    }

    /// Creates a new search result using a [SanitizedUrl] instead of a string.
    pub fn new_from_sanitized_url(
        title: String,
        sanitized_url: SanitizedUrl,
        summary: String,
    ) -> (String, Self) {
        (
            sanitized_url.to_strict_string(),
            Self {
                title,
                url: sanitized_url.to_string(),
                summary: summary,
            },
        )
    }

    /// Concatenates the title and summary into a snippet.
    pub fn snippet(&self) -> String {
        format!("{} | {}", self.title, self.summary)
    }
}
