# Quaero

Quaero is a small meta-search framework designed to be embedded in other programs.

## Example Usage

```rs
use quaero::{
    Quaero,
    models::search::{SafeSearch, SearchOptions},
};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::builder().build().unwrap();

    let meta_engine = Quaero::new(client, quaero_engines::default());

    let search_options = SearchOptions::default()
        .page_num(3)
        .safe_search(SafeSearch::Moderate);

    let response = meta_engine.search("What is ReactJs?", search_options).await;
}
```

## Installation

Quaero is not currently on crates.io so you'll need to install it via the github url.

```toml
quaero = { git = "https://github.com/quaero-search/quaero" }
quaero_engines = { git = "https://github.com/quaero-search/quaero" }
```

## Score Refinement

Quaero internally uses `tf-idf` to rank each search result. You can however rerank the top `n` results with a more robust algorithm.

Here's an example using Bert to refine the top 10 search results.

```rs
use quaero_bert::BertScoreRefiner;

let meta_engine = Quaero::new(client, quaero_engines::default())
    .with_score_refiner(BertScoreRefiner::new(10))
    .await;
```

```toml
quaero_bert = { git = "https://github.com/quaero-search/quaero" }
```
