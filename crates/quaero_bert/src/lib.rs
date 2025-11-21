use anyhow::{Error, Result, anyhow};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{Repo, RepoType, api::tokio::Api};
use quaero_shared::models::score_refiner::ScoreRefiner;
use tokenizers::{PaddingParams, Tokenizer};
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};

/// The data that [BertScoreRefiner] relies on.
pub struct BertScoreRefinerData {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

/// Refines scores using the `sentence-transformers/all-MiniLM-L6-v2`
/// model which is distilled and optimized from BERT and RoBERTa models.
pub struct BertScoreRefiner {
    /// The maximum amount of results to refine scores for.
    max_results: usize,
    data: Mutex<Option<BertScoreRefinerData>>,
}

impl BertScoreRefiner {
    /// Creates a new [BertScoreRefiner].
    pub fn new(max_results: usize) -> Self {
        Self {
            max_results,
            data: Mutex::new(None),
        }
    }

    /// Gets the underlying data and makes sure it's initialized.
    pub async fn get(&self) -> anyhow::Result<MappedMutexGuard<'_, BertScoreRefinerData>> {
        self.init().await?;

        Ok(MutexGuard::map(self.data.lock().await, |opt| {
            opt.as_mut().expect("data should be initialized")
        }))
    }
}

#[async_trait::async_trait]
impl ScoreRefiner for BertScoreRefiner {
    fn max_results(&self) -> usize {
        self.max_results
    }

    async fn init(&self) -> anyhow::Result<()> {
        let mut data = self.data.lock().await;

        if data.is_none() {
            let device = best_device(0)?;

            let api = Api::new().unwrap();
            let repo = Repo::with_revision(
                "sentence-transformers/all-MiniLM-L6-v2".to_string(),
                RepoType::Model,
                "refs/pr/21".to_string(),
            );
            let repo = api.repo(repo);

            let config_filename = repo.get("config.json").await?;
            let tokenizer_filename = repo.get("tokenizer.json").await?;
            let weights_filename = repo.get("model.safetensors").await?;

            let config = std::fs::read_to_string(config_filename)?;
            let mut config: Config = serde_json::from_str(&config)?;
            config.hidden_act = candle_transformers::models::bert::HiddenAct::GeluApproximate;

            let mut tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(Error::msg)?;
            let vb = unsafe {
                VarBuilder::from_mmaped_safetensors(&[weights_filename], DTYPE, &device)?
            };

            let model = BertModel::load(vb, &config)?;

            if let Some(pp) = tokenizer.get_padding_mut() {
                pp.strategy = tokenizers::PaddingStrategy::BatchLongest
            } else {
                let pp = PaddingParams {
                    strategy: tokenizers::PaddingStrategy::BatchLongest,
                    ..Default::default()
                };
                tokenizer.with_padding(Some(pp));
            }

            *data = Some(BertScoreRefinerData {
                model,
                tokenizer,
                device,
            });
        }

        Ok(())
    }

    async fn scores(
        &self,
        query: &str,
        targets: &[String],
    ) -> anyhow::Result<Box<[anyhow::Result<f32>]>> {
        let BertScoreRefinerData {
            model,
            tokenizer,
            device,
        } = &*self.get().await?;

        let to_encode: Vec<&str> = targets
            .iter()
            .map(|t| t.as_ref())
            .chain(std::iter::once(query))
            .collect();

        let to_encode_len = to_encode.len();

        let tokens = tokenizer
            .encode_batch(to_encode, true)
            .map_err(Error::msg)?;

        let token_ids = tokens
            .iter()
            .map(|tokens| {
                let tokens = tokens.get_ids().to_vec();
                Ok(Tensor::new(tokens.as_slice(), &device)?)
            })
            .collect::<Result<Box<[_]>>>()?;

        let attention_mask = tokens
            .iter()
            .map(|tokens| {
                let tokens = tokens.get_attention_mask().to_vec();
                Ok(Tensor::new(tokens.as_slice(), &device)?)
            })
            .collect::<Result<Box<[_]>>>()?;

        let token_ids = Tensor::stack(&token_ids, 0)?;
        let attention_mask = Tensor::stack(&attention_mask, 0)?;
        let token_type_ids = token_ids.zeros_like()?;

        let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask))?;
        let (_, tokens_length, _) = embeddings.dims3()?;
        let embeddings = (embeddings.sum(1)? / (tokens_length as f64))?;
        let embeddings = normalize_l2(&embeddings)?;

        let query_embeddings = embeddings.get(to_encode_len - 1)?;

        Ok((0..to_encode_len - 1)
            .map(|idx| {
                let this_embeddings = match embeddings.get(idx) {
                    Ok(this_embeddings) => this_embeddings,
                    Err(err) => return Err(anyhow!(err)),
                };

                let score = (&query_embeddings * &this_embeddings)
                    .and_then(|score| score.sum_all())
                    .and_then(|score| score.to_scalar::<f32>())
                    .map_err(|e| anyhow!(e));

                score
            })
            .collect())
    }
}

fn normalize_l2(v: &Tensor) -> Result<Tensor> {
    Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}

fn best_device(#[allow(unused)] ordinal: usize) -> candle_core::Result<Device> {
    #[cfg(feature = "cuda")]
    {
        if let Ok(dev) = Device::new_cuda(ordinal) {
            return Ok(dev);
        }
    }

    #[cfg(feature = "metal")]
    {
        if let Ok(dev) = Device::new_metal(ordinal) {
            return Ok(dev);
        }
    }

    Ok(Device::Cpu)
}
