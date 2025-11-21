# Quaero

Quaero is a small meta-search framework designed to be embedded in other programs.

## Example Usage

```rs
use quaero::{
    Quaero,
    models::search::{SafeSearch, SearchOptions},
};

tokio::runtime::Runtime::new().unwrap().block_on(async {
    let client = reqwest::Client::builder().build().unwrap();

    let meta_engine = Quaero::new(client, quaero_engines::default());

    let search_options = SearchOptions::default()
        .page_num(3)
        .safe_search(SafeSearch::Moderate);

    let response = meta_engine.search("What is ReactJs?", search_options).await;
})
```
