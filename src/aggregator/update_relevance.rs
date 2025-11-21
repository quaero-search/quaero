use keyword_extraction::tf_idf::{TfIdf, TfIdfParams};
use quaero_shared::models::search::SearchResultWithMetadata;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub trait UpdateRelevance {
    /// Uses `td-idf` to calculate how relevant this search result is to a particular search query.
    fn update_relevance(
        &mut self,
        query_tokens: &Vec<String>,
        stop_words: &[String],
        punctuation: &[String],
    );
}

impl UpdateRelevance for SearchResultWithMetadata {
    fn update_relevance(
        &mut self,
        query_tokens: &Vec<String>,
        stop_words: &[String],
        punctuation: &[String],
    ) {
        let documents: Vec<String> = vec![
            self.search_result.title.to_string(),
            self.search_result.url.to_string(),
            self.search_result.summary.to_string(),
        ];

        let params = TfIdfParams::UnprocessedDocuments(&documents, stop_words, Some(punctuation));
        let tf_idf = TfIdf::new(params);

        let relevance_score: f32 = query_tokens
            .par_iter()
            .map(|token| tf_idf.get_score(token))
            .sum();
        let relevance_score = relevance_score / query_tokens.len() as f32;

        self.relevance_score = f32::from(!relevance_score.is_nan()) * relevance_score;
    }
}
