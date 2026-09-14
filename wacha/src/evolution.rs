//! Word Evolution Timeline (R10 Phase W), อักษร ก only.
//!
//! A standalone, keyed comparison of one headword's definition across the three
//! RID editions the organizer shipped — 2542 → 2554 → 2569 — showing ORST's
//! continuous ชำระพจนานุกรม mission. Loaded from `data/evolution_ko.tsv`
//! (`headword \t edition \t definition`, built by `scripts/reshape_evolution.py`).
//!
//! Like [`crate::translit`], this does NOT touch the `Entry`/graph model.
//!
//! **Integrity:** the 2569 edition is an unfinished DRAFT. Its results MUST carry
//! [`DRAFT_2569_LABEL`] verbatim — presenting draft data as final would be exactly
//! the honesty lapse this project has avoided since R5.

use std::collections::BTreeMap;

/// Mandatory verbatim caveat on any 2569 (๒๕๖๙) row (ORST's own wording).
pub const DRAFT_2569_LABEL: &str = "ร่าง อยู่ระหว่างดำเนินการ ยังไม่เป็นข้อมูลทางการ";

/// The three editions, in chronological order, as they key the TSV.
pub const EDITIONS: [&str; 3] = ["๒๕๔๒", "๒๕๕๔", "๒๕๖๙"];

/// The draft edition whose rows carry [`DRAFT_2569_LABEL`].
pub const DRAFT_EDITION: &str = "๒๕๖๙";

/// One edition's definition of a headword.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvolutionEntry {
    pub edition: String,
    pub definition: String,
    /// True for the 2569 draft — the UI/CLI must show `DRAFT_2569_LABEL`.
    pub is_draft: bool,
}

/// Headword → its definitions across editions (chronological).
#[derive(Debug, Default, Clone)]
pub struct Evolution {
    by_headword: BTreeMap<String, Vec<EvolutionEntry>>,
}

impl Evolution {
    pub fn from_tsv(text: &str) -> Self {
        let mut e = Evolution::default();
        for line in text.lines() {
            let mut c = line.split('\t');
            let (Some(hw), Some(ed), Some(def)) = (c.next(), c.next(), c.next()) else {
                continue;
            };
            let hw = hw.trim();
            let ed = ed.trim();
            let def = def.trim();
            if hw.is_empty() || ed.is_empty() || def.is_empty() {
                continue;
            }
            e.by_headword.entry(hw.to_string()).or_default().push(EvolutionEntry {
                edition: ed.to_string(),
                definition: def.to_string(),
                is_draft: ed == DRAFT_EDITION,
            });
        }
        // Keep each headword's editions in chronological order.
        for v in e.by_headword.values_mut() {
            v.sort_by_key(|x| EDITIONS.iter().position(|&e| e == x.edition).unwrap_or(usize::MAX));
        }
        e
    }

    /// Number of headwords with a timeline.
    pub fn len(&self) -> usize {
        self.by_headword.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_headword.is_empty()
    }

    /// The timeline for a headword (chronological), or empty if none.
    pub fn timeline(&self, headword: &str) -> Vec<EvolutionEntry> {
        self.by_headword.get(headword.trim()).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Evolution {
        // Deliberately out of order to check chronological sorting.
        Evolution::from_tsv(
            "ก\t๒๕๖๙\tนิยามฉบับร่าง 2569\n\
             ก\t๒๕๔๒\tนิยามฉบับ 2542\n\
             ก\t๒๕๕๔\tนิยามฉบับ 2554\n",
        )
    }

    #[test]
    fn timeline_is_chronological_and_flags_draft() {
        let e = sample();
        let t = e.timeline("ก");
        assert_eq!(t.len(), 3);
        assert_eq!(t[0].edition, "๒๕๔๒");
        assert_eq!(t[1].edition, "๒๕๕๔");
        assert_eq!(t[2].edition, "๒๕๖๙");
        // only the 2569 row is flagged as draft
        assert!(!t[0].is_draft && !t[1].is_draft);
        assert!(t[2].is_draft);
    }

    #[test]
    fn draft_label_is_the_verbatim_orst_caveat() {
        assert_eq!(DRAFT_2569_LABEL, "ร่าง อยู่ระหว่างดำเนินการ ยังไม่เป็นข้อมูลทางการ");
    }

    #[test]
    fn unknown_headword_empty() {
        assert!(sample().timeline("ไม่มีคำนี้").is_empty());
    }
}
