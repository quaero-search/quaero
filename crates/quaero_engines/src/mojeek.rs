use std::borrow::Cow;

use chrono::Datelike;
use html_hybrid_parser::{ClassName, Node, Query, class_names_any};
use http::{
    HeaderMap, HeaderValue,
    header::{ACCEPT, REFERER, USER_AGENT},
};

use quaero_shared::models::{
    engine::{Engine, TaggedEngine},
    search::{DateTimeRange, SearchError, SearchOptions, SearchResult},
    user_agent::UserAgent,
};

/// An engine which parses search results from Mojeek.
pub struct MojeekEngine;

impl MojeekEngine {
    /// Creates a new Mojeek engine.
    pub fn new() -> TaggedEngine {
        TaggedEngine::new(Self {})
    }
}

#[async_trait::async_trait]
impl Engine for MojeekEngine {
    fn homepage(&self) -> &'static str {
        "https://www.mojeek.com"
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
        // Page 0 is `1`, Page 1 is `11`, Page 2 is `21`, etc...
        let page_start_idx_param = match page_num {
            0 => Cow::Borrowed(""),
            _ => {
                let results_per_page = 10;
                let page_start_idx = results_per_page * page_num + 1;
                Cow::Owned(format!("&s={page_start_idx}"))
            }
        };

        let safe_search_param = match safe_search.as_bool() {
            true => Cow::Owned(format!("&safe=1")),
            false => Cow::Borrowed(""),
        };

        let date_time_range_query_param = if let Some(DateTimeRange {
            start: start_range,
            end: end_range,
        }) = date_time_range
        {
            let start_range_str = format!(
                "since%3A{:04}{:02}{:02}",
                start_range.year(),
                start_range.month(),
                start_range.day()
            );
            let end_range_str = format!(
                "before%3A{:04}{:02}{:02}",
                end_range.year(),
                end_range.month(),
                end_range.day()
            );
            Cow::Owned(format!("%20{start_range_str}%20{end_range_str}"))
        } else {
            Cow::Borrowed("")
        };

        Ok(format!(
            "https://www.mojeek.com/search?q={query}{date_time_range_query_param}{page_start_idx_param}{safe_search_param}"
        ))
    }

    fn headers(&self, headers: &mut HeaderMap, _options: &SearchOptions) {
        headers.insert(USER_AGENT, UserAgent::random().into());
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );
        headers.append(REFERER, HeaderValue::from_static("https://google.com/"));
    }

    fn parse<'a>(&self, response_text: String) -> Result<Vec<(String, SearchResult)>, SearchError> {
        let dom = html_hybrid_parser::Parser::fast_but_constrained(&response_text);
        let parser = dom.parser();

        let Some(node) = dom.get_first_node_with_classes(&SEARCH_RESULT_WRAPPER_CLASSES, parser)
        else {
            return Err(SearchError::NoResultsFound);
        };

        Ok(node
            .get_child_nodes(parser)
            .filter_map(|this| {
                let Some(title_node_outer) = this.get_first_child_node_with_tag("h2", parser)
                else {
                    return None;
                };

                let Some(title_node) =
                    title_node_outer.get_first_child_node_with_classes(&TITLE_CLASSES, parser)
                else {
                    return None;
                };

                let title = title_node
                    .text(parser)
                    .map(|this| this.to_string())
                    .unwrap_or_default();

                let url = title_node
                    .get_href()
                    .map(|this| this.to_string())
                    .unwrap_or_default();

                let summary = this
                    .get_first_child_node_with_classes(&SUMMARY_CLASSES, parser)
                    .and_then(|this| this.text(parser).map(|this| this.to_string()))
                    .unwrap_or_default();

                Some(SearchResult::new(title, url, summary))
            })
            .collect())
    }
}

const SEARCH_RESULT_WRAPPER_CLASSES: ClassName = class_names_any! { "results-standard" };

const TITLE_CLASSES: ClassName = class_names_any! { "title" };

const SUMMARY_CLASSES: ClassName = class_names_any! { "s" };
