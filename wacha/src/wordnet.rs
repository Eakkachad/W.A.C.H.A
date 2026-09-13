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
/// Synset-id-keyed asset: `<synsetid>\t<word>\t<word>…` per line. Keeping the
/// synset id lets the graph route relations through a per-synset **Sense node**
/// (Task 4), which makes cross-synset 2-hop leakage structurally impossible.
const SYNSETS_TSV: &str = include_str!("../data/wordnet_synsets.tsv");

/// A parsed store of WordNet synonym groups.
#[derive(Debug, Clone, Default)]
pub struct WordNet {
    /// Each inner vec is one synonym group (>=2 Thai lemmas that share a synset).
    groups: Vec<Vec<String>>,
    /// Same groups, but each tagged with its Princeton WordNet synset id.
    /// `(synset_id, members)`. Members are pruned identically to `groups`.
    synsets: Vec<(String, Vec<String>)>,
}

impl WordNet {
    /// Parse the embedded default assets (both the legacy flat groups and the
    /// synset-id-keyed synsets). Never panics.
    pub fn embedded() -> Self {
        let mut wn = Self::from_tsv(SYNONYMS_TSV);
        wn.synsets = parse_synsets(SYNSETS_TSV);
        wn
    }

    /// Parse synonym groups from a TSV string (one group per line, tab-separated
    /// lemmas). Lines with fewer than 2 distinct non-empty lemmas are skipped.
    ///
    /// Audited-wrong seed relations (see `SUPPRESSED_SEED_MEMBERS` /
    /// `data/seed_wordnet_audit_2026-09-13.md`) are pruned **at group level**:
    /// if a synset group contains a seed word, the audited-wrong co-members are
    /// dropped from that group entirely. This removes not just the direct wrong
    /// pair but also the 2-hop bridges through it (e.g. นักเรียน↔นร.↔นิสิต — a
    /// direct-pair suppression alone wouldn't stop นิสิต reaching นักเรียน via
    /// นร., which shares the same conflated student synset).
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
            prune_suppressed(&mut words);
            if words.len() >= 2 {
                groups.push(words);
            }
        }
        Self { groups, synsets: Vec::new() }
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

    /// Synset-id-keyed groups `(synset_id, members)` — the Task 4 graph builds a
    /// Sense node per entry here. Members are pruned identically to `groups`.
    pub fn synsets(&self) -> &[(String, Vec<String>)] {
        &self.synsets
    }

    /// Iterate directed synonym pairs `(a, b)` to add to the graph. For each
    /// group of size n we emit the n·(n−1) ordered pairs (both directions), so
    /// the graph walk treats synonymy as undirected. The caller labels these
    /// with [`crate::dictionary::Relation::Synonym`].
    ///
    /// Audited-wrong seed relations are already removed at group-construction
    /// time (see [`WordNet::from_tsv`] / [`SUPPRESSED_SEED_MEMBERS`]).
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

/// Seed word → co-members that a manual audit judged NOT true synonyms of it
/// (`wacha/data/seed_wordnet_audit_2026-09-13.md`). When a WordNet synset group
/// contains the seed word, these co-members are pruned from that group — which
/// removes the direct wrong pair AND any multi-hop bridge through the shared
/// synset. Source of truth for *why* each is here is the audit file.
pub const SUPPRESSED_SEED_MEMBERS: &[(&str, &[&str])] = &[
    // นักเรียน (secondary) conflated with tertiary-student terms by Thai WordNet;
    // ORST keeps นักเรียน vs นักศึกษา/นิสิต distinct.
    (
        "นักเรียน",
        &["นศ.", "นักศึกษา", "นิสิต", "นิสิตนักศึกษา", "นักวิชาการ"],
    ),
    // One-off cross-lingual sense-mapping artifacts.
    ("ครู", &["ผู้สาธิตวิธีการ"]),
    ("ใหญ่", &["หลัก"]),
];

/// If `words` (one synset group) contains a seed listed in
/// [`SUPPRESSED_SEED_MEMBERS`], remove that seed's audited-wrong co-members from
/// the group in place. (The seed word itself is never removed.)
fn prune_suppressed(words: &mut Vec<String>) {
    for &(seed, bad) in SUPPRESSED_SEED_MEMBERS {
        if words.iter().any(|w| w == seed) {
            words.retain(|w| w == seed || !bad.contains(&w.as_str()));
        }
    }
    prune_ambiguous_members(words);
}

/// Parse the synset-id-keyed TSV (`<synsetid>\t<word>…`) with the same member
/// pruning as `from_tsv`. Skips groups that fall below 2 members after pruning.
fn parse_synsets(tsv: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for line in tsv.lines() {
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let Some(id) = fields.next() else { continue };
        let mut words: Vec<String> = Vec::new();
        for w in fields {
            let w = w.trim();
            if !w.is_empty() && w != "0" && !words.iter().any(|e| e == w) {
                words.push(w.to_string());
            }
        }
        prune_suppressed(&mut words);
        if words.len() >= 2 {
            out.push((id.to_string(), words));
        }
    }
    out
}

/// One-off sense-disambiguation for a genuinely ambiguous Thai abbreviation that
/// bridged two unrelated synsets (see `data/seed_wordnet_audit_2026-09-13.md`
/// addendum, Task 13). `อ.` abbreviates BOTH `อาจารย์`/`ครู` (teacher) AND
/// `อังคาร` (Tuesday). Thai WordNet lists `อ.` in two unrelated synsets; our
/// loader has no word-sense layer, so `อ.` became one graph node bridging
/// `ครู` → `วันอังคาร`/`อังคาร` via PPR.
///
/// Fix: drop `อ.` from the **calendar-sense** group ONLY (the one that also
/// contains a calendar marker like `วันอังคาร`), keeping it fully intact in the
/// teacher-sense group (the `ครู`↔`อ.` relation is audited KEEP, untouched).
/// This is a single targeted cut, NOT a general disambiguation system.
///
/// Format: (member_to_remove, &[markers]) — remove `member` from a group iff the
/// group also contains any of `markers`.
const SUPPRESSED_AMBIGUOUS_MEMBERS: &[(&str, &[&str])] =
    &[("อ.", &["วันอังคาร", "อังคาร"])];

fn prune_ambiguous_members(words: &mut Vec<String>) {
    for &(member, markers) in SUPPRESSED_AMBIGUOUS_MEMBERS {
        let has_member = words.iter().any(|w| w == member);
        let has_marker = words.iter().any(|w| markers.contains(&w.as_str()));
        if has_member && has_marker {
            words.retain(|w| w != member);
        }
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

    #[test]
    fn audited_wrong_seed_members_are_pruned_from_seed_groups() {
        // After the 2026-09-13 seed audit, no synset group that contains a seed
        // word may still contain that seed's audited-wrong co-members — this
        // removes both the direct wrong pair and any bridge through the group.
        let wn = WordNet::embedded();
        for &(seed, bad) in SUPPRESSED_SEED_MEMBERS {
            for g in wn.groups() {
                if g.iter().any(|w| w == seed) {
                    for &b in bad {
                        assert!(
                            !g.iter().any(|w| w == b),
                            "audited-wrong member {b} still in a group with seed {seed}"
                        );
                    }
                }
            }
        }
        // Sanity: a KEEP co-member is still grouped with its seed.
        let kru_keeps_ajarn = wn
            .groups()
            .iter()
            .any(|g| g.iter().any(|w| w == "ครู") && g.iter().any(|w| w == "อาจารย์"));
        assert!(kru_keeps_ajarn, "kept pair ครู/อาจารย์ should survive pruning");
    }

    #[test]
    fn prune_keeps_the_seed_itself() {
        // Pruning must never drop the seed word, only its bad co-members.
        let mut g: Vec<String> = ["นักเรียน", "นักศึกษา", "ผู้เรียน"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        prune_suppressed(&mut g);
        assert!(g.contains(&"นักเรียน".to_string()));   // seed kept
        assert!(g.contains(&"ผู้เรียน".to_string()));   // good member kept
        assert!(!g.contains(&"นักศึกษา".to_string()));  // bad member pruned
    }

    #[test]
    fn ambiguous_abbrev_pruned_from_calendar_group_only() {
        // Task 13: อ. is an ambiguous abbrev (อาจารย์/ครู AND อังคาร/Tuesday).
        // Calendar-sense group: อ. must be pruned; วันอังคาร/อังคาร stay.
        let mut cal: Vec<String> = ["วันอังคาร", "อ.", "อังคาร"]
            .iter().map(|s| s.to_string()).collect();
        prune_suppressed(&mut cal);
        assert!(!cal.contains(&"อ.".to_string()), "อ. must be cut from calendar group");
        assert!(cal.contains(&"วันอังคาร".to_string()) && cal.contains(&"อังคาร".to_string()),
            "วันอังคาร/อังคาร must remain (still a valid >=2 group)");

        // Teacher-sense group: อ. must stay (ครู↔อ. is audited KEEP, untouched).
        let mut teach: Vec<String> = ["ครู", "ครูบาอาจารย์", "ผู้สอน", "ผู้ให้ความรู้", "อ.", "อาจารย์"]
            .iter().map(|s| s.to_string()).collect();
        prune_suppressed(&mut teach);
        assert!(teach.contains(&"อ.".to_string()), "อ. must remain in the teacher group");
    }

    #[test]
    fn kru_has_no_calendar_bridge_via_embedded_data() {
        // End-to-end on the shipped asset: no group containing ครู's abbrev อ.
        // also contains a calendar marker (the bridge is severed at source).
        let wn = WordNet::embedded();
        for g in wn.groups() {
            let has_aor = g.iter().any(|w| w == "อ.");
            let has_cal = g.iter().any(|w| w == "วันอังคาร" || w == "อังคาร");
            assert!(!(has_aor && has_cal), "อ. still shares a group with a calendar word: {g:?}");
        }
    }
}
