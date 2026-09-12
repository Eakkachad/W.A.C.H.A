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

/// A relationship engine: a knowledge graph built from dictionary triples.
pub struct RelationEngine {
    graph: KnowledgeGraph,
    triple_count: usize,
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
}

impl RelationEngine {
    /// Build the graph by extracting triples from every entry's explicit
    /// relations. Symmetric relations (synonym, antonym) are added in both
    /// directions so the graph walk treats them as undirected, which is what a
    /// user means by "related."
    pub fn from_dictionary(dict: &Dictionary) -> Self {
        let mut graph = KnowledgeGraph::new();
        let mut triple_count = 0;
        for entry in dict.all_entries() {
            for (rel, target) in &entry.relations {
                let label = rel.thai_label();
                graph.add_triple(&entry.word, label, target);
                triple_count += 1;
                // Make symmetric relations bidirectional for traversal.
                if matches!(
                    rel,
                    Relation::Synonym | Relation::Antonym | Relation::SeeAlso | Relation::RelatedTo
                ) {
                    graph.add_triple(target, label, &entry.word);
                    triple_count += 1;
                }
            }
        }
        Self { graph, triple_count }
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
                RelatedWord { word: target, score, path }
            })
            .collect()
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
        let seed_edges: Vec<&crate::graph::Triple> = subgraph
            .iter()
            .filter(|t| t.subject_id == seed || t.object_id == seed)
            .collect();
        let target_edges: Vec<&crate::graph::Triple> = subgraph
            .iter()
            .filter(|t| t.subject_id == target || t.object_id == target)
            .collect();
        let mut path = Vec::new();
        for se in &seed_edges {
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
}
