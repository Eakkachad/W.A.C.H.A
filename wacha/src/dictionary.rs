//! Dictionary data: the word list that drives segmentation, optional word
//! frequencies (for ranking), and structured dictionary entries carrying the
//! explicit relations from which we derive knowledge-graph triples.
//!
//! Data provenance (resolved Day-0, `AGENT_HANDOFF.md` §6):
//! - Word list: PyThaiNLP `words_th.txt`, 62,107 entries, **CC0-1.0** (public
//!   domain; derived from NECTEC LEXiTRON). Free to use with no attribution.
//! - Frequencies: PyThaiNLP `tnc_freq.txt` (Thai National Corpus), **CC0-1.0**.
//!
//! At the real event, if ORST hands out an official RID dataset it supersedes
//! this — the `Entry`/relation model below is the integration point.

use std::collections::HashMap;

/// A dictionary relation kind. Kept small and explicit — we only extract
/// relations the source data actually carries, never inferred/hallucinated ones
/// (per the guardrails). Each maps to a human-readable Thai relation label used
/// in the knowledge graph and shown to the user in explanations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relation {
    /// synonym — คำพ้องความหมาย
    Synonym,
    /// antonym — คำตรงข้าม
    Antonym,
    /// hypernym / "is a kind of" — เป็นชนิดของ
    IsA,
    /// "see also" / cross reference — ดูเพิ่มที่
    SeeAlso,
    /// category / domain — อยู่ในหมวด
    Category,
}

impl Relation {
    /// The Thai label shown in explanations and stored as the graph relation.
    pub fn thai_label(self) -> &'static str {
        match self {
            Relation::Synonym => "มีความหมายเหมือนกับ",
            Relation::Antonym => "ตรงข้ามกับ",
            Relation::IsA => "เป็นชนิดของ",
            Relation::SeeAlso => "ดูเพิ่มที่",
            Relation::Category => "อยู่ในหมวด",
        }
    }
}

/// One dictionary entry: a headword, its definition, part of speech, and the
/// explicit relations it carries to other words.
#[derive(Debug, Clone)]
pub struct Entry {
    pub word: String,
    pub pos: String,
    pub definition: String,
    /// (relation, target-word) pairs, e.g. (Synonym, "สุนัข").
    pub relations: Vec<(Relation, String)>,
}

/// The dictionary: a headword-indexed store of entries plus a frequency table.
#[derive(Default)]
pub struct Dictionary {
    entries: HashMap<String, Entry>,
    /// Insertion order for stable iteration / display.
    order: Vec<String>,
    freq: HashMap<String, u64>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace an entry.
    pub fn insert(&mut self, entry: Entry) {
        if !self.entries.contains_key(&entry.word) {
            self.order.push(entry.word.clone());
        }
        self.entries.insert(entry.word.clone(), entry);
    }

    /// Look up an entry by exact headword.
    pub fn get(&self, word: &str) -> Option<&Entry> {
        self.entries.get(word)
    }

    /// All headwords, in insertion order.
    pub fn words(&self) -> impl Iterator<Item = &str> {
        self.order.iter().map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Load a frequency table (word \t count per line), used to rank results.
    pub fn load_frequencies(&mut self, text: &str) {
        for line in text.lines() {
            let mut parts = line.splitn(2, '\t');
            if let (Some(w), Some(c)) = (parts.next(), parts.next()) {
                if let Ok(count) = c.trim().parse::<u64>() {
                    self.freq.insert(w.trim().to_string(), count);
                }
            }
        }
    }

    /// Corpus frequency of a word (0 if unknown). Higher = more common.
    pub fn frequency(&self, word: &str) -> u64 {
        self.freq.get(word).copied().unwrap_or(0)
    }

    pub fn all_entries(&self) -> impl Iterator<Item = &Entry> {
        self.order.iter().filter_map(move |w| self.entries.get(w))
    }
}

/// A curated seed of real Thai dictionary entries with genuine relations.
///
/// These stand in for RID-derived structured entries until a real dataset is
/// wired in. They are *real* Thai words with *real* semantic relations (not
/// invented facts) — enough to demonstrate the explainable-relationship graph
/// end-to-end. The relations here are exactly the kind an RID entry carries
/// explicitly (synonym / antonym / cross-reference / category), so the
/// extraction path in `relations.rs` transfers unchanged to real data.
pub fn seed_entries() -> Vec<Entry> {
    use Relation::*;
    let mk = |word: &str, pos: &str, def: &str, rels: &[(Relation, &str)]| Entry {
        word: word.to_string(),
        pos: pos.to_string(),
        definition: def.to_string(),
        relations: rels.iter().map(|(r, w)| (*r, w.to_string())).collect(),
    };
    vec![
        mk("แมว", "น.", "สัตว์เลี้ยงลูกด้วยนมชนิดหนึ่ง เลี้ยงไว้ในบ้าน จับหนูเป็นอาหาร",
            &[(IsA, "สัตว์"), (Category, "สัตว์เลี้ยง"), (SeeAlso, "เสือ")]),
        mk("สุนัข", "น.", "สัตว์เลี้ยงลูกด้วยนมชนิดหนึ่ง เลี้ยงไว้เฝ้าบ้าน; หมา",
            &[(IsA, "สัตว์"), (Synonym, "หมา"), (Category, "สัตว์เลี้ยง")]),
        mk("หมา", "น.", "สุนัข",
            &[(Synonym, "สุนัข"), (IsA, "สัตว์")]),
        mk("เสือ", "น.", "สัตว์กินเนื้อขนาดใหญ่ในวงศ์แมว",
            &[(IsA, "สัตว์"), (SeeAlso, "แมว")]),
        mk("สัตว์", "น.", "สิ่งมีชีวิตที่เคลื่อนไหวได้และกินสิ่งอื่นเป็นอาหาร",
            &[(Category, "สิ่งมีชีวิต")]),
        mk("ใหญ่", "ว.", "มีขนาดโตกว่าปรกติ",
            &[(Antonym, "เล็ก")]),
        mk("เล็ก", "ว.", "มีขนาดย่อมกว่าปรกติ",
            &[(Antonym, "ใหญ่")]),
        mk("สุข", "น.", "ความสบายกายสบายใจ",
            &[(Antonym, "ทุกข์"), (Synonym, "ความสุข")]),
        mk("ทุกข์", "น.", "ความไม่สบายกายไม่สบายใจ",
            &[(Antonym, "สุข")]),
        mk("ครู", "น.", "ผู้สั่งสอนศิษย์; ผู้ถ่ายทอดความรู้",
            &[(Synonym, "อาจารย์"), (Category, "การศึกษา"), (SeeAlso, "โรงเรียน")]),
        mk("อาจารย์", "น.", "ผู้สั่งสอนวิชาความรู้ในระดับสูง",
            &[(Synonym, "ครู"), (Category, "การศึกษา")]),
        mk("นักเรียน", "น.", "ผู้เรียนในโรงเรียน",
            &[(Category, "การศึกษา"), (SeeAlso, "โรงเรียน"), (Antonym, "ครู")]),
        mk("โรงเรียน", "น.", "สถานที่สำหรับสอนและเรียนหนังสือ",
            &[(Category, "การศึกษา"), (SeeAlso, "นักเรียน")]),
        mk("หนังสือ", "น.", "เอกสารที่เขียนหรือพิมพ์เป็นเล่มสำหรับอ่าน",
            &[(Category, "การศึกษา"), (SeeAlso, "พจนานุกรม")]),
        mk("พจนานุกรม", "น.", "หนังสือรวบรวมคำและความหมายเรียงตามลำดับตัวอักษร",
            &[(IsA, "หนังสือ"), (Category, "การศึกษา"), (SeeAlso, "คำ")]),
        mk("คำ", "น.", "เสียงพูดหรือตัวหนังสือที่มีความหมาย",
            &[(SeeAlso, "ความหมาย"), (SeeAlso, "ภาษา")]),
        mk("ความหมาย", "น.", "สิ่งที่คำหรือข้อความนั้นสื่อให้เข้าใจ",
            &[(SeeAlso, "คำ")]),
        mk("ภาษา", "น.", "เสียงหรือตัวหนังสือที่ใช้สื่อความหมายกัน",
            &[(SeeAlso, "คำ"), (Category, "การศึกษา")]),
        mk("อ่าน", "ก.", "ดูตัวหนังสือแล้วเข้าใจความหมาย; ออกเสียงตามตัวหนังสือ",
            &[(SeeAlso, "หนังสือ"), (Antonym, "เขียน")]),
        mk("เขียน", "ก.", "ทำให้เป็นตัวหนังสือหรือรูปด้วยเครื่องมือ",
            &[(Antonym, "อ่าน"), (SeeAlso, "หนังสือ")]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut d = Dictionary::new();
        d.insert(Entry {
            word: "แมว".into(),
            pos: "น.".into(),
            definition: "สัตว์".into(),
            relations: vec![],
        });
        assert_eq!(d.len(), 1);
        assert_eq!(d.get("แมว").unwrap().pos, "น.");
        assert!(d.get("หมา").is_none());
    }

    #[test]
    fn frequencies_parse() {
        let mut d = Dictionary::new();
        d.load_frequencies("ที่\t818364\nการ\t592036\n");
        assert_eq!(d.frequency("ที่"), 818364);
        assert_eq!(d.frequency("ไม่มี"), 0);
    }

    #[test]
    fn seed_is_nonempty_and_relations_resolve() {
        let entries = seed_entries();
        assert!(entries.len() >= 15);
        // Every relation target should ideally be a known headword (some may be
        // category nodes that aren't headwords — that's allowed).
        let words: std::collections::HashSet<_> =
            entries.iter().map(|e| e.word.as_str()).collect();
        let mut resolved = 0;
        let mut total = 0;
        for e in &entries {
            for (_r, tgt) in &e.relations {
                total += 1;
                if words.contains(tgt.as_str()) {
                    resolved += 1;
                }
            }
        }
        // Most relations should point at real headwords (sanity, not strict).
        assert!(resolved * 2 >= total, "too many dangling relation targets");
    }
}
