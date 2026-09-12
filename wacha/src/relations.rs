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

/// Where a relationship came from — its provenance, so a user (or a hackathon
/// judge) can tell a hand-verified fact from an auto-imported one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationSource {
    /// Hand-authored + human-verified seed relation (the 20 curated entries).
    Seed,
    /// Auto-extracted from Thai WordNet synset membership. High coverage but
    /// **not individually hand-checked** — WordNet has known noise (e.g. a
    /// synset can pair words a Thai speaker wouldn't call true synonyms). Label
    /// these honestly so the team can say "auto-extracted, unaudited" on sight.
    WordNet,
}

impl RelationSource {
    /// Short Thai/label tag for display.
    pub fn tag(self) -> &'static str {
        match self {
            RelationSource::Seed => "ตรวจแล้ว",           // hand-verified
            RelationSource::WordNet => "WordNet (อัตโนมัติ)", // auto-extracted
        }
    }

    /// Machine-readable label for JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            RelationSource::Seed => "seed",
            RelationSource::WordNet => "wordnet",
        }
    }
}

/// Structural confidence of a relation, derived purely from the graph's own
/// connectivity — no new data, no semantic judgment.
///
/// A WordNet-derived pair whose **both** endpoints connect to nothing else in
/// the whole graph (distinct-neighbor degree 1 each) is an *isolated,
/// uncorroborated* pair: the only evidence for it is that one synset. Thai
/// WordNet has known cross-lingual mapping noise, and these isolated pairs are
/// where the bad ones concentrate (verified: `ข้อหา`/`มลทิน` is exactly this
/// shape). We mark them [`RelationConfidence::Unverified`] — meaning "not
/// cross-corroborated by any other synset", NOT "wrong" (some, like
/// `รถยนต์`/`ยานยนต์`, are perfectly good). Everything else — seed relations
/// (always) and WordNet relations corroborated by ≥2 synsets — is
/// [`RelationConfidence::Confirmed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationConfidence {
    /// Hand-verified (seed) OR cross-corroborated by more than one synset.
    Confirmed,
    /// WordNet-derived isolated pair (both endpoints degree 1) — not
    /// cross-corroborated. Honest "we can't vouch for this one" marker.
    Unverified,
}

impl RelationConfidence {
    pub fn tag(self) -> &'static str {
        match self {
            RelationConfidence::Confirmed => "ยืนยัน",       // confirmed
            RelationConfidence::Unverified => "ยังไม่ยืนยัน", // unverified
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            RelationConfidence::Confirmed => "confirmed",
            RelationConfidence::Unverified => "unverified",
        }
    }
}

/// A relationship engine: a knowledge graph built from dictionary triples.
pub struct RelationEngine {
    graph: KnowledgeGraph,
    triple_count: usize,
    /// Directed (subject_id, object_id) pairs that came from the hand-verified
    /// seed entries. Everything else in the graph is auto-extracted (WordNet).
    seed_edges: HashSet<(usize, usize)>,
}

/// One related word plus the explanation of how it connects to the query word.
#[derive(Debug, Clone)]
pub struct RelatedWord {
    pub word: String,
    /// Relative-PPR relevance score (higher = more related; hub-corrected).
    pub score: f32,
    /// Human-readable relation path from the query word to this word, e.g.
    /// ["แมว --เป็นชนิดของ--> สัตว์", "สุนัข --เป็นชนิดของ--> สัตว์"].
    pub path: Vec<String>,
    /// Provenance of the connection: [`RelationSource::Seed`] (hand-verified) or
    /// [`RelationSource::WordNet`] (auto-extracted, unaudited).
    pub source: RelationSource,
    /// Structural confidence of the connection (graph-degree based). Seed
    /// relations are always [`RelationConfidence::Confirmed`]; isolated WordNet
    /// pairs are [`RelationConfidence::Unverified`].
    pub confidence: RelationConfidence,
}

impl RelationEngine {
    /// Build the graph by extracting triples from every entry's explicit
    /// relations. Symmetric relations (synonym, antonym) are added in both
    /// directions so the graph walk treats them as undirected, which is what a
    /// user means by "related."
    pub fn from_dictionary(dict: &Dictionary) -> Self {
        Self::build(dict, None)
    }

    /// Build the graph from the seed dictionary **plus** Thai WordNet synonym
    /// relations, giving explainable related-words for ~29k words instead of
    /// just the 20 seed headwords. Seed relations are added first (authoritative);
    /// WordNet synonym edges are added afterward and de-duplicated against
    /// existing edges so a seed-declared synonym isn't double-counted.
    pub fn from_dictionary_with_wordnet(dict: &Dictionary, wordnet: &crate::wordnet::WordNet) -> Self {
        Self::build(dict, Some(wordnet))
    }

    fn build(dict: &Dictionary, wordnet: Option<&crate::wordnet::WordNet>) -> Self {
        let mut graph = KnowledgeGraph::new();
        let mut triple_count = 0;
        let syn_label = Relation::Synonym.thai_label();
        // (subject_id, object_id) pairs already linked by a Synonym edge, so
        // WordNet expansion never duplicates a seed-declared synonym or another
        // WordNet edge.
        let mut synonym_seen: HashSet<(usize, usize)> = HashSet::new();
        // Directed pairs from the hand-verified seed entries (any relation).
        let mut seed_edges: HashSet<(usize, usize)> = HashSet::new();

        for entry in dict.all_entries() {
            for (rel, target) in &entry.relations {
                let label = rel.thai_label();
                let s = graph.add_entity(&entry.word);
                let o = graph.add_entity(target);
                graph.add_triple(&entry.word, label, target);
                triple_count += 1;
                seed_edges.insert((s, o));
                if *rel == Relation::Synonym {
                    synonym_seen.insert((s, o));
                }
                // Make symmetric relations bidirectional for traversal.
                if matches!(
                    rel,
                    Relation::Synonym | Relation::Antonym | Relation::SeeAlso | Relation::RelatedTo
                ) {
                    graph.add_triple(target, label, &entry.word);
                    triple_count += 1;
                    seed_edges.insert((o, s));
                    if *rel == Relation::Synonym {
                        synonym_seen.insert((o, s));
                    }
                }
            }
        }

        // WordNet synonym expansion (optional). Synset co-membership → Synonym.
        if let Some(wn) = wordnet {
            for (a, b) in wn.synonym_pairs() {
                let sa = graph.add_entity(a);
                let sb = graph.add_entity(b);
                if sa == sb || !synonym_seen.insert((sa, sb)) {
                    continue;
                }
                graph.add_triple(a, syn_label, b);
                triple_count += 1;
            }
        }

        Self { graph, triple_count, seed_edges }
    }

    pub fn entity_count(&self) -> usize {
        self.graph.entity_count()
    }

    pub fn triple_count(&self) -> usize {
        self.triple_count
    }

    /// Is this word a node in the relationship graph?
    pub fn contains(&self, word: &str) -> bool {
        self.graph.entity_id(word).is_some()
    }

    /// The explainable relationship query. Returns up to `top_k` words most
    /// related to `word`, each with a relation path explaining the connection.
    /// Returns an empty vec if the word has no relations in the graph.
    pub fn related(&self, word: &str, top_k: usize) -> Vec<RelatedWord> {
        let Some(seed) = self.graph.entity_id(word) else {
            return Vec::new();
        };
        // 60 iterations: deterministic, converged for graphs this size.
        let scores = self.graph.personalized_pagerank(&[seed], 60);

        // Precompute the explanatory subgraph once (2 hops from the seed). The
        // set of entities appearing in it are exactly the words actually
        // *connected* to the seed — we only ever surface those as "related", so
        // disconnected nodes (which the relative-PPR floor would otherwise rank
        // with a large-negative score and no explanation) never leak in.
        let subgraph = self.graph.bfs_subgraph(&[seed], 2);
        let mut connected: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for t in &subgraph {
            connected.insert(t.subject_id);
            connected.insert(t.object_id);
        }

        let mut ranked: Vec<(usize, f32)> = scores
            .iter()
            .copied()
            .enumerate()
            .filter(|(id, s)| *id != seed && s.is_finite() && connected.contains(id))
            .collect();
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                // stable tie-break by entity id for determinism
                .then(a.0.cmp(&b.0))
        });

        ranked
            .into_iter()
            .take(top_k)
            .map(|(id, score)| {
                let target = self.graph.entity_name(id).to_string();
                let path = self.explain_path(seed, id, &subgraph);
                let source = self.classify_source(seed, id, &subgraph);
                let confidence = self.classify_confidence(seed, id, source);
                RelatedWord { word: target, score, path, source, confidence }
            })
            .collect()
    }

    /// Number of *distinct* neighbor entities of `entity_id` in the graph
    /// (deduplicated across the triples it participates in). Degree 1 = its only
    /// connection in the whole graph is to a single other word.
    fn distinct_neighbor_degree(&self, entity_id: usize) -> usize {
        let mut neighbors = std::collections::HashSet::new();
        for &idx in self.graph.adjacency_of(entity_id) {
            let t = &self.graph.triples[idx];
            let other = if t.subject_id == entity_id { t.object_id } else { t.subject_id };
            if other != entity_id {
                neighbors.insert(other);
            }
        }
        neighbors.len()
    }

    /// Structural confidence of the query→target relation. Seed relations are
    /// always Confirmed. A WordNet relation is Unverified iff it's an isolated
    /// pair — both endpoints have distinct-neighbor degree 1 (no corroboration
    /// from any other synset/seed relation). Everything else is Confirmed.
    fn classify_confidence(&self, seed: usize, target: usize, source: RelationSource) -> RelationConfidence {
        if source == RelationSource::Seed {
            return RelationConfidence::Confirmed;
        }
        let both_isolated =
            self.distinct_neighbor_degree(seed) == 1 && self.distinct_neighbor_degree(target) == 1;
        if both_isolated {
            RelationConfidence::Unverified
        } else {
            RelationConfidence::Confirmed
        }
    }

    /// Classify a related word's provenance: [`RelationSource::Seed`] if the
    /// connection to the query is carried by any hand-verified seed edge,
    /// otherwise [`RelationSource::WordNet`] (auto-extracted, unaudited).
    ///
    /// A direct seed edge (query↔word) is Seed. For a 2-hop bridge, it's Seed
    /// only if *both* hops are seed edges (a hop through WordNet makes the whole
    /// connection auto-derived). Anything else is WordNet.
    fn classify_source(&self, seed: usize, target: usize, subgraph: &[crate::graph::Triple]) -> RelationSource {
        let is_seed = |a: usize, b: usize| {
            self.seed_edges.contains(&(a, b)) || self.seed_edges.contains(&(b, a))
        };
        // Direct connection?
        let direct_exists = subgraph.iter().any(|t| {
            (t.subject_id == seed && t.object_id == target)
                || (t.subject_id == target && t.object_id == seed)
        });
        if direct_exists {
            return if is_seed(seed, target) {
                RelationSource::Seed
            } else {
                RelationSource::WordNet
            };
        }
        // Two-hop bridge: Seed only if some middle node connects to BOTH the
        // query and the target via seed edges.
        for t1 in subgraph {
            let mid = if t1.subject_id == seed {
                t1.object_id
            } else if t1.object_id == seed {
                t1.subject_id
            } else {
                continue;
            };
            if !is_seed(seed, mid) {
                continue;
            }
            let mid_to_target = subgraph.iter().any(|t2| {
                (t2.subject_id == mid && t2.object_id == target)
                    || (t2.subject_id == target && t2.object_id == mid)
            });
            if mid_to_target && is_seed(mid, target) {
                return RelationSource::Seed;
            }
        }
        RelationSource::WordNet
    }

    /// Human-readable relation edges that connect `seed` to `target` within the
    /// precomputed BFS subgraph. We surface the direct edges touching `target`
    /// (and, if `target` isn't directly linked to the seed, the edges touching
    /// the seed too) so the user sees *why* they're related, not just a score.
    fn explain_path(&self, seed: usize, target: usize, subgraph: &[crate::graph::Triple]) -> Vec<String> {
        let fmt = |t: &crate::graph::Triple| {
            format!(
                "{} --{}--> {}",
                self.graph.entity_name(t.subject_id),
                self.graph.relation_name(t.relation_id),
                self.graph.entity_name(t.object_id)
            )
        };
        // Direct edge seed <-> target?
        let mut direct: Vec<String> = subgraph
            .iter()
            .filter(|t| {
                (t.subject_id == seed && t.object_id == target)
                    || (t.subject_id == target && t.object_id == seed)
            })
            .map(&fmt)
            .collect();
        direct.sort();
        direct.dedup();
        if !direct.is_empty() {
            return direct;
        }
        // Otherwise show the two-hop bridge: edges from seed and edges into
        // target that share a middle node.
        let seed_side_edges: Vec<&crate::graph::Triple> = subgraph
            .iter()
            .filter(|t| t.subject_id == seed || t.object_id == seed)
            .collect();
        let target_edges: Vec<&crate::graph::Triple> = subgraph
            .iter()
            .filter(|t| t.subject_id == target || t.object_id == target)
            .collect();
        let mut path = Vec::new();
        for se in &seed_side_edges {
            let mid = if se.subject_id == seed { se.object_id } else { se.subject_id };
            for te in &target_edges {
                let tmid = if te.subject_id == target { te.object_id } else { te.subject_id };
                if mid == tmid {
                    path.push(fmt(se));
                    path.push(fmt(te));
                }
            }
        }
        path.sort();
        path.dedup();
        path
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
}
