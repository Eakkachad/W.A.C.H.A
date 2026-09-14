//! Reverse dictionary (Round 8, Phase C) — "ค้นคำจากความหมาย": find headwords
//! by describing their meaning, instead of looking a word up to get its meaning.
//!
//! ## How it works (all our own code — no external search engine)
//! 1. Every defined entry's **definition is segmented with our own segmenter**
//!    (the same greedy longest-match trie that powers forward lookup), so the
//!    reverse index and the forward path share one tokenizer.
//! 2. Tokens are collected into an inverted index stored in **CSR** form
//!    (compressed sparse row): a flat postings array + per-term offset slice,
//!    so a term's postings are one contiguous span — cache-friendly, and it
//!    serialises to a compact blob for the WASM build.
//! 3. Ranking is **Okapi BM25** (k1 = 1.2, b = 0.75), ~40 lines below. This is
//!    hand-written here; `katgpt-rs` has no BM25 and is not involved.
//! 4. A result carries the **matched query tokens** for that document, so the UI
//!    can show *why* each candidate matched (explainability, same principle as
//!    the forward relation paths).
//!
//! Stopwords: a tiny set of ultra-frequent Thai function words is skipped at
//! BOTH index and query time so they neither bloat postings nor dominate scores.

use std::collections::HashMap;

const K1: f32 = 1.2;
const B: f32 = 0.75;
/// Q2 coordination/coverage exponent: final score is BM25 × coverage^COORD_ALPHA,
/// where coverage = (distinct query terms matched) / (distinct query terms). This
/// damps a hit that matches only one rare high-IDF query word. 1.0 = linear
/// coverage weighting (a doc matching half the query keeps half its score).
const COORD_ALPHA: f32 = 1.0;

/// Ultra-frequent Thai function words that carry no discriminative meaning.
/// Skipped at index and query time. Deliberately small and conservative.
const STOPWORDS: &[&str] = &[
    "ที่", "ของ", "และ", "เป็น", "ใน", "การ", "ความ", "หรือ", "ให้", "ได้",
    "มี", "กับ", "ก็", "จะ", "ว่า", "ซึ่ง", "อัน", "โดย", "แก่", "ต่อ",
    "นั้น", "นี้", "อย่าง", "เช่น", "คือ", "ๆ", "ๆๆ",
];

/// A ranked reverse-dictionary hit.
#[derive(Debug, Clone)]
pub struct ReverseHit {
    /// The candidate headword.
    pub word: String,
    /// BM25 score (higher = better).
    pub score: f32,
    /// The query tokens that matched this document's definition (explainability).
    pub matched: Vec<String>,
}

/// Inverted index over segmented definitions, CSR postings + BM25 statistics.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ReverseIndex {
    /// doc_id -> headword.
    docs: Vec<String>,
    /// doc_id -> definition length in (non-stopword) tokens.
    doc_len: Vec<u32>,
    /// term string -> term_id.
    term_id: HashMap<String, u32>,
    /// CSR row offsets into `postings`: term `t`'s postings are
    /// `postings[offsets[t] .. offsets[t+1]]`. Length = num_terms + 1.
    offsets: Vec<u32>,
    /// Flat postings: (doc_id, tf) pairs, grouped by term (CSR).
    postings: Vec<(u32, u32)>,
    /// Average document length (tokens), for BM25.
    avg_len: f32,
}

impl ReverseIndex {
    fn is_stopword(tok: &str) -> bool {
        let t = tok.trim();
        t.is_empty() || t.chars().all(|c| !c.is_alphabetic()) || STOPWORDS.contains(&t)
    }

    /// Build from `(headword, definition)` pairs, segmenting each definition with
    /// the supplied tokenizer closure (the engine's segmenter). Deterministic.
    pub fn build<'a, I, F>(entries: I, mut segment: F) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
        F: FnMut(&str) -> Vec<String>,
    {
        let mut docs: Vec<String> = Vec::new();
        let mut doc_len: Vec<u32> = Vec::new();
        // term_id assignment + per-term (doc -> tf) accumulation.
        let mut term_id: HashMap<String, u32> = HashMap::new();
        // term_id -> Vec<(doc_id, tf)>
        let mut term_postings: Vec<Vec<(u32, u32)>> = Vec::new();

        for (hw, def) in entries {
            let did = docs.len() as u32;
            let toks = segment(def);
            // token -> tf within this doc (stopwords skipped).
            let mut tf: HashMap<String, u32> = HashMap::new();
            let mut len = 0u32;
            for t in toks {
                if Self::is_stopword(&t) {
                    continue;
                }
                *tf.entry(t).or_insert(0) += 1;
                len += 1;
            }
            if len == 0 {
                continue; // a definition of only stopwords contributes nothing
            }
            docs.push(hw.to_string());
            doc_len.push(len);
            for (t, f) in tf {
                let tid = *term_id.entry(t).or_insert_with(|| {
                    term_postings.push(Vec::new());
                    (term_postings.len() - 1) as u32
                });
                term_postings[tid as usize].push((did, f));
            }
        }

        // Flatten to CSR (postings sorted by doc_id within each term for
        // determinism and better locality).
        let num_terms = term_postings.len();
        let mut offsets = Vec::with_capacity(num_terms + 1);
        let mut postings = Vec::new();
        offsets.push(0u32);
        for mut plist in term_postings {
            plist.sort_unstable_by_key(|&(d, _)| d);
            postings.extend_from_slice(&plist);
            offsets.push(postings.len() as u32);
        }

        let total: u64 = doc_len.iter().map(|&l| l as u64).sum();
        let avg_len = if doc_len.is_empty() { 0.0 } else { total as f32 / doc_len.len() as f32 };

        ReverseIndex { docs, doc_len, term_id, offsets, postings, avg_len }
    }

    /// Number of indexed documents (definitions).
    pub fn doc_count(&self) -> usize {
        self.docs.len()
    }

    /// Number of distinct terms.
    pub fn term_count(&self) -> usize {
        self.term_id.len()
    }

    /// Approximate in-memory size of the index (bytes) — postings + offsets +
    /// doc metadata (excludes the term-string HashMap keys, reported separately).
    pub fn approx_bytes(&self) -> usize {
        self.postings.len() * std::mem::size_of::<(u32, u32)>()
            + self.offsets.len() * 4
            + self.doc_len.len() * 4
            + self.docs.iter().map(|d| d.len()).sum::<usize>()
    }

    /// BM25 idf for a term with document frequency `df` over `n` docs.
    #[inline]
    fn idf(n: usize, df: usize) -> f32 {
        // Okapi BM25 idf with the standard +0.5 smoothing, floored at 0.
        let n = n as f32;
        let df = df as f32;
        (((n - df + 0.5) / (df + 0.5)) + 1.0).ln()
    }

    /// Reverse lookup: segment `query`, score every document that shares a
    /// (non-stopword) term via BM25, and return the top-`k` headwords with the
    /// query tokens that matched. Deterministic (score desc, then headword).
    pub fn search<F>(&self, query: &str, k: usize, mut segment: F) -> Vec<ReverseHit>
    where
        F: FnMut(&str) -> Vec<String>,
    {
        let n = self.docs.len();
        // Distinct, non-stopword query terms.
        let mut q_terms: Vec<String> = Vec::new();
        for t in segment(query) {
            if Self::is_stopword(&t) {
                continue;
            }
            if !q_terms.contains(&t) {
                q_terms.push(t);
            }
        }
        // doc_id -> (accumulated score, matched terms)
        let mut acc: HashMap<u32, (f32, Vec<String>)> = HashMap::new();
        for qt in &q_terms {
            let Some(&tid) = self.term_id.get(qt) else { continue };
            let lo = self.offsets[tid as usize] as usize;
            let hi = self.offsets[tid as usize + 1] as usize;
            let df = hi - lo;
            if df == 0 {
                continue;
            }
            let idf = Self::idf(n, df);
            for &(did, tf) in &self.postings[lo..hi] {
                let dl = self.doc_len[did as usize] as f32;
                let tf = tf as f32;
                // BM25 term contribution.
                let denom = tf + K1 * (1.0 - B + B * dl / self.avg_len);
                let contrib = idf * (tf * (K1 + 1.0)) / denom;
                let e = acc.entry(did).or_insert((0.0, Vec::new()));
                e.0 += contrib;
                e.1.push(qt.clone());
            }
        }
        let mut hits: Vec<ReverseHit> = acc
            .into_iter()
            .map(|(did, (score, matched))| {
                // Q2 damping: a coordination/coverage factor so one rare
                // high-IDF query term cannot carry a hit on its own. A doc that
                // matches more of the DISTINCT query terms is boosted; a doc
                // matching only 1 of, say, 4 query terms is damped hard.
                // coverage = matched_distinct / query_distinct ∈ (0,1].
                // We use coverage^COORD_ALPHA so the effect is strong but a
                // single-term match on a 1-term query is unaffected (coverage=1).
                let q = q_terms.len().max(1) as f32;
                let coverage = (matched.len() as f32 / q).clamp(0.0, 1.0);
                let damped = score * coverage.powf(COORD_ALPHA);
                ReverseHit {
                    word: self.docs[did as usize].clone(),
                    score: damped,
                    matched,
                }
            })
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.word.cmp(&b.word))
        });
        hits.truncate(k);
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A trivial whitespace segmenter for unit tests (real code uses the trie).
    fn ws(s: &str) -> Vec<String> {
        s.split_whitespace().map(|x| x.to_string()).collect()
    }

    #[test]
    fn bm25_finds_the_right_doc_and_reports_matches() {
        let entries = vec![
            ("สุนัข", "สัตว์ เลี้ยง สี่ ขา เห่า ได้"),
            ("แมว", "สัตว์ เลี้ยง สี่ ขา ร้อง เหมียว"),
            ("นก", "สัตว์ ปีก บิน ได้"),
        ];
        let idx = ReverseIndex::build(entries, ws);
        assert_eq!(idx.doc_count(), 3);
        let hits = idx.search("สัตว์ เห่า", 3, ws);
        assert_eq!(hits[0].word, "สุนัข", "เห่า is unique to สุนัข -> ranks first");
        assert!(hits[0].matched.contains(&"เห่า".to_string()));
        assert!(hits[0].matched.contains(&"สัตว์".to_string()));
    }

    #[test]
    fn stopwords_do_not_create_hits() {
        let entries = vec![("ก", "ที่ ของ และ เป็น"), ("ข", "ที่ ของ นก")];
        let idx = ReverseIndex::build(entries, ws);
        // Doc "ก" is all stopwords -> not indexed.
        assert_eq!(idx.doc_count(), 1);
        // Query of only stopwords -> no hits.
        assert!(idx.search("ที่ ของ", 5, ws).is_empty());
        // A content word still hits.
        assert_eq!(idx.search("นก", 5, ws)[0].word, "ข");
    }

    #[test]
    fn empty_query_returns_nothing() {
        let idx = ReverseIndex::build(vec![("ก", "นก บิน")], ws);
        assert!(idx.search("", 5, ws).is_empty());
        assert!(idx.search("ไม่เคยมีคำนี้", 5, ws).is_empty());
    }
}
