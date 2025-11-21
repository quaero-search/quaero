use std::borrow::Cow;

use anyhttp::{
    HeaderMap, HeaderValue,
    header::{ACCEPT, CONTENT_TYPE, COOKIE, REFERER, USER_AGENT},
};
use chrono::Datelike;
use html_hybrid_parser::{ClassName, ClassNames, Node, Query, QueryClassNames, class_names_any};

use quaero_shared::models::{
    engine::{Engine, TaggedEngine},
    search::{DateTimeRange, SearchError, SearchOptions, SearchResult},
    user_agent::UserAgent,
};

/// An engine which parses search results from Brave.
pub struct BraveEngine;

impl BraveEngine {
    /// Creates a new Brave engine.
    pub fn new() -> TaggedEngine {
        TaggedEngine::new(Self {})
    }
}

#[async_trait::async_trait]
impl Engine for BraveEngine {
    fn homepage(&self) -> &'static str {
        "https://search.brave.com"
    }

    fn url(
        &self,
        query: &str,
        SearchOptions {
            page_num,
            date_time_range,
            ..
        }: &SearchOptions,
    ) -> Result<String, SearchError> {
        let date_time_range_param = if let Some(DateTimeRange {
            start: start_range,
            end: end_range,
        }) = date_time_range
        {
            let start_range_str = format!(
                "{}-{}-{}",
                start_range.year(),
                start_range.month(),
                start_range.day()
            );
            let end_range_str = format!(
                "{}-{}-{}",
                end_range.year(),
                end_range.month(),
                end_range.day()
            );
            Cow::Owned(format!("&tf={start_range_str}to{end_range_str}"))
        } else {
            Cow::Borrowed("")
        };

        Ok(format!(
            "https://search.brave.com/search?q={query}&offset={page_num}{date_time_range_param}"
        ))
    }

    fn headers(&self, headers: &mut HeaderMap, SearchOptions { safe_search, .. }: &SearchOptions) {
        let safe_search = safe_search.as_lowercase_string();

        headers.insert(USER_AGENT, UserAgent::random_no_js().into());
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.append(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers.append(
            COOKIE,
            HeaderValue::from_str(&format!("safe_search={safe_search}")).unwrap(),
        );
        headers.append(REFERER, HeaderValue::from_static("https://google.com/"));
    }

    fn parse<'a>(&self, response_text: String) -> Result<Vec<(String, SearchResult)>, SearchError> {
        let decoded_data = html_escape::decode_html_entities(&response_text);

        let dom = html_hybrid_parser::Parser::comprehensive_but_slow(decoded_data.as_ref());
        let parser = dom.parser();

        let Some(results) = dom.get_first_node_with_id("results", parser) else {
            return Err(SearchError::NoResultsFound);
        };

        if results
            .get_first_node_with_id("bad-results-info-banner", parser)
            .is_some()
        {
            return Err(SearchError::NoResultsFound);
        }

        let nodes = results
            .get_child_nodes_with_classes(&SEARCH_RESULT_CLASSES, parser)
            // Removes any nodes which:
            // - Don't have the `[data-type="web"]` attributes (non-web results).
            // TODO: look into extracting data from `standalone` snippets as they do contain useful data.
            // - Have the `.noscript-hide` (hidden and empty data) or `standalone` (non standard web result) classes.
            // - Have the `#search-elsewhere` id (search suggestions).
            // - Have the `#search-ad` id (advertisement).
            .filter(|this| {
                if let Some(data_type_attribute) = this.get_attribute("data-type") {
                    if data_type_attribute.as_ref() != "web" {
                        return false;
                    }
                }

                if SEARCH_RESULT_BLOCKLISTED_CLASSES.matches(this.class()) {
                    return false;
                }

                if let Some(id) = this.id() {
                    let id = id.as_ref();
                    if id == "search_anywhere" || id == "search-ad" {
                        return false;
                    }
                }

                true
            });

        Ok(nodes
            .filter_map(|this| {
                let (title, url) = this
                    .get_first_node_with_tag("a", parser)
                    .map(|this| {
                        let title = this
                            .get_first_node_with_classes(&TITLE_CLASSES, parser)
                            .and_then(|this| this.text(parser).map(|this| this.to_string()))
                            .unwrap_or_default();

                        let url = this
                            .get_href()
                            .map(|this| this.to_string())
                            .unwrap_or_default();

                        (title, url)
                    })
                    .unwrap_or_default();

                let summary = this
                    .get_first_node_with_classes(&SUMMARY_CLASSES, parser)
                    .and_then(|this| this.text(parser).map(|this| this.trim_start().to_string()))
                    // Sometimes summaries may be in a q&a format.
                    .unwrap_or_else(|| {
                        this.get_first_node_with_classes(&SUMMARY_QNA_CLASSES, parser)
                            .and_then(|this| this.text(parser).map(|this| this.to_string()))
                            .unwrap_or_default()
                    });

                Some(SearchResult::new(title, url, summary))
            })
            .collect())
    }
}

const SEARCH_RESULT_CLASSES: ClassName = class_names_any! { "snippet" };
const SEARCH_RESULT_BLOCKLISTED_CLASSES: ClassNames =
    class_names_any! { "noscript-hide", "standalone" };

const TITLE_CLASSES: ClassName = class_names_any! { "title" };

const SUMMARY_CLASSES: ClassName = class_names_any! { "content" };
const SUMMARY_QNA_CLASSES: ClassName = class_names_any! { "inline-qa-answer" };
