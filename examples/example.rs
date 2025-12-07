use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhttp_reqwest::ReqwestClientWrapper;
    use quaero::{
        Quaero,
        models::search::{SafeSearch, SearchOptions},
    };
    use quaero_bert::BertScoreRefiner;
    use tokio::time::Instant;

    tracing_subscriber::fmt::init();

    // We need to put the client in a wrapper
    // as a workaround to rust's orphan rule.
    let client = ReqwestClientWrapper::new(reqwest::Client::new());

    let meta_engine = Quaero::new(client, quaero_engines::default())
        .score_refiner(BertScoreRefiner::new(10))
        .await
        .timeout(Duration::from_secs(8));

    let search_options = SearchOptions::default()
        .page_num(0)
        .safe_search(SafeSearch::Off)
        .into_arc();

    let start = Instant::now();

    let response = meta_engine
        .search(
            "Top reasons why Rust is the best programming language?",
            search_options,
        )
        .await;

    let end = (Instant::now() - start).as_secs_f32();

    println!("{:#?}", response.results);
    println!("{:#?}", response.statuses);
    println!("{}", end);

    Ok(())
}
