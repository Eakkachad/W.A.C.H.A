//! # วาจา (WACHA) — Word Architecture, Cluster-aware Hybrid Analysis
//!
//! Hybrid Thai dictionary engine for the ORST "เปิดคลังคำ พลิกคลังคิด" hackathon.
//!
//! Two of the user's own research assets, each used for what it's good at:
//! - **Segmentation** ([`segmenter`]) — `katgpt-tokenizer`'s `Datrie` double-array
//!   trie drives greedy longest-match Thai word segmentation, built directly from
//!   the dictionary's own word list. OOV text falls back to whole Thai Character
//!   Clusters ([`tcc`]), never raw codepoints.
//! - **Explainable relationships** ([`relations`]) — AXIOM's vendored
//!   [`graph::KnowledgeGraph`] (triple-store + Personalized PageRank + BFS) over
//!   dictionary-derived triples answers "how are these words related, and why."
//!
//! [`Engine`] is the facade that stitches them into one user journey:
//! *type a word → segment it → look up its definition → see related words with an
//! explanation of the connection.*

pub mod dictionary;
pub mod graph;
pub mod relations;
pub mod segmenter;
pub mod tcc;

use dictionary::{Dictionary, Entry};
use relations::{RelatedWord, RelationEngine};
use segmenter::{Segmenter, Token};

/// The full-journey facade over the three subsystems.
pub struct Engine {
    dict: Dictionary,
    segmenter: Segmenter,
    relations: RelationEngine,
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
    pub fn build<I, S>(word_list: I, entries: Vec<Entry>, freq_text: Option<&str>) -> Self
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
        let segmenter = Segmenter::from_words(all_words);
        let relations = RelationEngine::from_dictionary(&dict);
        Self { dict, segmenter, relations }
    }

    /// A compact demo-ready engine: the curated seed entries only (no external
    /// data files needed). Good for tests and offline demos.
    pub fn seed_only() -> Self {
        Self::build(Vec::<String>::new(), dictionary::seed_entries(), None)
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
        Lookup { segmentation, entry, related }
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
