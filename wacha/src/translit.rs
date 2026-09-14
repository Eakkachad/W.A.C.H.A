//! Transliteration assistant (R10 Phase T).
//!
//! A flat, bidirectional lookup over the ORST official transliteration list
//! (ประมวลคำทับศัพท์ภาษาอังกฤษ พ.ศ. 2563 (ราชบัณฑิตยสภา), 2,256 pairs). This is
//! deliberately NOT part of the `Entry`/`Sense`/graph model — it's a standalone
//! dictionary of English↔Thai transliteration pairs, the cheapest new surface and
//! a concrete "we use ORST's own official rules" demo beat.
//!
//! Both directions are supported: an English loanword → its official Thai
//! transliteration, and a Thai transliteration → the English source term.

use std::collections::BTreeMap;

/// The reference line shown as provenance on every transliteration result.
pub const TRANSLIT_SOURCE: &str = "ประมวลคำทับศัพท์ภาษาอังกฤษ พ.ศ. 2563 (ราชบัณฑิตยสภา)";

/// One transliteration result (a matched pair with any ORST note).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslitHit {
    pub english: String,
    pub thai: String,
    /// ORST note (e.g. "ยังไม่มีในระบบ", or an alternate spelling); may be empty.
    pub note: String,
}

/// A bidirectional transliteration index.
#[derive(Debug, Default, Clone)]
pub struct Translit {
    /// lowercased English → hits
    en_to_th: BTreeMap<String, Vec<TranslitHit>>,
    /// Thai transliteration → hits
    th_to_en: BTreeMap<String, Vec<TranslitHit>>,
}

impl Translit {
    /// Parse the `english \t thai \t note` TSV projection (see
    /// `scripts/reshape_translit.py`).
    pub fn from_tsv(text: &str) -> Self {
        let mut t = Translit::default();
        for line in text.lines() {
            let mut cols = line.split('\t');
            let (Some(en), Some(th)) = (cols.next(), cols.next()) else {
                continue;
            };
            let en = en.trim();
            let th = th.trim();
            if en.is_empty() || th.is_empty() {
                continue;
            }
            let note = cols.next().unwrap_or("").trim().to_string();
            let hit = TranslitHit { english: en.to_string(), thai: th.to_string(), note };
            t.en_to_th.entry(en.to_lowercase()).or_default().push(hit.clone());
            t.th_to_en.entry(th.to_string()).or_default().push(hit);
        }
        t
    }

    /// Number of distinct English terms indexed.
    pub fn len(&self) -> usize {
        self.en_to_th.values().map(|v| v.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.en_to_th.is_empty()
    }

    /// Look up a query in EITHER direction. If the query contains ASCII letters
    /// it's treated as English (→ Thai); otherwise as a Thai transliteration
    /// (→ English). Returns all matching pairs (a term can map to several).
    pub fn lookup(&self, query: &str) -> Vec<TranslitHit> {
        let q = query.trim();
        if q.is_empty() {
            return Vec::new();
        }
        let looks_english = q.chars().any(|c| c.is_ascii_alphabetic());
        if looks_english {
            self.en_to_th.get(&q.to_lowercase()).cloned().unwrap_or_default()
        } else {
            self.th_to_en.get(q).cloned().unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Translit {
        Translit::from_tsv(
            "computer\tคอมพิวเตอร์\t\n\
             internet\tอินเทอร์เน็ต\t\n\
             digital\tดิจิทัล\t\n\
             calculus\tแคลคูลัส\t\n\
             graphic\tกราฟิก\tยังไม่มีในระบบ\n",
        )
    }

    #[test]
    fn five_round_trips_both_directions() {
        let t = sample();
        let pairs = [
            ("computer", "คอมพิวเตอร์"),
            ("internet", "อินเทอร์เน็ต"),
            ("digital", "ดิจิทัล"),
            ("calculus", "แคลคูลัส"),
            ("graphic", "กราฟิก"),
        ];
        for (en, th) in pairs {
            // EN -> TH
            let f = t.lookup(en);
            assert!(f.iter().any(|h| h.thai == th), "{en} -> {th}");
            // TH -> EN (round-trip)
            let b = t.lookup(th);
            assert!(b.iter().any(|h| h.english == en), "{th} -> {en}");
        }
    }

    #[test]
    fn case_insensitive_english_and_note_preserved() {
        let t = sample();
        assert!(t.lookup("COMPUTER").iter().any(|h| h.thai == "คอมพิวเตอร์"));
        let g = t.lookup("graphic");
        assert_eq!(g[0].note, "ยังไม่มีในระบบ");
    }

    #[test]
    fn unknown_query_returns_empty() {
        assert!(sample().lookup("zzzznotaword").is_empty());
        assert!(sample().lookup("ไม่มีคำนี้").is_empty());
    }
}
