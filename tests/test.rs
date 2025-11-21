#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test() -> anyhow::Result<()> {
    use quaero::{
        Quaero,
        models::search::{SafeSearch, SearchOptions},
    };
    use quaero_bert::BertScoreRefiner;
    use tokio::time::Instant;

    tracing_subscriber::fmt::init();

    let client = reqwest::Client::builder().build().unwrap();

    let meta_engine = Quaero::new(client, quaero_engines::default())
        .with_score_refiner(BertScoreRefiner::new(10))
        .await;

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
    println!("{}", end);

    Ok(())
}
