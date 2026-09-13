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
pub mod import;
pub mod learner;
pub mod relations;
pub mod segmenter;
pub mod tcc;
pub mod wordnet;

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
    /// ลักษณนาม for the primary sense (may be empty).
    pub classifiers: Vec<String>,
    /// Register marker (โบ/ปาก/ราชา/…) of the primary sense, if any.
    pub register: Option<String>,
    /// Subject field tag of the primary sense, if any.
    pub subject: Option<String>,
    /// Source label of the primary sense (e.g. "Kaikki (Wiktionary)").
    pub source: String,
    /// Licence label of the primary sense.
    pub license: String,
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
        let relations = RelationEngine::from_dictionary_with_wordnet(&dict, &wordnet::WordNet::embedded());
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
        let relations = RelationEngine::from_dictionary_with_wordnet(&dict, &wordnet::WordNet::embedded());
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
            all_words.push(e.headword.clone());
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
        use crate::import::kaikki::KaikkiImporter;
        use crate::import::Importer;

        let words_path = dir.join("words_th.txt");
        let words_txt = std::fs::read_to_string(&words_path)?;
        let mut word_list: Vec<String> = words_txt.lines().map(|s| s.to_string()).collect();

        let freq_path = dir.join("tnc_freq.txt");
        let freq_txt = std::fs::read_to_string(&freq_path).ok();

        log(&format!(
            "loaded {} words from {}{}",
            word_list.len(),
            words_path.display(),
            if freq_txt.is_some() { " (+ frequencies)" } else { "" }
        ));

        // Assemble dictionary entries from all available sources (Task 2/3):
        // seed (always) + Kaikki (if data/kaikki_th.jsonl is present), merged.
        let mut source_lists: Vec<Vec<Entry>> = vec![dictionary::seed_entries()];
        // Real RID data (Task 8) — the organizer's dataset drops into data/rid/.
        // Highest merge priority (Rid > HumanSeed > CoinedWord > Kaikki > Lexitron).
        let rid_dir = dir.join("rid");
        if rid_dir.is_dir() {
            use crate::import::rid::RidImporter;
            let t = Instant::now();
            match RidImporter.load(&rid_dir) {
                Ok(entries) if !entries.is_empty() => {
                    let (n_e, n_s) = (entries.len(), entries.iter().map(|e| e.senses.len()).sum::<usize>());
                    log(&format!(
                        "loaded {n_e} RID entries / {n_s} senses from {} in {:?}",
                        rid_dir.display(),
                        t.elapsed()
                    ));
                    source_lists.push(entries);
                }
                Ok(_) => {}
                Err(e) => log(&format!("warning: RID load failed ({e}); continuing without it")),
            }
        }
        let kaikki_path = dir.join("kaikki_th.jsonl");
        if kaikki_path.exists() {
            let t = Instant::now();
            match KaikkiImporter.load(&kaikki_path) {
                Ok(entries) => {
                    let (n_e, n_s) = (entries.len(), entries.iter().map(|e| e.senses.len()).sum::<usize>());
                    log(&format!(
                        "loaded {n_e} Kaikki entries / {n_s} senses from {} in {:?}",
                        kaikki_path.display(),
                        t.elapsed()
                    ));
                    source_lists.push(entries);
                }
                Err(e) => log(&format!("warning: Kaikki load failed ({e}); continuing without it")),
            }
        }
        // ศัพท์บัญญัติ (ORST coined words) — Task 6. Loaded from the cached HTML
        // in data/coined_word_cache/ if the dir exists and is non-empty (fetched
        // once, offline, by scripts/fetch_coined_word.sh). Never fetches here.
        let coined_dir = dir.join("coined_word_cache");
        if coined_dir.is_dir() {
            use crate::import::coined_word::CoinedWordImporter;
            let has_html = std::fs::read_dir(&coined_dir)
                .map(|mut it| it.any(|e| e.ok().map(|e| e.path().extension().map(|x| x == "html").unwrap_or(false)).unwrap_or(false)))
                .unwrap_or(false);
            if has_html {
                let t = Instant::now();
                match CoinedWordImporter.load(&coined_dir) {
                    Ok(entries) => {
                        let (n_e, n_s) = (entries.len(), entries.iter().map(|e| e.senses.len()).sum::<usize>());
                        log(&format!(
                            "loaded {n_e} ศัพท์บัญญัติ (CoinedWord) entries / {n_s} senses from {} in {:?}",
                            coined_dir.display(),
                            t.elapsed()
                        ));
                        source_lists.push(entries);
                    }
                    Err(e) => log(&format!("warning: ศัพท์บัญญัติ load failed ({e}); continuing without it")),
                }
            }
        }
        let entries = crate::import::merge(source_lists);
        // Extend the segmenter vocab with any entry headwords not already present.
        for e in &entries {
            word_list.push(e.headword.clone());
        }

        let cache_path = dir.join("words_th.datrie.cache");
        // Expected hash of the FULL merged vocab (words_th + all entry headwords).
        let t_hash = Instant::now();
        let expected_hash = crate::segmenter::vocab_hash(word_list.iter());
        log(&format!("vocab_hash ({} words) computed in {:?}", word_list.len(), t_hash.elapsed()));
        if cache_path.exists() {
            let t_load = Instant::now();
            match Segmenter::load_cache_checked(&cache_path, expected_hash) {
                Ok(seg) => {
                    log(&format!("segmenter cache validated + deserialized in {:?}", t_load.elapsed()));
                    let t = Instant::now();
                    let engine = Self::build_from_segmenter(
                        word_list,
                        entries,
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
                // The error message already names the condition (hash mismatch,
                // version mismatch, or corrupt) — log it so a stale-cache rebuild
                // is never silent.
                Err(e) => log(&format!("cache not usable: {e}")),
            }
        }

        log("building trie from scratch (first run is slow, ~40s for 62k words)…");
        let t = Instant::now();
        let engine = Self::build(word_list, entries, freq_txt.as_deref());
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

    /// Definition coverage: (# entries with ≥1 non-empty sense, # searchable
    /// words in the segmenter vocab, percentage). The denominator is the *union*
    /// searchable vocabulary (LEXiTRON words + every merged headword), which is
    /// what a user can actually type — the honest coverage figure.
    pub fn definition_coverage(&self) -> (usize, usize, f64) {
        let defined = self
            .dict
            .all_entries()
            .filter(|e| e.senses.iter().any(|s| !s.definition.trim().is_empty()))
            .count();
        let searchable = self.segmenter.word_count();
        let pct = if searchable == 0 {
            0.0
        } else {
            100.0 * defined as f64 / searchable as f64
        };
        (defined, searchable, pct)
    }

    pub fn relation_entity_count(&self) -> usize {
        self.relations.entity_count()
    }

    pub fn relation_triple_count(&self) -> usize {
        self.relations.triple_count()
    }

    /// Number of distinct sense groups (Sense nodes) in the relation graph.
    pub fn relation_sense_count(&self) -> usize {
        self.relations.sense_count()
    }

    /// Recount of cross-sense candidate pairs in the live relation graph
    /// (A2.4). Must be 0 — every related candidate shares a sense group.
    pub fn cross_sense_pair_count(&self) -> usize {
        self.relations.count_cross_sense_pairs()
    }

    /// Whether `other` appears among `word`'s related results (top_k deep).
    /// Used by the audit recall/absence measurement (A2).
    pub fn related_contains(&self, word: &str, other: &str, top_k: usize) -> bool {
        self.relations.related(word, top_k).iter().any(|r| r.word == other)
    }

    /// All distinct related pairs with cross-source attestation + tier (Phase N).
    pub fn enumerate_relation_pairs(&self) -> Vec<crate::relations::PairInfo> {
        self.relations.enumerate_pairs()
    }

    /// Frequency-weighted definition coverage (A3): of the top-`n` most frequent
    /// Thai words (by `tnc_freq.txt`), how many have a non-empty definition.
    /// Returns (defined, n_considered, pct). This answers "does a judge typing a
    /// *common* word get a definition?" — a more honest signal than raw coverage
    /// over a denominator full of rare inflected forms.
    pub fn frequency_weighted_coverage(&self, n: usize) -> (usize, usize, f64) {
        let ranked = self.dict.freq_ranked_words();
        let considered = ranked.iter().take(n).count();
        let defined = ranked
            .iter()
            .take(n)
            .filter(|w| {
                self.dict
                    .get(w)
                    .map(|e| e.senses.iter().any(|s| !s.definition.trim().is_empty()))
                    .unwrap_or(false)
            })
            .count();
        let pct = if considered == 0 { 0.0 } else { 100.0 * defined as f64 / considered as f64 };
        (defined, considered, pct)
    }

    /// Segment arbitrary Thai text.
    pub fn segment(&self, text: &str) -> Vec<Token> {
        self.segmenter.segment(text)
    }

    /// The full lookup journey for a query word.
    pub fn lookup(&self, query: &str, top_k: usize) -> Lookup {
        let query = query.trim();
        let segmentation = self.segmenter.segment(query);
        let entry = self.dict.get(query).map(|e| {
            let primary = e.senses.first();
            EntryView {
                word: e.headword.clone(),
                pos: e.primary_pos_marker().unwrap_or("").to_string(),
                definition: e.primary_definition().unwrap_or("").to_string(),
                classifiers: primary.map(|s| s.classifiers.clone()).unwrap_or_default(),
                register: primary.and_then(|s| s.register).map(|r| r.marker().to_string()),
                subject: primary.and_then(|s| s.subject.as_ref()).map(|s| s.tag().to_string()),
                source: primary.map(|s| s.provenance.source.label().to_string()).unwrap_or_default(),
                license: primary.map(|s| s.provenance.license.label().to_string()).unwrap_or_default(),
            }
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
/// (Superseded by hash-based invalidation in `load_cache_checked`, Task 5.)
#[allow(dead_code)]
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
