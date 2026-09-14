//! Specialized-domain terms importer (R10 Phase E).
//!
//! Reads `data/specialized_terms.tsv` (derived by `scripts/reshape_specialized.py`
//! from the three ORST specialized dictionaries — ศัพท์จิตวิทยา / ศัพท์ปรัชญา /
//! ศัพท์แพทยศาสตร์) into the SAME `Entry` shape [`CoinedWordImporter`] produces:
//! headword = the Thai coined term, one [`Sense`] per (Thai, discipline) carrying
//! `Source::CoinedWord` / `OrstEducational` / `Confirmed`.
//!
//! HONEST framing (per the CoinedWord doc): this is authoritative English↔Thai
//! **term equivalence** per discipline, NOT full encyclopedic definitions.
//!
//! Cross-discipline link: the organizer's dedicated column
//! ("ศัพท์อื่นที่เป็นคำเดียวกันในต่างสาขา") is EMPTY in all three real files
//! (measured: 0/7,241), so it can't be used. Instead we derive the genuine
//! cross-discipline equivalence from the data itself: when the SAME English
//! headword is coined in MORE THAN ONE discipline, those Thai terms are linked
//! with [`Relation::RelatedTo`] (labelled cross-discipline), which is the real,
//! checkable "one English word → different Thai term per field" claim.

use crate::dictionary::{
    Entry, License, Pos, Provenance, Relation, RelationConfidence, Sense, Source, Subject,
};
use crate::import::{ImportResult, Importer};
use std::collections::BTreeMap;
use std::path::Path;

pub struct SpecializedImporter;

impl Importer for SpecializedImporter {
    fn name(&self) -> &'static str {
        "specialized"
    }
    fn source(&self) -> Source {
        Source::CoinedWord
    }

    /// `path` is the `specialized_terms.tsv` file (english\tthai\tdiscipline\tcross).
    fn load(&self, path: &Path) -> ImportResult<Vec<Entry>> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::parse(&text))
    }
}

impl SpecializedImporter {
    pub fn parse(text: &str) -> Vec<Entry> {
        // (english_lower -> [(thai, discipline)]) to find cross-discipline pairs,
        // and (thai -> Entry) to accumulate senses.
        let mut by_english: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        let mut rows: Vec<(String, String, String)> = Vec::new(); // (english, thai, discipline)
        for line in text.lines() {
            let mut c = line.split('\t');
            let (Some(en), Some(thai), Some(disc)) = (c.next(), c.next(), c.next()) else {
                continue;
            };
            let en = en.trim();
            let thai = thai.trim();
            let disc = disc.trim();
            if en.is_empty() || thai.is_empty() || disc.is_empty() {
                continue;
            }
            by_english
                .entry(en.to_lowercase())
                .or_default()
                .push((thai.to_string(), disc.to_string()));
            rows.push((en.to_string(), thai.to_string(), disc.to_string()));
        }

        let mut entries: BTreeMap<String, Entry> = BTreeMap::new();
        for (english, thai, discipline) in &rows {
            let subject = match Subject::from_marker(discipline) {
                Subject::Other(_) => Subject::Other(discipline.clone()),
                known => known,
            };
            let definition = format!("{} (ศัพท์บัญญัติ · {})", english, discipline);
            let sense = Sense {
                pos: Some(Pos::Nam),
                subject: Some(subject),
                register: None,
                definition,
                examples: Vec::new(),
                classifiers: Vec::new(),
                provenance: Provenance {
                    source: Source::CoinedWord,
                    license: License::OrstEducational,
                    confidence: RelationConfidence::Confirmed,
                },
            };
            let entry = entries
                .entry(thai.clone())
                .or_insert_with(|| Entry::headword_only(thai));
            entry.senses.push(sense);

            // Cross-discipline link: other Thai coinages of the SAME English term
            // in a DIFFERENT discipline (the "same word, different field" claim).
            if let Some(sibs) = by_english.get(&english.to_lowercase()) {
                for (other_thai, other_disc) in sibs {
                    if other_thai != thai && other_disc != discipline {
                        let rel = (Relation::RelatedTo, other_thai.clone());
                        if !entry.relations.contains(&rel) {
                            entry.relations.push(rel);
                        }
                    }
                }
            }
        }
        entries.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_entries_and_cross_discipline_links() {
        // Same English "abnormal" coined differently in two disciplines.
        let tsv = "abnormal\tอปรกติ\tจิตวิทยา\t\n\
                   abnormal\tผิดปรกติ\tแพทยศาสตร์\t\n\
                   ability\tความสามารถ\tจิตวิทยา\t\n";
        let entries = SpecializedImporter::parse(tsv);
        // one entry per distinct Thai term
        assert_eq!(entries.len(), 3);
        let abn = entries.iter().find(|e| e.headword == "อปรกติ").expect("อปรกติ");
        // subject tag present + CoinedWord source
        assert_eq!(abn.senses[0].provenance.source, Source::CoinedWord);
        assert!(abn.senses[0].definition.contains("จิตวิทยา"));
        // cross-discipline link to the แพทยศาสตร์ coinage of the same English word
        assert!(abn
            .relations
            .iter()
            .any(|(r, t)| *r == Relation::RelatedTo && t == "ผิดปรกติ"));
    }
}
