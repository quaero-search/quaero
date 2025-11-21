/// A trait for defining behaviour for refining relevance scores for search results.
#[async_trait::async_trait]
pub trait ScoreRefiner: Send + Sync {
    /// Refines results based on the query and target strings obtained from each result.
    async fn scores(
        &self,
        query: &str,
        targets: &[String],
    ) -> anyhow::Result<Box<[anyhow::Result<f32>]>>;

    /// Initialises any data that the refiner may need.
    async fn init(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Returns the maximum amount of results that is allowed to be refined.
    fn max_results(&self) -> usize;
}
