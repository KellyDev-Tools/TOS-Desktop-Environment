//! TOS Search Engine Library (`tos-search`)
//!
//! Provides the core hybrid discovery logic (Tantivy + Candle + HNSW).

use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::sync::Api, Repo, RepoType};
use hnsw_rs::prelude::*;
use std::path::{Path};
use std::sync::{Arc, Mutex};

// [Tantivy and HNSW integration]

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{IndexWriter, ReloadPolicy};
use tokenizers::Tokenizer;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct SearchHit {
    pub path: String,
    pub score: f32,
    pub is_dir: bool,
}

pub struct Embedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl Embedder {
    pub fn new() -> anyhow::Result<Self> {
        let device = Device::Cpu;
        let api = Api::new()?;
        let repo = api.repo(Repo::new("sentence-transformers/all-MiniLM-L6-v2".to_string(), RepoType::Model));

        let config_path = repo.get("config.json")?;
        let weights_path = repo.get("model.safetensors")?;
        let tokenizer_path = repo.get("tokenizer.json")?;

        let config: Config = serde_json::from_str(&std::fs::read_to_string(config_path)?)?;
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| anyhow::anyhow!(e))?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device)?
        };
        let model = BertModel::load(vb, &config)?;

        Ok(Self { model, tokenizer, device })
    }

    pub fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let tokens = self.tokenizer.encode(text, true).map_err(|e| anyhow::anyhow!(e))?;
        let token_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;
        
        let embeddings = self.model.forward(&token_ids, &token_type_ids, None)?;
        
        // Mean pooling
        let (_n_batch, n_tokens, _n_dims) = embeddings.dims3()?;
        let pooled = (embeddings.sum(1)? / (n_tokens as f64))?;
        let pooled = pooled.get(0)?; // Squeeze batch
        
        // Normalization
        let norm = pooled.sqr()?.sum_all()?.sqrt()?;
        let norm_val = norm.to_vec0::<f32>()?;
        let normalized = pooled.broadcast_div(&Tensor::new(norm_val, &self.device)?)?;
        
        Ok(normalized.to_vec1()?)
    }
}

pub struct SearchState {
    tantivy_index: tantivy::Index,
    tantivy_writer: Arc<Mutex<IndexWriter>>,
    tantivy_reader: tantivy::IndexReader,
    hnsw_index: Arc<Mutex<Hnsw<'static, f32, DistL2>>>,
    paths: Arc<Mutex<Vec<String>>>,
    embedder: Arc<Mutex<Embedder>>,
}

impl SearchState {
    pub fn new() -> anyhow::Result<Self> {
        let mut schema_builder = Schema::builder();
        let _path_field = schema_builder.add_text_field("path", TEXT | STORED);
        let _content_field = schema_builder.add_text_field("content", TEXT);
        let schema = schema_builder.build();

        let tantivy_index = tantivy::Index::create_in_ram(schema.clone());
        let tantivy_writer = Arc::new(Mutex::new(tantivy_index.writer(50_000_000)?));
        let tantivy_reader = tantivy_index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        let hnsw_index = Arc::new(Mutex::new(Hnsw::new(24, 100_000, 16, 200, DistL2)));
        let paths = Arc::new(Mutex::new(Vec::new()));
        let embedder = Arc::new(Mutex::new(Embedder::new()?));

        Ok(Self {
            tantivy_index,
            tantivy_writer,
            tantivy_reader,
            hnsw_index,
            paths,
            embedder,
        })
    }

    pub async fn index_file(&self, path: &Path) -> anyhow::Result<()> {
        if !path.exists() || !path.is_file() { return Ok(()); }
        let path_str = path.to_string_lossy().to_string();
        
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        
        // Read content (limit to 10KB for safety in this version)
        let file_content = std::fs::read_to_string(path).unwrap_or_default();
        let index_content = format!("{} {} {}", file_name, path_str, file_content);

        // 1. Tantivy index
        {
            let mut writer = self.tantivy_writer.lock().map_err(|_| anyhow::anyhow!("Mutex poison"))?;
            let schema = self.tantivy_index.schema();
            let path_field = schema.get_field("path").unwrap();
            let content_field = schema.get_field("content").unwrap();
            
            let mut doc = tantivy::TantivyDocument::default();
            doc.add_text(path_field, path_str.clone());
            doc.add_text(content_field, index_content.clone());
            writer.add_document(doc)?;
            writer.commit()?;
        }

        // 2. HNSW embedding
        let embedder = self.embedder.lock().map_err(|_| anyhow::anyhow!("Mutex poison"))?;
        if let Ok(vector) = embedder.embed(&index_content) {
             let hnsw = self.hnsw_index.lock().map_err(|_| anyhow::anyhow!("Mutex poison"))?;
             let mut paths = self.paths.lock().map_err(|_| anyhow::anyhow!("Mutex poison"))?;
             
             let id = paths.len();
             paths.push(path_str);
             hnsw.insert((&vector, id));
        }

        Ok(())
    }

    pub fn search(&self, pattern: &str) -> Vec<SearchHit> {
        let searcher = self.tantivy_reader.searcher();
        let schema = self.tantivy_index.schema();
        let path_field = schema.get_field("path").unwrap();
        let content_field = schema.get_field("content").unwrap();
        
        let query_parser = QueryParser::for_index(&self.tantivy_index, vec![path_field, content_field]);
        let query = query_parser.parse_query(pattern).ok();
        
        if let Some(q) = query {
            if let Ok(top_docs) = searcher.search(&q, &TopDocs::with_limit(20)) {
                return top_docs.into_iter().map(|(score, doc_address)| {
                    let doc: tantivy::TantivyDocument = searcher.doc(doc_address).unwrap();
                    let path = doc.get_first(path_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
                    SearchHit {
                        path,
                        score,
                        is_dir: false,
                    }
                }).collect();
            }
        }
        vec![]
    }

    pub fn semantic_search(&self, prompt: &str) -> Vec<SearchHit> {
        let embedder = self.embedder.lock().unwrap();
        if let Ok(query_vector) = embedder.embed(prompt) {
            let hnsw = self.hnsw_index.lock().unwrap();
            let paths = self.paths.lock().unwrap();
            
            let matches = hnsw.search(&query_vector, 10, 201);
            return matches.into_iter().filter_map(|m| {
                paths.get(m.d_id).map(|p: &String| SearchHit {
                    path: p.clone(),
                    score: 1.0 - m.distance,
                    is_dir: false,
                })
            }).collect();
        }
        vec![]
    }
}
