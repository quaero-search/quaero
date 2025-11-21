use std::borrow::Cow;

use anyhttp::{
    HeaderMap, HeaderValue,
    header::{ACCEPT, COOKIE, REFERER, USER_AGENT},
};
use chrono::Duration;
use html_hybrid_parser::{ClassNames, Node, Query, class_names_exact};

use quaero_shared::models::{
    engine::{Engine, TaggedEngine},
    sanitized_url::SanitizedUrl,
    search::{SearchError, SearchOptions, SearchResult},
    user_agent::UserAgent,
};

/// An engine which parses search results from Google.
pub struct GoogleEngine;

impl GoogleEngine {
    /// Creates a new Google engine.
    pub fn new() -> TaggedEngine {
        TaggedEngine::new(Self {})
    }
}

#[async_trait::async_trait]
impl Engine for GoogleEngine {
    fn homepage(&self) -> &'static str {
        "https://www.google.com"
    }

    fn url(
        &self,
        query: &str,
        SearchOptions {
            page_num,
            safe_search,
            date_time_range,
            ..
        }: &SearchOptions,
    ) -> Result<String, SearchError> {
        // Turns the page number into the index of the first result.
        // Page 0 is `0`, Page 1 is `10`, Page 2 is `20`, etc...
        let results_per_page = 10;
        let page_start_idx = page_num * results_per_page;

        let safe_search = safe_search.as_lowercase_string();

        // Google's no-js search engine doesn't support custom time range filtering.
        // So we need to find the closest preset to our range.
        let date_time_range_param = if let Some(date_time_range) = date_time_range {
            let date_time_range = date_time_range.find_closest_preset(&DATE_TIME_PRESETS);
            Cow::Owned(format!("&tbs=qdr%3A{date_time_range}"))
        } else {
            Cow::Borrowed("")
        };

        Ok(format!(
            "https://www.google.com/search?q={query}&ie=utf8&oe=utf8&start={page_start_idx}&filter=0&safe={safe_search}&tbs={date_time_range_param}"
        ))
    }

    fn headers(&self, headers: &mut HeaderMap, _options: &SearchOptions) {
        headers.insert(USER_AGENT, UserAgent::random_no_js().into());
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );
        headers.append(COOKIE, HeaderValue::from_static("SOCS=CAESHAgBEhIaAB"));
        headers.append(REFERER, HeaderValue::from_static("https://google.com/"));
    }

    /*fn validate_response(&self, response: &Response<Vec<u8>>) -> Result<(), SearchError> {
        let url = response.url();

        let was_captcha_gated =
            url.host_str() == Some("sorry.google.com") || url.path().starts_with("/sorry");

        if was_captcha_gated {
            Err(SearchError::Captcha)
        } else {
            Ok(())
        }
    }*/

    fn parse<'a>(&self, response_text: String) -> Result<Vec<(String, SearchResult)>, SearchError> {
        let dom = html_hybrid_parser::Parser::fast_but_constrained(&response_text);
        let parser = dom.parser();

        let nodes = dom.get_nodes_with_classes(&SEARCH_RESULT_CLASSES, parser);

        Ok(nodes
            .filter_map(|this| {
                let Some(title_node) = this.get_first_node_with_classes(&TITLE_CLASSES, parser)
                else {
                    return None;
                };

                let title = title_node
                    .get_first_node_with_classes(&TITLE_TEXT_CLASSES, parser)
                    .map(|this| this.inner_text(parser).to_string())
                    .unwrap_or_default();

                let url = title_node
                    .get_first_node_with_tag("a", parser)
                    .and_then(|this| {
                        this.get_href().map(|this| {
                            this.strip_prefix("/url?q=")
                                .unwrap_or(this.as_ref())
                                .to_owned()
                        })
                    })
                    .unwrap_or_default();

                let summary = this
                    .get_first_node_with_classes(&SUMMARY_CLASSES, parser)
                    .and_then(|this| {
                        this.get_first_node_with_classes(&SUMMARY_CLASSES, parser)
                            .and_then(|this| {
                                this.children_raw_text(parser).map(|this| this.to_string())
                            })
                    })
                    .unwrap_or_default();

                let sanitized_url = SanitizedUrl::new(&url, filter_search_param_in_result_url);
                Some(SearchResult::new_from_sanitized_url(
                    title,
                    sanitized_url,
                    summary,
                ))
            })
            .collect())
    }
}

const SEARCH_RESULT_CLASSES: ClassNames = class_names_exact! { "Gx5Zad", "xpd", "EtOod", "pkphOe" };

const TITLE_CLASSES: ClassNames = class_names_exact! { "egMi0", "kCrYT" };
const TITLE_TEXT_CLASSES: ClassNames = class_names_exact! { "ilUpNd", "UFvD1", "aSRlid" };

const SUMMARY_CLASSES: ClassNames = class_names_exact! { "ilUpNd", "H66NU", "aSRlid" };

const DATE_TIME_PRESETS: [(Duration, &'static str); 5] = [
    (Duration::hours(1), "h"),
    (Duration::hours(24), "d"),
    (Duration::weeks(1), "w"),
    (Duration::days(30), "m"),
    (Duration::days(365), "y"),
];

fn filter_search_param_in_result_url(key: &str, _value: &str) -> bool {
    key == "ved" || key == "sa" || key == "usg" || key.starts_with("utm")
}
