use thiserror::Error;

/// Potential errors when making a search.
#[derive(Debug, Error)]
pub enum SearchError {
    /// The underlying request has failed.
    #[error("The underlying request has failed.")]
    RequestFailed,

    /// The response text is empty or missing.
    #[error("The response text is empty or missing.")]
    NoResponseText,

    /// No search results could be found. This could indicate a captcha gate,
    /// or a change to how the engine structures search results.
    #[error(
        "No search results could be found. This could indicate a captcha gate, or a change to how the engine structures search results."
    )]
    NoResultsFound,

    /// Temporarily blocked from accessing the search engine, potentially due to a captcha.
    #[error("Temporarily blocked from accessing the search engine, potentially due to a captcha.")]
    Blocked,

    /// Temporarily blocked from accessing the search engine with a captcha.
    #[error("Temporarily blocked from accessing the search engine with a captcha.")]
    Captcha,

    /// The engine doesn't support the requested safe search level.
    #[error("The engine doesn't support the requested safe search level.")]
    SafeSearchRestriction,

    /// Error reason is unknown (use sparingly).
    #[error("Error reason is unknown.")]
    Unknown,

    /// Could not fetch results within the allowed time limit.
    #[error("Could not fetch results within the allowed time limit.")]
    Timeout,
}
