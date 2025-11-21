use std::borrow::Cow;

use anyhttp::{
    HeaderMap, HeaderValue,
    header::{ACCEPT, REFERER, USER_AGENT},
};
use chrono::Duration;
use html_hybrid_parser::{ClassName, Node, Query, QueryClassNames, class_names_any};

use quaero_shared::models::{
    engine::{Engine, TaggedEngine},
    search::{SafeSearch, SearchError, SearchOptions, SearchResult},
    user_agent::UserAgent,
};

/// An engine which parses search results from Yahoo.
pub struct YahooEngine;

impl YahooEngine {
    /// Creates a new Yahoo engine.
    pub fn new() -> TaggedEngine {
        TaggedEngine::new(Self {})
    }
}

#[async_trait::async_trait]
impl Engine for YahooEngine {
    fn homepage(&self) -> &'static str {
        "https://search.yahoo.com/search"
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
        let results_per_page = 10;
        let page_start_idx = results_per_page * page_num + 1;

        let safe_search_param = match safe_search {
            SafeSearch::Off => "&v=1",
            SafeSearch::Moderate => "&vm=p",
            SafeSearch::Strict => "&vm=r",
        };

        // Yahoo's search engine doesn't support custom time range filtering.
        // So we need to find the closest preset to our range.
        let date_time_range_param = if let Some(date_time_range) = date_time_range {
            let date_time_range = date_time_range.find_closest_preset(&DATE_TIME_PRESETS);
            Cow::Owned(format!("&btf={date_time_range}"))
        } else {
            Cow::Borrowed("")
        };

        Ok(format!(
            "https://search.yahoo.com/search?p={query}&b={page_start_idx}{safe_search_param}{date_time_range_param}&nocache=1&nojs=1"
        ))
    }

    fn headers(&self, headers: &mut HeaderMap, _options: &SearchOptions) {
        headers.insert(USER_AGENT, UserAgent::random_no_js().into());
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

        let nodes = results
            .get_nodes_with_classes(&SEARCH_RESULT_CLASSES, parser)
            // Removes any nodes which:
            // - Have the `AlsoTry_M` class (search suggestions).
            .filter(|this| !SEARCH_RESULT_BLOCKLISTED_CLASSES.matches(this.class()));

        Ok(nodes
            .filter_map(|this| {
                let Some(title_node) = this.get_first_node_with_classes(&TITLE_CLASSES, parser)
                else {
                    return None;
                };

                let title = title_node
                    .children_raw_text(parser)
                    .map(|this| this.to_string())
                    .unwrap_or_default();

                let url = title_node
                    .get_href()
                    .map(|this| clean_url(this.to_string()))
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

fn clean_url(input_url: String) -> String {
    let Some(start_idx) = input_url.find("RU=") else {
        return input_url;
    };
    let Some(end_idx) = input_url.find("RK=2") else {
        return input_url;
    };

    input_url[start_idx + 3..=end_idx - 1].to_string()
}

const SEARCH_RESULTS_WRAPPER_CLASSES: ClassName = class_names_any! { "searchCenterMiddle" };

const SEARCH_RESULT_CLASSES: ClassName = class_names_any! { "dd" };
const SEARCH_RESULT_BLOCKLISTED_CLASSES: ClassName = class_names_any! { "AlsoTry_M" };

const TITLE_CLASSES: ClassName = class_names_any! { "s-title" };

const SUMMARY_CLASSES: ClassName = class_names_any! { "s-desc" };

const DATE_TIME_PRESETS: [(Duration, &'static str); 3] = [
    (Duration::hours(24), "d"),
    (Duration::weeks(1), "w"),
    (Duration::days(30), "m"),
];
