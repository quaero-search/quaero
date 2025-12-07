//! # Quaero
//!
//! A metasearch engine written in rust.
//!
//! ## Example
//! ```
//! use quaero::{
//!     Quaero,
//!     models::search::{SafeSearch, SearchOptions},
//! };
//! use anyhttp_reqwest::ReqwestClientWrapper;
//!
//! tokio::runtime::Runtime::new().unwrap().block_on(async {
//!     // We need to put the client in a wrapper
//!     // as a workaround to rust's orphan rule.
//!     let client = ReqwestClientWrapper::new(reqwest::Client::new());
//!
//!     let meta_engine = Quaero::empty(client);
//!
//!     let search_options = SearchOptions::default()
//!         .page_num(3)
//!         .safe_search(SafeSearch::Moderate);
//!
//!     let response = meta_engine.search("What is ReactJs?", search_options).await;
//! })
//! ```

#![warn(missing_docs)]

use anyhttp::HttpClient;
use smallvec::SmallVec;
use std::{sync::Arc, time::Duration};

mod aggregator;
use aggregator::aggregate_results;

use quaero_shared::models::{
    engine::TaggedEngine,
    sanitized_url::PUBLIC_SUFFIX_LIST,
    score_refiner::ScoreRefiner,
    search::{SearchOptions, SearchResponse},
};

pub use quaero_shared::*;

const DEFAULT_TIMEOUT_SECS: u64 = 15;

/// The main struct of this library. Used to store engines to query and aggregate later.
pub struct Quaero<C: HttpClient + 'static, const N: usize = 10> {
    client: Arc<C>,
    engines: SmallVec<[TaggedEngine; N]>,
    score_refiner: Option<Box<dyn ScoreRefiner>>,
    timeout: Duration,
}

impl<C: HttpClient + Send + Sync + 'static, const N: usize> Quaero<C, N> {
    /// Creates a new Quaero instance with the included engines stored inline.
    pub fn new(client: C, engines: impl Into<SmallVec<[TaggedEngine; N]>>) -> Quaero<C, N> {
        init();

        Quaero {
            client: client.into(),
            engines: engines.into(),
            score_refiner: None,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    /// Inserts an engine into the Quaero instance.
    pub fn push_engine(&mut self, engine: TaggedEngine) {
        self.engines.push(engine)
    }

    /// Extends the quaero instance's engines with an iterator.
    pub fn extend_engines<I: IntoIterator<Item = TaggedEngine>>(&mut self, iter: I) {
        self.engines.extend(iter);
    }

    /// Sets and initializes the score refiner for this quaero instance.
    ///
    /// Score refiners apply another stage of reranking the the search results.
    pub async fn score_refiner(mut self, refiner: impl ScoreRefiner + 'static) -> Self {
        let _ = refiner.init().await;

        self.score_refiner = Some(Box::new(refiner));
        self
    }

    /// Sets the maximum amount of time each engine has to fetch its results.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Performs a search query across all of the quaero instance's engines and aggregates their results.
    pub async fn search<'a>(
        &'a self,
        query: impl AsRef<str>,
        options: impl Into<Arc<SearchOptions>>,
    ) -> SearchResponse {
        aggregate_results(self, query.as_ref(), options.into()).await
    }
}

impl<C: HttpClient + Send + Sync + 'static> Quaero<C> {
    /// Creates and empty Quaero instance.
    pub fn empty(client: impl Into<Arc<C>>) -> Quaero<C> {
        init();

        Quaero {
            client: client.into(),
            engines: SmallVec::new(),
            score_refiner: None,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }
}

#[inline(always)]
fn init() {
    // Makes sure certain things are initialized to prevent cold searches.

    #[cfg(feature = "synonyms")]
    {
        thesaurus::init();
    }

    // This is initialized when it is first accessed.
    let _ = PUBLIC_SUFFIX_LIST;
}
