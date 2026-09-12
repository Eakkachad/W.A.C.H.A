//! Thai WordNet synonym relations.
//!
//! Expands explainable relationship coverage far beyond the 20 hand-curated
//! seed words: ~29k Thai words gain real, sourced synonym relations.
//!
//! **Provenance & honesty:** the data is derived from Thai WordNet
//! (`wordnet_th.db`, Thai Computational Linguistic Laboratory / NICT, permissive
//! license — verified 2026-09-04). We ship only the derived, compact synonym
//! groups (`data/wordnet_synonyms.tsv`, ~1MB), not the 11MB source SQLite DB.
//!
//! **What we extract, and what we deliberately do NOT.** `wordnet_th.db`'s only
//! table is `word_synset(synsetid, li)` — i.e. *synset membership* (which Thai
//! lemmas share a Princeton WordNet synset). Words in the same synset are
//! synonyms, so we map co-membership → [`Relation::Synonym`]. The DB does **not**
//! contain hypernym/hyponym (is-a) links, so — matching the care of the
//! 2026-09-12 seed-relation audit — we do **not** fabricate a hierarchy we don't
//! have. Only genuine synonyms are extracted. (Some inherent WordNet noise, e.g.
//! near-duplicate typo variants, is left as-is and labeled WordNet-sourced
//! rather than hand-cleaned across 13k groups.)
//!
//! The asset format is one synonym group per line: the group's real Thai lemmas,
//! tab-separated. Runtime stays deterministic and LLM-free — this is a static
//! data expansion, parsed once at `Engine` build time.

/// The embedded synonym-groups asset (compile-time). Regenerate offline with:
/// `sqlite3 -noheader -separator '\t' data/wordnet_th.db \`
/// `  "SELECT GROUP_CONCAT(li, char(9)) FROM word_synset`
/// `   WHERE li!='0' AND li!='' GROUP BY synsetid HAVING COUNT(*)>=2`
/// `   ORDER BY synsetid;" > data/wordnet_synonyms.tsv`
const SYNONYMS_TSV: &str = include_str!("../data/wordnet_synonyms.tsv");

/// A parsed store of WordNet synonym groups.
#[derive(Debug, Clone, Default)]
pub struct WordNet {
    /// Each inner vec is one synonym group (>=2 Thai lemmas that share a synset).
    groups: Vec<Vec<String>>,
}

impl WordNet {
    /// Parse the embedded default asset. Never panics — a malformed/empty asset
    /// just yields fewer (or zero) groups; the relationship layer degrades
    /// gracefully to the seed relations only.
    pub fn embedded() -> Self {
        Self::from_tsv(SYNONYMS_TSV)
    }

    /// Parse synonym groups from a TSV string (one group per line, tab-separated
    /// lemmas). Lines with fewer than 2 distinct non-empty lemmas are skipped.
    pub fn from_tsv(tsv: &str) -> Self {
        let mut groups = Vec::new();
        for line in tsv.lines() {
            let line = line.trim_end_matches(['\r', '\n']);
            if line.is_empty() {
                continue;
            }
            let mut words: Vec<String> = Vec::new();
            for w in line.split('\t') {
                let w = w.trim();
                if !w.is_empty() && w != "0" && !words.iter().any(|e| e == w) {
                    words.push(w.to_string());
                }
            }
            if words.len() >= 2 {
                groups.push(words);
            }
        }
        Self { groups }
    }

    /// Empty WordNet (no expansion).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Number of synonym groups.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// The synonym groups.
    pub fn groups(&self) -> &[Vec<String>] {
        &self.groups
    }

    /// Iterate directed synonym pairs `(a, b)` to add to the graph. For each
    /// group of size n we emit the n·(n−1) ordered pairs (both directions), so
    /// the graph walk treats synonymy as undirected. The caller labels these
    /// with [`crate::dictionary::Relation::Synonym`].
    pub fn synonym_pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.groups.iter().flat_map(|g| {
            g.iter().enumerate().flat_map(move |(i, a)| {
                g.iter().enumerate().filter_map(move |(j, b)| {
                    if i != j {
                        Some((a.as_str(), b.as_str()))
                    } else {
                        None
                    }
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_asset_parses_many_groups() {
        let wn = WordNet::embedded();
        // The 2026-09-12 extraction produced 13,664 groups; assert a healthy
        // lower bound so a truncated/missing asset is caught.
        assert!(
            wn.group_count() > 10_000,
            "expected >10k synonym groups, got {}",
            wn.group_count()
        );
    }

    #[test]
    fn known_synonym_group_present() {
        let wn = WordNet::embedded();
        // สุนัข and หมา share Princeton synset 02084071-n — must be a pair.
        let has = wn.synonym_pairs().any(|(a, b)| a == "สุนัข" && b == "หมา");
        assert!(has, "expected สุนัข↔หมา synonym pair from WordNet");
    }

    #[test]
    fn pairs_are_bidirectional_and_skip_self() {
        let wn = WordNet::from_tsv("ก\tข\tค\n");
        let pairs: Vec<(String, String)> = wn
            .synonym_pairs()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        // 3 words -> 6 ordered pairs, no self-pairs.
        assert_eq!(pairs.len(), 6);
        assert!(pairs.contains(&("ก".into(), "ข".into())));
        assert!(pairs.contains(&("ข".into(), "ก".into())));
        assert!(!pairs.iter().any(|(a, b)| a == b));
    }

    #[test]
    fn short_or_empty_lines_skipped() {
        let wn = WordNet::from_tsv("เดี่ยว\n\nก\tข\n0\t0\n");
        // Only "ก\tข" is a valid >=2 group ("0" filtered, single-word skipped).
        assert_eq!(wn.group_count(), 1);
    }
}
