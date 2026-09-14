//! Pretrained semantic vectors (R11 Phase VEC).
//!
//! Loads the OVERLAP subset of PyThaiNLP's `thai2fit_wv` word embeddings
//! (MIT-licensed; trained on Thai Wikipedia) intersected with the wacha
//! vocabulary — 28,589 words × 300 dim, ~35 MB — from a gitignored, regenerable
//! blob (`data/thai2fit_overlap.vec.blob`, produced by
//! `scripts/gen_thai2fit_overlap.py`). We did NOT train anything.
//!
//! Nearest-neighbour = a flat f32 array + cosine similarity (linear scan). At this
//! vocabulary size that is fast enough; no octree/ANN index (a deliberate rejection
//! of specialized indexing this round — wrong scale, wrong problem).
//!
//! **Licence discipline (mirrors R9 D1 / R10 R3):** thai2fit is MIT, but it was
//! trained on CC BY-SA Thai Wikipedia. The author's explicit MIT release governs
//! the artifact; still, out of the same caution applied to RID/ศัพท์บัญญัติ, the
//! vectors are used in the LOCAL/judge build only and gated out of any public
//! deploy until an explicit confirmation.

use std::collections::HashMap;

/// A flat word-vector store with cosine nearest-neighbour search.
#[derive(Default)]
pub struct Vectors {
    dim: usize,
    words: Vec<String>,
    /// Row-major `words.len() * dim` f32, each row L2-normalized at load so cosine
    /// similarity is just a dot product.
    data: Vec<f32>,
    index: HashMap<String, usize>,
}

impl Vectors {
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
    pub fn len(&self) -> usize {
        self.words.len()
    }
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Parse the blob: `u32 count, u32 dim, [u16 len + utf8 + dim*f32]*`.
    /// Rows are L2-normalized on load. Returns an empty store on any malformation.
    pub fn from_blob(bytes: &[u8]) -> Self {
        let mut v = Vectors::default();
        if bytes.len() < 8 {
            return v;
        }
        let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let dim = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        if dim == 0 {
            return v;
        }
        v.dim = dim;
        v.words.reserve(count);
        v.data.reserve(count * dim);
        let mut off = 8;
        for _ in 0..count {
            if off + 2 > bytes.len() {
                break;
            }
            let wl = u16::from_le_bytes(bytes[off..off + 2].try_into().unwrap()) as usize;
            off += 2;
            if off + wl + dim * 4 > bytes.len() {
                break;
            }
            let word = match std::str::from_utf8(&bytes[off..off + wl]) {
                Ok(s) => s.to_string(),
                Err(_) => break,
            };
            off += wl;
            let mut row = Vec::with_capacity(dim);
            for _ in 0..dim {
                let f = f32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
                off += 4;
                row.push(f);
            }
            // L2-normalize so cosine == dot product.
            let norm = row.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for x in &mut row {
                    *x /= norm;
                }
            }
            v.index.insert(word.clone(), v.words.len());
            v.words.push(word);
            v.data.extend_from_slice(&row);
        }
        v
    }

    fn row(&self, i: usize) -> &[f32] {
        &self.data[i * self.dim..(i + 1) * self.dim]
    }

    /// The (L2-normalized) vector for a word, if present.
    pub fn vector_of(&self, word: &str) -> Option<&[f32]> {
        self.index.get(word).map(|&i| self.row(i))
    }

    /// A centroid (mean of the in-vocab words' normalized rows, then re-normalized).
    /// Returns None if none of the words are in vocab. Used by the R12 intent
    /// fallback to precompute a seed-phrase centroid per intent.
    pub fn centroid(&self, words: &[String]) -> Option<Vec<f32>> {
        if self.dim == 0 {
            return None;
        }
        let mut acc = vec![0f32; self.dim];
        let mut n = 0usize;
        for w in words {
            if let Some(v) = self.vector_of(w) {
                for (a, x) in acc.iter_mut().zip(v) {
                    *a += *x;
                }
                n += 1;
            }
        }
        if n == 0 {
            return None;
        }
        let norm = acc.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut acc {
                *x /= norm;
            }
        }
        Some(acc)
    }

    /// Cosine similarity between an already-normalized vector and a centroid
    /// (also treated as normalized). Both must be `dim`-length.
    pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    /// Cosine similarity between two words (both must be present), or None.
    pub fn similarity(&self, a: &str, b: &str) -> Option<f32> {
        let (&ia, &ib) = (self.index.get(a)?, self.index.get(b)?);
        Some(self.row(ia).iter().zip(self.row(ib)).map(|(x, y)| x * y).sum())
    }

    /// Top-`k` cosine nearest neighbours of `word` (excluding itself), if present.
    /// Linear scan — deterministic (score desc, then word asc).
    pub fn neighbours(&self, word: &str, k: usize) -> Vec<(String, f32)> {
        let Some(&qi) = self.index.get(word) else { return Vec::new() };
        let q = self.row(qi);
        let mut scored: Vec<(f32, usize)> = (0..self.words.len())
            .filter(|&i| i != qi)
            .map(|i| {
                let s: f32 = q.iter().zip(self.row(i)).map(|(x, y)| x * y).sum();
                (s, i)
            })
            .collect();
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| self.words[a.1].cmp(&self.words[b.1]))
        });
        scored.into_iter().take(k).map(|(s, i)| (self.words[i].clone(), s)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build a tiny 2-dim store: A=(1,0), B≈(0.99,0.14) close to A, C=(0,1) far.
    fn tiny() -> Vectors {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(&2u32.to_le_bytes());
        for (w, v) in [("A", [1.0f32, 0.0]), ("B", [0.99, 0.14]), ("C", [0.0, 1.0])] {
            let b = w.as_bytes();
            bytes.extend_from_slice(&(b.len() as u16).to_le_bytes());
            bytes.extend_from_slice(b);
            for f in v {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
        }
        Vectors::from_blob(&bytes)
    }

    #[test]
    fn loads_and_normalizes() {
        let v = tiny();
        assert_eq!(v.len(), 3);
        assert_eq!(v.dim(), 2);
        // self-similarity of a normalized row is ~1.0
        assert!((v.similarity("A", "A").unwrap() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn nearest_neighbour_orders_by_cosine() {
        let v = tiny();
        let nn = v.neighbours("A", 2);
        assert_eq!(nn[0].0, "B"); // B is closer to A than C
        assert!(nn[0].1 > nn[1].1);
    }

    #[test]
    fn missing_word_is_empty() {
        assert!(tiny().neighbours("Z", 3).is_empty());
        assert!(tiny().similarity("A", "Z").is_none());
    }
}
