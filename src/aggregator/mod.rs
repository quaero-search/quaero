use anyhttp::HttpClient;
use http::{Request, StatusCode};
use keyword_extraction::tokenizer::Tokenizer;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};
use tokio::task::JoinSet;
use tracing::{Level, error, info, level_enabled};

mod update_relevance;

use quaero_shared::models::{
    engine::{EngineId, TaggedEngine},
    score_refiner::ScoreRefiner,
    search::{SearchError, SearchOptions, SearchResponse, SearchResult, SearchResultWithMetadata},
};

use crate::{Quaero, aggregator::update_relevance::UpdateRelevance};

#[inline(always)]
pub async fn aggregate_results<C: HttpClient + 'static, const N: usize>(
    quaero: &Quaero<C, N>,
    query: &str,
    options: Arc<SearchOptions>,
) -> SearchResponse {
    info!("Commencing Quaero search!");

    let encoded_query = urlencoding::encode(query);
    let encoded_query = Arc::new(encoded_query.to_string());

    let query_tokens = get_query_tokens(query);

    let mut tasks: JoinSet<(EngineId, Result<Vec<(String, SearchResult)>, SearchError>)> =
        JoinSet::new();

    let engines = &quaero.engines;
    let engines_len = engines.len();

    let timeout_duration = quaero.timeout;

    for TaggedEngine(engine_id, engine) in engines {
        let engine_name = engine.name();

        info!("[{}] Fetching search results...", engine_name);

        let (outer_engine_id, engine_id, engine) =
            (engine_id.clone(), engine_id.clone(), engine.clone());
        let client = quaero.client.clone();
        let options = options.clone();
        let encoded_query = encoded_query.clone();

        tasks.spawn(async move {
            let result = tokio::time::timeout(timeout_duration, async {
                let request_url = match engine.url(encoded_query.as_ref(), options.as_ref()) {
                    Ok(request_url) => request_url,
                    Err(search_error) => {
                        error!(
                            "[{}] Error when obtaining URL: {}",
                            engine_name, search_error
                        );
                        return (engine_id, Err(search_error));
                    }
                };

                let mut request = match Request::get(request_url).body(vec![]) {
                    Ok(request) => request,
                    Err(err) => {
                        error!("[{}] Failed to build request: {:#?}", engine_name, err);
                        return (engine_id, Err(SearchError::RequestFailed));
                    }
                };

                engine.headers(request.headers_mut(), &options);

                let response_result = client.execute(request).await;

                let response = match response_result {
                    Ok(response) => response,
                    Err(err) => {
                        error!("[{}] Failed to fetch results: {:#?}", engine_name, err);
                        return (engine_id, Err(SearchError::RequestFailed));
                    }
                };

                if let Err(search_error) = engine.validate_response(&response) {
                    error!("[{}] Failed pre-parse check: {}", engine_name, search_error);
                    return (engine_id, Err(search_error));
                }

                if response.status() == StatusCode::TOO_MANY_REQUESTS {
                    error!(
                        "[{}] Failed to fetch results: {}",
                        engine_name,
                        SearchError::Blocked
                    );
                    return (engine_id, Err(SearchError::Blocked));
                }

                let bytes = match response.bytes().await {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        error!(
                            "[{}] Failed to parse response text: {:#?}",
                            engine_name, err
                        );
                        return (engine_id, Err(SearchError::RequestFailed));
                    }
                };

                let Ok(data) = str::from_utf8(&bytes) else {
                    error!("[{}] No response text was found.", engine_name);
                    return (engine_id, Err(SearchError::NoResponseText));
                };
                let data = data.to_string();

                let results = tokio::task::spawn_blocking(move || engine.parse(data))
                    .await
                    .unwrap_or_else(|_| Err(SearchError::Unknown))
                    .and_then(|this| {
                        if this.len() == 0 {
                            Err(SearchError::NoResultsFound)
                        } else {
                            Ok(this)
                        }
                    });

                let results = match results {
                    Ok(results) => results,
                    Err(search_error) => {
                        error!(
                            "[{}] Failed to parse results: {}",
                            engine_name, search_error
                        );
                        return (engine_id, Err(search_error));
                    }
                };

                info!("[{}] Successfully fetched search results!", engine_name);

                (engine_id, Ok(results))
            })
            .await;

            match result {
                Ok(result) => result,
                Err(_err) => {
                    error!(
                        "[{}] Could not fetch results within the allowed time limit.",
                        engine_name
                    );
                    (outer_engine_id, Err(SearchError::Timeout))
                }
            }
        });
    }

    let mut statuses: Vec<(EngineId, Result<(), SearchError>)> = Vec::with_capacity(engines_len);
    let mut results: HashMap<String, SearchResultWithMetadata> = HashMap::new();

    while let Some(Ok((engine_id, engine_results))) = tasks.join_next().await {
        let engine_results = match engine_results {
            Ok(engine_results) => engine_results,
            Err(reason) => {
                statuses.push((engine_id, Err(reason)));
                continue;
            }
        };

        let engine_results: Box<[(String, SearchResultWithMetadata)]> = engine_results
            .into_par_iter()
            .map(|(url, result)| {
                let mut result = SearchResultWithMetadata::new(result, &engine_id);
                result.update_relevance(&query_tokens, &*STOP_WORDS, &*PUNCTUATION);

                (url, result)
            })
            .collect();

        for (url, mut result) in engine_results {
            match results.entry(url) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let existing_result = entry.get_mut();

                    if existing_result.relevance_score >= result.relevance_score {
                        existing_result.engines.extend(result.engines.into_iter());
                    } else {
                        result.engines.extend(existing_result.engines.drain(..));
                        *existing_result = result;
                    }
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(result);
                }
            }
        }

        statuses.push((engine_id, Ok(())));
    }

    let results = sort_results(results, query, quaero.score_refiner.as_ref()).await;

    if level_enabled!(Level::INFO) {
        info!(
            "Finished quaero search: {}/{} engines queried successfully.",
            statuses.iter().filter(|this| this.1.is_ok()).count(),
            statuses.len()
        )
    }

    SearchResponse { results, statuses }
}

async fn sort_results(
    results: HashMap<String, SearchResultWithMetadata>,
    query: &str,
    score_refiner: Option<&Box<dyn ScoreRefiner + 'static>>,
) -> Box<[SearchResultWithMetadata]> {
    let mut results: Box<[SearchResultWithMetadata]> = results.into_values().collect();

    results.par_sort_unstable_by(|a, b| b.relevance_score.total_cmp(&a.relevance_score));

    if let Some(score_refiner) = score_refiner {
        let top_results_count = score_refiner.max_results().min(results.len());

        let top_results = &mut results[0..top_results_count];

        let top_summaries = top_results
            .iter()
            .map(|result| result.search_result.snippet())
            .collect::<Box<[String]>>();

        let top_scores = score_refiner.scores(query, &top_summaries).await.unwrap();

        for (idx, score) in top_scores.into_iter().enumerate() {
            let Ok(score) = score else {
                continue;
            };

            top_results[idx].relevance_score = score;
        }

        top_results.par_sort_unstable_by(|a, b| b.relevance_score.total_cmp(&a.relevance_score));
    }

    results
}

fn get_query_tokens(query: &str) -> Vec<String> {
    let query_tokenizer = Tokenizer::new(query, &*STOP_WORDS, Some(&*PUNCTUATION));
    let mut query_tokens = query_tokenizer.split_into_words();

    #[cfg(feature = "synonyms")]
    {
        let extra_query_tokens: Box<[String]> = query_tokens
            .iter()
            .flat_map(|token| thesaurus::synonyms(token))
            .collect();

        query_tokens.extend(extra_query_tokens);
    }

    query_tokens
}

static STOP_WORDS: LazyLock<Box<[String]>> =
    LazyLock::new(|| stop_words::get(stop_words::LANGUAGE::English).into_boxed_slice());

static PUNCTUATION: LazyLock<[String; 14]> = LazyLock::new(|| {
    [
        ".".to_string(),
        ",".to_string(),
        ":".to_string(),
        ";".to_string(),
        "!".to_string(),
        "?".to_string(),
        "(".to_string(),
        ")".to_string(),
        "[".to_string(),
        "]".to_string(),
        "{".to_string(),
        "}".to_string(),
        "\"".to_string(),
        "'".to_string(),
    ]
});
