use std::fmt::Display;

/// Potential errors when making a search.
#[derive(Debug)]
pub enum SearchError {
    /// The underlying request has failed.
    RequestFailed,

    /// The response text is empty or missing.
    NoResponseText,

    /// No search results could be found. This could indicate a captcha gate,
    /// or a change to how the engine structures search results.
    NoResultsFound,

    /// Temporarily blocked from accessing the search engine, potentially due to a captcha.
    Blocked,

    /// Temporarily blocked from accessing the search engine with a captcha.
    Captcha,

    /// The engine doesn't support the requested safe search level.
    SafeSearchRestriction,

    /// Error reason is unknown (use sparingly).
    Unknown,
}

impl SearchError {
    /// Fetches a user-friendly reason for the error.
    pub fn reason(&self) -> &'static str {
        match self {
            Self::RequestFailed => "The underlying request has failed.",
            Self::NoResponseText => "The response text is empty or missing.",
            Self::NoResultsFound => {
                "No search results could be found. This could indicate a captcha gate, or a change to how the engine structures search results."
            }
            Self::Blocked => {
                "Temporarily blocked from accessing the search engine, potentially due to a captcha."
            }
            Self::Captcha => "Temporarily blocked from accessing the search engine with a captcha.",
            Self::SafeSearchRestriction => {
                "The engine doesn't support the requested safe search level."
            }
            Self::Unknown => "Error reason is unknown",
        }
    }
}

impl Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?} - {}", self, self.reason())
    }
}
