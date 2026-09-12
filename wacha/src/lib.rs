//! # วาจา (WACHA) — Word Architecture, Cluster-aware Hybrid Analysis
//!
//! Hybrid Thai dictionary engine for the ORST "เปิดคลังคำ พลิกคลังคิด" hackathon.
//!
//! Two of the user's own research assets, each vendored as a single
//! self-contained module (not a live dependency — this crate builds with
//! nothing outside this repo) and used for what it's good at:
//! - **Segmentation** ([`segmenter`]) — the vendored [`datrie::DatrieVocab`]
//!   double-array trie (originally `katgpt-tokenizer`, MIT-licensed) drives
//!   greedy longest-match Thai word segmentation, built directly from the
//!   dictionary's own word list. OOV text falls back to whole Thai Character
//!   Clusters ([`tcc`]), never raw codepoints.
//! - **Explainable relationships** ([`relations`]) — AXIOM's vendored
//!   [`graph::KnowledgeGraph`] (triple-store + Personalized PageRank + BFS) over
//!   dictionary-derived triples answers "how are these words related, and why."
//!
//! [`Engine`] is the facade that stitches them into one user journey:
//! *type a word → segment it → look up its definition → see related words with an
//! explanation of the connection.*

pub mod datrie;
pub mod dictionary;
pub mod graph;
pub mod learner;
pub mod relations;
pub mod segmenter;
pub mod tcc;

use dictionary::{Dictionary, Entry};
use learner::{LearnerContent, LearnerStore};
use relations::{RelatedWord, RelationEngine};
use segmenter::{Segmenter, Token};

/// The full-journey facade over the three subsystems.
pub struct Engine {
    dict: Dictionary,
    segmenter: Segmenter,
    relations: RelationEngine,
    learner: LearnerStore,
}

/// The result of the full lookup journey for one query word.
#[derive(Debug, Clone)]
pub struct Lookup {
    /// The query, segmented into tokens (in-vocab words vs OOV clusters).
    pub segmentation: Vec<Token>,
    /// The dictionary entry for the query word, if the query is a single known
    /// headword (definition + pos).
    pub entry: Option<EntryView>,
    /// Related words with explanations (may be empty).
    pub related: Vec<RelatedWord>,
    /// Offline-precomputed learner content (คำอธิบายง่าย + example), if any.
    /// A clearly-labeled enrichment — never overrides `entry`'s formal
    /// definition, and carries its own honest provenance label.
    pub learner: Option<LearnerContent>,
}

/// A display-friendly view of a dictionary entry.
#[derive(Debug, Clone)]
pub struct EntryView {
    pub word: String,
    pub pos: String,
    pub definition: String,
}

impl Engine {
    /// Build an engine from a word list (drives segmentation), dictionary
    /// entries (definitions + relations), and an optional frequency table.
    ///
    /// The segmenter's word list is the union of `word_list` and every
    /// dictionary headword, so known entries always segment as whole words.
    ///
    /// Note: building the segmenter's trie from the full 62k-word list is the
    /// expensive step (~43s). Use [`Engine::build_with_cache`] for a demo to
    /// pay that cost only once and load from disk thereafter.
    pub fn build<I, S>(word_list: I, entries: Vec<Entry>, freq_text: Option<&str>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let (all_words, dict) = Self::assemble_dict(word_list, entries, freq_text);
        let segmenter = Segmenter::from_words(all_words);
        let relations = RelationEngine::from_dictionary(&dict);
        Self { dict, segmenter, relations, learner: LearnerStore::embedded() }
    }

    /// Same as [`Engine::build`], but reuses an already-built (e.g. disk-cached)
    /// `Segmenter` instead of reconstructing the trie. The caller is responsible
    /// for ensuring the cached segmenter's word list matches `word_list` +
    /// entries (the CLI does this by keying the cache on the word-list file).
    pub fn build_from_segmenter<I, S>(
        word_list: I,
        entries: Vec<Entry>,
        freq_text: Option<&str>,
        segmenter: Segmenter,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // `word_list` is consumed only to build the dictionary side; the trie
        // comes from the supplied segmenter. We still drain the iterator so the
        // dictionary headwords + entries are registered.
        let (_all_words, dict) = Self::assemble_dict(word_list, entries, freq_text);
        let relations = RelationEngine::from_dictionary(&dict);
        Self { dict, segmenter, relations, learner: LearnerStore::embedded() }
    }

    /// Shared helper: fold a word list + entries + optional freq table into the
    /// combined word vector (for the segmenter) and the populated `Dictionary`.
    fn assemble_dict<I, S>(
        word_list: I,
        entries: Vec<Entry>,
        freq_text: Option<&str>,
    ) -> (Vec<String>, Dictionary)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut dict = Dictionary::new();
        let mut all_words: Vec<String> = Vec::new();
        for w in word_list {
            all_words.push(w.as_ref().to_string());
        }
        for e in entries {
            all_words.push(e.word.clone());
            dict.insert(e);
        }
        if let Some(ft) = freq_text {
            dict.load_frequencies(ft);
        }
        (all_words, dict)
    }

    /// A compact demo-ready engine: the curated seed entries only (no external
    /// data files needed). Good for tests and offline demos.
    pub fn seed_only() -> Self {
        Self::build(Vec::<String>::new(), dictionary::seed_entries(), None)
    }

    /// Load an engine from a data directory containing `words_th.txt` (and
    /// optionally `tnc_freq.txt`), using an on-disk trie cache
    /// (`words_th.datrie.cache`) to skip the ~43s rebuild when it's present and
    /// newer than the word list. This is the shared entry point for both the
    /// CLI and the web server — build the engine *once* at startup with it.
    ///
    /// `log` receives human-readable progress lines (pass e.g. `|m| eprintln!("{m}")`
    /// or a no-op closure).
    pub fn load_from_dir(dir: &std::path::Path, mut log: impl FnMut(&str)) -> std::io::Result<Self> {
        use std::time::Instant;

        let words_path = dir.join("words_th.txt");
        let words_txt = std::fs::read_to_string(&words_path)?;
        let word_list: Vec<&str> = words_txt.lines().collect();

        let freq_path = dir.join("tnc_freq.txt");
        let freq_txt = std::fs::read_to_string(&freq_path).ok();

        log(&format!(
            "loaded {} words from {}{}",
            word_list.len(),
            words_path.display(),
            if freq_txt.is_some() { " (+ frequencies)" } else { "" }
        ));

        let cache_path = dir.join("words_th.datrie.cache");
        if cache_is_fresh(&cache_path, &words_path) {
            match Segmenter::load_cache(&cache_path) {
                Ok(seg) => {
                    let t = Instant::now();
                    let engine = Self::build_from_segmenter(
                        word_list,
                        dictionary::seed_entries(),
                        freq_txt.as_deref(),
                        seg,
                    );
                    log(&format!(
                        "loaded segmenter from cache {} in {:?} (skipped ~43s trie build)",
                        cache_path.display(),
                        t.elapsed()
                    ));
                    return Ok(engine);
                }
                Err(e) => log(&format!("cache load failed ({e}); rebuilding from scratch")),
            }
        }

        log("building trie from scratch (first run is slow, ~40s for 62k words)…");
        let t = Instant::now();
        let engine = Self::build(word_list, dictionary::seed_entries(), freq_txt.as_deref());
        log(&format!("engine built in {:?}", t.elapsed()));
        match engine.segmenter().save_cache(&cache_path) {
            Ok(()) => log(&format!("wrote segmenter cache to {}", cache_path.display())),
            Err(e) => log(&format!("warning: could not write cache ({e})")),
        }
        Ok(engine)
    }

    pub fn segmenter(&self) -> &Segmenter {
        &self.segmenter
    }

    pub fn word_count(&self) -> usize {
        self.segmenter.word_count()
    }

    pub fn entry_count(&self) -> usize {
        self.dict.len()
    }

    pub fn relation_entity_count(&self) -> usize {
        self.relations.entity_count()
    }

    pub fn relation_triple_count(&self) -> usize {
        self.relations.triple_count()
    }

    /// Segment arbitrary Thai text.
    pub fn segment(&self, text: &str) -> Vec<Token> {
        self.segmenter.segment(text)
    }

    /// The full lookup journey for a query word.
    pub fn lookup(&self, query: &str, top_k: usize) -> Lookup {
        let query = query.trim();
        let segmentation = self.segmenter.segment(query);
        let entry = self.dict.get(query).map(|e| EntryView {
            word: e.word.clone(),
            pos: e.pos.clone(),
            definition: e.definition.clone(),
        });
        let related = self.relations.related(query, top_k);
        let learner = self.learner.get(query).cloned();
        Lookup { segmentation, entry, related, learner }
    }

    /// Number of words with offline-precomputed learner content loaded.
    pub fn learner_count(&self) -> usize {
        self.learner.len()
    }
}

/// A trie cache is usable if it exists and is at least as new as the word-list
/// file it was built from (editing the word list invalidates a stale cache).
fn cache_is_fresh(cache_path: &std::path::Path, words_path: &std::path::Path) -> bool {
    let (Ok(cache_meta), Ok(words_meta)) =
        (std::fs::metadata(cache_path), std::fs::metadata(words_path))
    else {
        return false;
    };
    match (cache_meta.modified(), words_meta.modified()) {
        (Ok(cache_m), Ok(words_m)) => cache_m >= words_m,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_engine_full_journey() {
        let engine = Engine::seed_only();
        let r = engine.lookup("แมว", 5);
        // segmentation recognizes it as a single in-vocab word
        assert_eq!(r.segmentation.len(), 1);
        assert!(r.segmentation[0].in_vocab);
        // definition present
        assert!(r.entry.is_some());
        assert!(r.entry.unwrap().definition.contains("สัตว์"));
        // related words with explanations
        assert!(!r.related.is_empty());
        assert!(r.related.iter().any(|w| !w.path.is_empty()));
    }

    #[test]
    fn oov_query_still_segments_without_shattering() {
        let engine = Engine::seed_only();
        let r = engine.lookup("เด็กน้อย", 5);
        // No orphaned marks in the segmentation (the Day-0 bug).
        for t in &r.segmentation {
            assert_ne!(t.text, "็");
            assert_ne!(t.text, "้");
            assert_ne!(t.text, "เ");
        }
    }
}
