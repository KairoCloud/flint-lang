use crate::{AiClient, AiError};

pub struct Embedder {
    model: String,
}

impl Embedder {
    pub fn new(model: &str) -> Self {
        Embedder {
            model: model.to_string(),
        }
    }

    pub fn embed(&self, text: &str) -> Result<Embedding, AiError> {
        let client = crate::global_client().ok_or(AiError::NotInitialized)?;
        let vector = client.embed(text)?;
        Ok(Embedding {
            vector,
            model: self.model.clone(),
        })
    }

    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>, AiError> {
        let client = crate::global_client().ok_or(AiError::NotInitialized)?;
        
        texts.iter()
            .map(|text| {
                let vector = client.embed(text)?;
                Ok(Embedding {
                    vector,
                    model: self.model.clone(),
                })
            })
            .collect()
    }

    pub async fn embed_async(&self, text: &str) -> Result<Embedding, AiError> {
        let client = crate::global_client().ok_or(AiError::NotInitialized)?;
        let vector = client.embed(text)?;
        Ok(Embedding {
            vector,
            model: self.model.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub model: String,
}

impl Embedding {
    pub fn cosine_similarity(&self, other: &Embedding) -> f32 {
        if self.vector.len() != other.vector.len() {
            return 0.0;
        }

        let dot: f32 = self.vector.iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();

        let mag_a: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }

        dot / (mag_a * mag_b)
    }

    pub fn euclidean_distance(&self, other: &Embedding) -> f32 {
        if self.vector.len() != other.vector.len() {
            return f32::MAX;
        }

        let sum: f32 = self.vector.iter()
            .zip(other.vector.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();

        sum.sqrt()
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddingRequest {
    pub texts: Vec<String>,
    pub model: String,
    pub normalize: bool,
}

impl EmbeddingRequest {
    pub fn new(texts: Vec<String>, model: &str) -> Self {
        EmbeddingRequest {
            texts,
            model: model.to_string(),
            normalize: true,
        }
    }

    pub fn with_normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }
}

pub struct EmbeddingCache {
    cache: std::collections::HashMap<String, Embedding>,
    max_size: usize,
}

impl EmbeddingCache {
    pub fn new(max_size: usize) -> Self {
        EmbeddingCache {
            cache: std::collections::HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, text: &str) -> Option<&Embedding> {
        self.cache.get(text)
    }

    pub fn insert(&mut self, text: String, embedding: Embedding) {
        if self.cache.len() >= self.max_size {
            self.cache.remove(&self.cache.keys().next().unwrap().clone());
        }
        self.cache.insert(text, embedding);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for EmbeddingCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

pub fn compute_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    dot / (mag_a * mag_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_similarity() {
        let a = Embedding { vector: vec![1.0, 0.0], model: "test".to_string() };
        let b = Embedding { vector: vec![1.0, 0.0], model: "test".to_string() };
        let c = Embedding { vector: vec![0.0, 1.0], model: "test".to_string() };

        assert!((a.cosine_similarity(&b) - 1.0).abs() < 0.001);
        assert!((a.cosine_similarity(&c)).abs() < 0.001);
    }

    #[test]
    fn test_cache() {
        let mut cache = EmbeddingCache::new(2);
        cache.insert("hello".to_string(), Embedding { vector: vec![1.0], model: "test".to_string() });
        assert!(cache.get("hello").is_some());
        assert!(cache.get("world").is_none());
    }
}