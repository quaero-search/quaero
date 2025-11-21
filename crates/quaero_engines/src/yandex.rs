use std::borrow::Cow;

use anyhttp::{
    HeaderMap, HeaderValue, Response,
    header::{ACCEPT, REFERER, USER_AGENT},
};
use chrono::Datelike;
use html_hybrid_parser::{ClassName, Node, Query, class_names_any};

use quaero_shared::models::{
    engine::{Engine, TaggedEngine},
    search::{DateTimeRange, SearchError, SearchOptions, SearchResult},
    user_agent::UserAgent,
};

/// An engine which parses search results from Yandex.
pub struct YandexEngine;

impl YandexEngine {
    /// Creates a new Yandex engine.
    pub fn new() -> TaggedEngine {
        TaggedEngine::new(Self {})
    }
}

#[async_trait::async_trait]
impl Engine for YandexEngine {
    fn homepage(&self) -> &'static str {
        "https://yandex.com"
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
        if safe_search.as_incrementing_usize() == 2 {
            return Err(SearchError::SafeSearchRestriction);
        }

        let date_time_range_params = if let Some(date_time_range) = date_time_range {
            let DateTimeRange { start, end } = date_time_range;

            Cow::Owned(format!(
                "constraintid=0&within=777&from_day={}&from_month={}&from_year={}&to_day={}&to_month={}&to_year={}",
                start.day(),
                start.month(),
                start.year(),
                end.day(),
                end.month(),
                end.year()
            ))
        } else {
            Cow::Borrowed("")
        };

        Ok(format!(
            "https://yandex.com/search/site/?text={query}&p={page_num}&tmpl_version=releases&web=1&frame=1&searchid={SEARCH_ID}&{date_time_range_params}"
        ))
    }

    fn validate_response(&self, response: &Response<Vec<u8>>) -> Result<(), SearchError> {
        if response.url().path().starts_with("/showcaptcha") {
            Err(SearchError::Captcha)
        } else {
            Ok(())
        }
    }

    fn headers(&self, headers: &mut HeaderMap, _options: &SearchOptions) {
        headers.insert(USER_AGENT, UserAgent::random().into());
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers.append(REFERER, HeaderValue::from_static("https://google.com/"));
    }

    fn parse<'a>(&self, response_text: String) -> Result<Vec<(String, SearchResult)>, SearchError> {
        let dom = html_hybrid_parser::Parser::fast_but_constrained(&response_text);
        let parser = dom.parser();

        let Some(results) =
            dom.get_first_node_with_classes(&SEARCH_RESULTS_WRAPPER_CLASSES, parser)
        else {
            return Err(SearchError::NoResultsFound);
        };

        Ok(results
            .get_nodes_with_classes(&SEARCH_RESULT_CLASSES, parser)
            .filter_map(|this| {
                let Some(title_node) = this.get_first_node_with_classes(&TITLE_CLASSES, parser)
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
                    .get_first_node_with_classes(&SUMMARY_CLASSES, parser)
                    .and_then(|this| this.text(parser).map(|this| this.to_string()))
                    .unwrap_or_default();

                Some(SearchResult::new(title, url, summary))
            })
            .collect())
    }
}

// This is the search id from searxng and 4get.
const SEARCH_ID: &str = "3131712";

const SEARCH_RESULTS_WRAPPER_CLASSES: ClassName = class_names_any! { "b-serp-list" };
const SEARCH_RESULT_CLASSES: ClassName = class_names_any! { "b-serp-item" };

const TITLE_CLASSES: ClassName = class_names_any! { "b-serp-item__title-link" };

const SUMMARY_CLASSES: ClassName = class_names_any! { "b-serp-item__text" };
