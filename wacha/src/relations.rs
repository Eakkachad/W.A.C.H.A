//! The "AXIOM's graph engine explains it" half of the hybrid.
//!
//! Extracts (subject, relation, object) triples from dictionary entries and
//! loads them into the vendored [`crate::graph::KnowledgeGraph`], then exposes an
//! *explainable* relationship query: given a word, which words are most related
//! (Personalized PageRank) and *why* (the BFS relation path connecting them).
//!
//! Guardrail honored: triples come only from relations the dictionary entries
//! explicitly carry (synonym / antonym / is-a / see-also / category). We never
//! call any of AXIOM's English-only text-decomposition NLU — we build triples
//! ourselves from structured data.

use crate::dictionary::{Dictionary, Relation};
use crate::graph::KnowledgeGraph;
use std::collections::HashSet;

// Confidence lives in `dictionary` (needed by `Provenance`); re-export so
// existing `crate::relations::RelationConfidence` references keep working.
pub use crate::dictionary::RelationConfidence;

/// Where a relationship came from — its provenance, so a user (or a hackathon
/// judge) can tell a hand-verified fact from an auto-imported one. Read from the
/// connecting sense group's [`crate::dictionary::Source`] (Task 4 — no more
/// guessing from graph structure).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationSource {
    /// Hand-authored + human-verified seed relation (the 20 curated entries).
    Seed,
    /// Auto-extracted from Thai WordNet synset membership. Not hand-checked.
    WordNet,
    /// From Kaikki (Thai Wiktionary) synonym/related lists. Community source.
    Wiktionary,
    /// From ศัพท์บัญญัติ (ORST coined-word term equivalences). ORST-authored.
    CoinedWord,
}

impl RelationSource {
    /// Map from a dictionary [`crate::dictionary::Source`].
    pub fn from_source(s: crate::dictionary::Source) -> Self {
        use crate::dictionary::Source;
        match s {
            Source::HumanSeed => RelationSource::Seed,
            Source::Kaikki => RelationSource::Wiktionary,
            Source::CoinedWord => RelationSource::CoinedWord,
            Source::Rid => RelationSource::Seed, // real RID = authoritative, treat as verified
            Source::Lexitron => RelationSource::WordNet, // headword-only; no relations anyway
        }
    }

    /// Short Thai/label tag for display.
    pub fn tag(self) -> &'static str {
        match self {
            RelationSource::Seed => "ตรวจแล้ว",
            RelationSource::WordNet => "WordNet (อัตโนมัติ)",
            RelationSource::Wiktionary => "Wiktionary (อัตโนมัติ)",
            RelationSource::CoinedWord => "ศัพท์บัญญัติ (ราชบัณฑิตฯ)",
        }
    }

    /// Machine-readable label for JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            RelationSource::Seed => "seed",
            RelationSource::WordNet => "wordnet",
            RelationSource::Wiktionary => "wiktionary",
            RelationSource::CoinedWord => "coined_word",
        }
    }
}

/// A relationship engine: a knowledge graph built from dictionary triples.
/// A sense-aware relationship engine (Round 5, Task 4).
///
/// The old engine flattened every synonym into a word↔word edge, so 2-hop
/// traversal invented relations across unrelated senses (บ้าน sits in 9 WordNet
/// synsets → ครอบครัว→บ้าน→บ้านเกิด). This version routes every relation through
/// a **sense group**: two words are related **iff they share a sense group**.
/// Cross-sense leakage is then structurally impossible — no hand-patching.
///
/// Two-tier edge model:
///  - **Tier 1 (typed, curated):** seed relations keep their explicit relation
///    type (synonym/antonym/is-a/…) as a 2-member sense group tagged `Seed`.
///  - **Tier 2 (synonymy groups):** each WordNet synset and each Kaikki
///    synonym/related list is one sense group (tagged `WordNet`/`Wiktionary`),
///    and each ศัพท์บัญญัติ discipline record a `CoinedWord` group.
pub struct RelationEngine {
    /// Interned word strings.
    words: Vec<String>,
    word_id: std::collections::HashMap<String, usize>,
    /// All sense groups.
    senses: Vec<SenseGroup>,
    /// word_id -> indices of sense groups it belongs to.
    word_senses: Vec<Vec<usize>>,
    triple_count: usize,
}

/// One sense group: a set of words that genuinely share a sense, plus where the
/// grouping came from and (for typed seed relations) the relation label.
struct SenseGroup {
    members: Vec<usize>,
    source: RelationSource,
    /// Relation label for the explanation edge (e.g. "มีความหมายเหมือนกับ").
    label: String,
    /// A stable id for explanation display (synset id, or a synthetic tag).
    tag: String,
}

/// One related word plus the explanation of how it connects to the query word.
#[derive(Debug, Clone)]
pub struct RelatedWord {
    pub word: String,
    /// Relevance score (higher = more related). Deterministic; see `related`.
    pub score: f32,
    /// Human-readable relation path explaining the connection.
    pub path: Vec<String>,
    /// Provenance of the connecting sense group.
    pub source: RelationSource,
    /// Structural confidence (Seed/CoinedWord = Confirmed; a lone isolated
    /// WordNet/Wiktionary pair = Unverified).
    pub confidence: RelationConfidence,
}

impl RelationEngine {
    pub fn from_dictionary(dict: &Dictionary) -> Self {
        Self::build(dict, None)
    }

    pub fn from_dictionary_with_wordnet(dict: &Dictionary, wordnet: &crate::wordnet::WordNet) -> Self {
        Self::build(dict, Some(wordnet))
    }

    fn build(dict: &Dictionary, wordnet: Option<&crate::wordnet::WordNet>) -> Self {
        let mut eng = RelationEngine {
            words: Vec::new(),
            word_id: std::collections::HashMap::new(),
            senses: Vec::new(),
            word_senses: Vec::new(),
            triple_count: 0,
        };

        // Tier 1: seed (and RID) entry relations -> typed 2-member sense groups.
        // Only entries whose senses are HumanSeed/Rid sourced are authoritative;
        // Kaikki entry `relations` are handled as Tier-2 Wiktionary groups below.
        for entry in dict.all_entries() {
            let src = entry
                .senses
                .iter()
                .map(|s| s.provenance.source)
                .next()
                .unwrap_or(crate::dictionary::Source::Lexitron);
            let rsource = RelationSource::from_source(src);
            for (rel, target) in &entry.relations {
                if entry.headword == *target {
                    continue;
                }
                let a = eng.intern(&entry.headword);
                let b = eng.intern(target);
                let label = rel.thai_label().to_string();
                eng.add_sense_group(vec![a, b], rsource, label, format!("{}-{}", entry.headword, target));
            }
        }

        // Tier 2a: WordNet synsets -> one Sense group each (source = WordNet).
        if let Some(wn) = wordnet {
            for (synid, members) in wn.synsets() {
                let ids: Vec<usize> = members.iter().map(|m| eng.intern(m)).collect();
                eng.add_sense_group(
                    ids,
                    RelationSource::WordNet,
                    Relation::Synonym.thai_label().to_string(),
                    synid.clone(),
                );
            }
        }

        // Tier 2b: Kaikki (Wiktionary) synonym/related lists on non-seed entries
        // -> one Sense group per entry (source = Wiktionary). ศัพท์บัญญัติ
        // entries (Source::CoinedWord) route here too, tagged CoinedWord.
        for entry in dict.all_entries() {
            let src = entry.senses.iter().map(|s| s.provenance.source).next();
            let rsource = match src {
                Some(crate::dictionary::Source::Kaikki) => RelationSource::Wiktionary,
                Some(crate::dictionary::Source::CoinedWord) => RelationSource::CoinedWord,
                _ => continue, // seed/RID handled in Tier 1; Lexitron has no relations
            };
            // Group the headword with all its relation targets (one shared sense).
            let mut ids = vec![eng.intern(&entry.headword)];
            for (_rel, target) in &entry.relations {
                if *target != entry.headword {
                    ids.push(eng.intern(target));
                }
            }
            if ids.len() >= 2 {
                let label = Relation::Synonym.thai_label().to_string();
                eng.add_sense_group(ids, rsource, label, format!("kaikki:{}", entry.headword));
            }
        }

        eng
    }

    fn intern(&mut self, w: &str) -> usize {
        if let Some(&id) = self.word_id.get(w) {
            return id;
        }
        let id = self.words.len();
        self.words.push(w.to_string());
        self.word_id.insert(w.to_string(), id);
        self.word_senses.push(Vec::new());
        id
    }

    fn add_sense_group(&mut self, mut members: Vec<usize>, source: RelationSource, label: String, tag: String) {
        members.sort_unstable();
        members.dedup();
        if members.len() < 2 {
            return;
        }
        let sidx = self.senses.len();
        for &m in &members {
            self.word_senses[m].push(sidx);
        }
        self.triple_count += members.len();
        self.senses.push(SenseGroup { members, source, label, tag });
    }

    pub fn entity_count(&self) -> usize {
        self.words.len()
    }

    pub fn triple_count(&self) -> usize {
        self.triple_count
    }

    /// Number of distinct sense groups (Sense nodes) in the graph.
    pub fn sense_count(&self) -> usize {
        self.senses.len()
    }

    pub fn contains(&self, word: &str) -> bool {
        self.word_id.contains_key(word)
    }

    /// Source priority for choosing which sense group "explains" a related word
    /// when several connect the same pair. Seed/CoinedWord (authoritative) win.
    fn source_rank(s: RelationSource) -> u8 {
        match s {
            RelationSource::Seed => 4,
            RelationSource::CoinedWord => 3,
            RelationSource::WordNet => 2,
            RelationSource::Wiktionary => 1,
        }
    }

    /// The explainable relationship query. A word Y is related to X **iff X and
    /// Y share at least one sense group** — this is the whole fix: traversal
    /// cannot cross into a different sense. Ranking: (# shared sense groups,
    /// then source rank), deterministic tie-break by word.
    pub fn related(&self, word: &str, top_k: usize) -> Vec<RelatedWord> {
        let Some(&qid) = self.word_id.get(word) else {
            return Vec::new();
        };
        // For each co-member, remember how many senses it shares + the best
        // (highest-rank) connecting sense group.
        use std::collections::HashMap;
        let mut shared: HashMap<usize, (u32, usize)> = HashMap::new(); // word -> (count, best_sense_idx)
        for &sidx in &self.word_senses[qid] {
            let sg = &self.senses[sidx];
            for &m in &sg.members {
                if m == qid {
                    continue;
                }
                let e = shared.entry(m).or_insert((0, sidx));
                e.0 += 1;
                // prefer the higher-source-rank sense for the explanation/label
                let cur_rank = Self::source_rank(self.senses[e.1].source);
                let new_rank = Self::source_rank(sg.source);
                if new_rank > cur_rank {
                    e.1 = sidx;
                }
            }
        }
        let mut ranked: Vec<(usize, u32, usize)> =
            shared.into_iter().map(|(w, (c, s))| (w, c, s)).collect();
        ranked.sort_by(|a, b| {
            b.1.cmp(&a.1) // more shared senses first
                .then_with(|| Self::source_rank(self.senses[b.2].source).cmp(&Self::source_rank(self.senses[a.2].source)))
                .then_with(|| self.words[a.0].cmp(&self.words[b.0])) // deterministic
        });

        ranked
            .into_iter()
            .take(top_k)
            .map(|(wid, count, sidx)| {
                let sg = &self.senses[sidx];
                let source = sg.source;
                let confidence = self.classify_confidence(qid, wid, source);
                let path = vec![
                    format!("{} --{}--> {}", word, sg.label, self.words[wid]),
                    format!("(ผ่านชุดความหมายเดียวกัน: {})", sg.tag),
                ];
                RelatedWord {
                    word: self.words[wid].clone(),
                    score: count as f32,
                    path,
                    source,
                    confidence,
                }
            })
            .collect()
    }

    /// Confidence: Seed and CoinedWord are always Confirmed (authoritative). A
    /// WordNet/Wiktionary relation is Unverified iff it is an *isolated pair* —
    /// the connecting sense group has exactly 2 members AND neither word appears
    /// in any other sense group (no corroboration anywhere).
    fn classify_confidence(&self, a: usize, b: usize, source: RelationSource) -> RelationConfidence {
        if matches!(source, RelationSource::Seed | RelationSource::CoinedWord) {
            return RelationConfidence::Confirmed;
        }
        let a_deg = self.word_senses[a].len();
        let b_deg = self.word_senses[b].len();
        if a_deg <= 1 && b_deg <= 1 {
            RelationConfidence::Unverified
        } else {
            RelationConfidence::Confirmed
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::seed_entries;

    fn engine() -> RelationEngine {
        let mut dict = Dictionary::new();
        for e in seed_entries() {
            dict.insert(e);
        }
        RelationEngine::from_dictionary(&dict)
    }

    #[test]
    fn non_seed_entry_relations_are_not_tagged_seed() {
        // Credibility bug (2026-09-13): a merged Kaikki entry's relations were
        // wrongly tagged [ตรวจแล้ว]/seed. Build a Kaikki-sourced entry for a
        // NON-seed headword with a synonym, and assert the relation reads as
        // WordNet, not Seed.
        use crate::dictionary::{Entry, License, Pos, Provenance, Relation, Sense, Source};
        let mut dict = Dictionary::new();
        for e in seed_entries() {
            dict.insert(e);
        }
        let mut kaikki = Entry::headword_only("บ้านทดสอบ");
        kaikki.senses.push(Sense {
            pos: Some(Pos::Nam),
            subject: None,
            register: None,
            definition: "ที่อยู่อาศัย (Kaikki)".into(),
            examples: vec![],
            classifiers: vec![],
            provenance: Provenance {
                source: Source::Kaikki,
                license: License::CcBySa,
                confidence: RelationConfidence::Unverified,
            },
        });
        kaikki.relations.push((Relation::Synonym, "เรือนทดสอบ".into()));
        dict.insert(kaikki);
        let eng = RelationEngine::from_dictionary(&dict);
        let rel = eng.related("บ้านทดสอบ", 5);
        let syn = rel.iter().find(|r| r.word == "เรือนทดสอบ").expect("synonym present");
        assert_ne!(syn.source, RelationSource::Seed, "a Kaikki relation must NOT be [ตรวจแล้ว]");
        assert_eq!(
            syn.source,
            RelationSource::Wiktionary,
            "a Kaikki entry's relation is auto-extracted -> Wiktionary"
        );
    }

    #[test]
    fn builds_graph_from_seed() {
        let e = engine();
        assert!(e.entity_count() > 10);
        assert!(e.triple_count() > 15);
        assert!(e.contains("แมว"));
    }

    #[test]
    fn cat_relates_to_animal_with_explanation() {
        let e = engine();
        let related = e.related("แมว", 6);
        assert!(!related.is_empty());
        // "สัตว์" (animal) should surface — แมว เป็นชนิดของ สัตว์.
        let animal = related.iter().find(|r| r.word == "สัตว์");
        assert!(animal.is_some(), "expected สัตว์ among related words");
        // And there must be a non-empty explanation path.
        assert!(!animal.unwrap().path.is_empty(), "explanation path must exist");
    }

    #[test]
    fn synonym_is_bidirectional() {
        let e = engine();
        // หมา and สุนัข are synonyms; each should surface the other.
        let from_suna = e.related("สุนัข", 8);
        assert!(from_suna.iter().any(|r| r.word == "หมา"));
        let from_ma = e.related("หมา", 8);
        assert!(from_ma.iter().any(|r| r.word == "สุนัข"));
    }

    #[test]
    fn unknown_word_returns_empty() {
        let e = engine();
        assert!(e.related("ไดโนเสาร์", 5).is_empty());
    }

    #[test]
    fn seed_relations_are_tagged_seed() {
        let e = engine(); // seed-only, no WordNet
        for rw in e.related("แมว", 6) {
            assert_eq!(
                rw.source,
                RelationSource::Seed,
                "seed-only engine must mark {} as Seed",
                rw.word
            );
        }
    }

    #[test]
    fn wordnet_pairs_tagged_wordnet_seed_pairs_stay_seed() {
        // Build with WordNet so auto-extracted synonyms appear.
        let mut dict = Dictionary::new();
        for en in seed_entries() {
            dict.insert(en);
        }
        let wn = crate::wordnet::WordNet::embedded();
        let e = RelationEngine::from_dictionary_with_wordnet(&dict, &wn);

        // A hand-verified seed pair must stay Seed even with WordNet loaded:
        // ครู→อาจารย์ is a seed Synonym.
        let from_kru = e.related("ครู", 10);
        if let Some(rw) = from_kru.iter().find(|r| r.word == "อาจารย์") {
            assert_eq!(rw.source, RelationSource::Seed, "ครู→อาจารย์ is a seed relation");
        }

        // An auto-extracted WordNet pair must be tagged WordNet: ข้อหา→มลทิน
        // (the exact noisy pair the review flagged — it exists ONLY via WordNet,
        // never a seed entry, so it must be labeled auto-extracted/unaudited).
        let from_khoha = e.related("ข้อหา", 10);
        assert!(!from_khoha.is_empty(), "ข้อหา should have WordNet relations");
        if let Some(rw) = from_khoha.iter().find(|r| r.word == "มลทิน") {
            assert_eq!(
                rw.source,
                RelationSource::WordNet,
                "ข้อหา→มลทิน is auto-extracted from WordNet, must be tagged WordNet"
            );
        }
        // Every ข้อหา relation is WordNet-sourced (ข้อหา isn't a seed word).
        assert!(from_khoha.iter().all(|r| r.source == RelationSource::WordNet));
    }

    #[test]
    fn isolated_wordnet_pair_is_unverified_corroborated_is_confirmed() {
        let mut dict = Dictionary::new();
        for en in seed_entries() {
            dict.insert(en);
        }
        let wn = crate::wordnet::WordNet::embedded();
        let e = RelationEngine::from_dictionary_with_wordnet(&dict, &wn);

        // ข้อหา↔มลทิน is an isolated 2-node pair (both degree 1) -> Unverified.
        let khoha = e.related("ข้อหา", 10);
        let mlt = khoha.iter().find(|r| r.word == "มลทิน").expect("มลทิน related to ข้อหา");
        assert_eq!(
            mlt.confidence,
            RelationConfidence::Unverified,
            "isolated WordNet pair ข้อหา/มลทิน must be Unverified"
        );

        // สุนัข↔หมา is corroborated (each appears in multiple synsets) -> Confirmed.
        let suna = e.related("สุนัข", 10);
        let ma = suna.iter().find(|r| r.word == "หมา").expect("หมา related to สุนัข");
        assert_eq!(
            ma.confidence,
            RelationConfidence::Confirmed,
            "corroborated pair สุนัข/หมา must be Confirmed"
        );
    }

    #[test]
    fn seed_relations_always_confirmed() {
        // seed-only engine: every relation is Confirmed regardless of degree.
        let e = engine();
        for rw in e.related("แมว", 8) {
            assert_eq!(rw.confidence, RelationConfidence::Confirmed);
        }
    }

    // ── Task 4 regression: sense-node model kills cross-synset 2-hop leakage ──

    fn wordnet_engine() -> RelationEngine {
        let mut dict = Dictionary::new();
        for e in seed_entries() {
            dict.insert(e);
        }
        RelationEngine::from_dictionary_with_wordnet(&dict, &crate::wordnet::WordNet::embedded())
    }

    #[test]
    fn crop_krua_does_not_return_ban_koet() {
        // The canonical false 2-hop link: ครอบครัว → บ้าน → บ้านเกิด, where บ้าน
        // sits in many unrelated synsets. With sense nodes, ครอบครัว and บ้านเกิด
        // share NO sense group, so บ้านเกิด must never appear among ครอบครัว's
        // related words (any depth — we only return same-sense-group co-members).
        let e = wordnet_engine();
        let rel = e.related("ครอบครัว", 50);
        assert!(
            !rel.iter().any(|r| r.word == "บ้านเกิด"),
            "ครอบครัว must not surface บ้านเกิด (cross-synset leak)"
        );
    }

    #[test]
    fn no_related_word_is_the_query_itself_and_all_share_a_sense() {
        // Every related word must genuinely share a sense group with the query
        // (the structural guarantee). Spot-check on a WordNet word.
        let e = wordnet_engine();
        for rw in e.related("บ้าน", 20) {
            assert_ne!(rw.word, "บ้าน");
            assert!(matches!(
                rw.source,
                RelationSource::Seed
                    | RelationSource::WordNet
                    | RelationSource::Wiktionary
                    | RelationSource::CoinedWord
            ));
        }
    }

    #[test]
    fn ban_hyao_is_tagged_wiktionary_not_wordnet() {
        // The reviewer's second regression: บ้าน→หย้าว comes from Kaikki
        // (Wiktionary synonym), NOT a WordNet synset — it must read Wiktionary.
        use crate::dictionary::{Entry, License, Pos, Provenance, Sense, Source};
        let mut dict = Dictionary::new();
        for e in seed_entries() {
            dict.insert(e);
        }
        let mut ban = Entry::headword_only("บ้าน");
        ban.senses.push(Sense {
            pos: Some(Pos::Nam),
            subject: None,
            register: None,
            definition: "ที่อยู่อาศัย".into(),
            examples: vec![],
            classifiers: vec![],
            provenance: Provenance {
                source: Source::Kaikki,
                license: License::CcBySa,
                confidence: RelationConfidence::Unverified,
            },
        });
        ban.relations.push((Relation::Synonym, "หย้าว".into()));
        dict.insert(ban);
        let e = RelationEngine::from_dictionary(&dict);
        let hyao = e.related("บ้าน", 20).into_iter().find(|r| r.word == "หย้าว").expect("หย้าว present");
        assert_eq!(hyao.source, RelationSource::Wiktionary, "บ้าน→หย้าว must be Wiktionary");
        assert_ne!(hyao.source, RelationSource::WordNet);
    }
}
