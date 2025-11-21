use std::borrow::Cow;

use anyhttp::{
    HeaderMap, HeaderValue,
    header::{ACCEPT, CONTENT_TYPE, COOKIE, REFERER, USER_AGENT},
};
use chrono::{TimeZone, Utc};
use html_hybrid_parser::{ClassName, ClassNames, Node, Query, class_names_any, class_names_exact};

use quaero_shared::models::{
    engine::{Engine, TaggedEngine},
    search::{SearchError, SearchOptions, SearchResult},
    user_agent::UserAgent,
};

/// An engine which parses search results from Bing.
pub struct BingEngine;

impl BingEngine {
    /// Creates a new Bing engine.
    pub fn new() -> TaggedEngine {
        TaggedEngine::new(Self {})
    }
}

#[async_trait::async_trait]
impl Engine for BingEngine {
    fn homepage(&self) -> &'static str {
        "https://www.bing.com"
    }

    fn url(
        &self,
        query: &str,
        SearchOptions {
            page_num,
            safe_search,
            date_time_range,
        }: &SearchOptions,
    ) -> Result<String, SearchError> {
        // Turns the page number into the index of the first result.
        // Page 0 is `1`, Page 1 is `11`, Page 2 is `21`, etc...
        let results_per_page = 10;
        let page_start_idx = results_per_page * page_num + 1;

        let safe_search = safe_search.as_lowercase_string();

        let date_time_range_param = if let Some(range) = date_time_range {
            let epoch = Utc.timestamp_opt(0, 0).unwrap();
            let start_timestamp = range.start.signed_duration_since(epoch).num_days();
            let end_timestamp = range.end.signed_duration_since(epoch).num_days();

            Cow::Owned(format!(
                "&filters=ex1%3A%22ez5_{start_timestamp}_{end_timestamp}%22"
            ))
        } else {
            Cow::Borrowed("")
        };

        Ok(format!(
            "https://www.bing.com/search?q={query}&first={page_start_idx}&form=QBLH&safeSearch={safe_search}{date_time_range_param}"
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
        headers.append(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers.append(
            COOKIE,
            HeaderValue::from_static(
                "_EDGE_V=1; SRCHD=AF=NOFORM; _Rwho=u=d; bngps=s=0; _UR=QS=0&TQS=0; ",
            ),
        );
        headers.append(REFERER, HeaderValue::from_static("https://google.com/"));
    }

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
                    .text(parser)
                    .map(|this| this.to_string())
                    .unwrap_or_default();

                let url = title_node
                    .get_first_node_with_tag("a", parser)
                    .and_then(|this| this.get_href().map(|this| this.into_owned()))
                    .unwrap_or_default();

                let summary = this
                    .get_first_node_with_classes(&TEXT_SUMMARY_WRAPPER_CLASSES, parser)
                    .and_then(|this| {
                        this.get_first_node_with_classes(&TEXT_SUMMARY_CLASSES, parser)
                            .and_then(|this| {
                                let Some(text) = this.children_raw_text(parser) else {
                                    return None;
                                };
                                match text {
                                    Cow::Owned(this) => Some(
                                        this.strip_prefix("\u{a0}· ").unwrap_or(&this).to_string(),
                                    ),
                                    Cow::Borrowed(this) => Some(
                                        this.strip_prefix("\u{a0}· ").unwrap_or(&this).to_string(),
                                    ),
                                }
                            })
                    })
                    // If we can't find a summary then the result may have cards instead of basic text.
                    .unwrap_or_else(|| {
                        this.get_first_node_with_classes(&CARD_SUMMARY_CLASSES, parser)
                            .and_then(|this| {
                                this.get_child_nodes_with_classes(
                                    &CARD_SUMMARY_CONTENT_CLASSES,
                                    parser,
                                )
                                .nth(1)
                                .and_then(|this| this.text(parser).map(|this| this.to_string()))
                            })
                            .unwrap_or_default()
                    });

                Some(SearchResult::new(title, url, summary))
            })
            .collect())
    }
}

const SEARCH_RESULT_CLASSES: ClassName = class_names_any! { "b_algo" };

const TITLE_CLASSES: ClassName = class_names_any! { "b_algoheader" };

const TEXT_SUMMARY_WRAPPER_CLASSES: ClassNames = class_names_exact! { "b_caption", "b_capmedia" };
const TEXT_SUMMARY_CLASSES: ClassName = class_names_exact! { "b_lineclamp3" };

const CARD_SUMMARY_CLASSES: ClassNames = class_names_exact! { "b_cards2", "slide" };
const CARD_SUMMARY_CONTENT_CLASSES: ClassName = class_names_exact! { "exsni" };
