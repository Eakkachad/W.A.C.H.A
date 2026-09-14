//! Dictionary data model — reshaped (Round 5, Task 1) to match the structure of
//! พจนานุกรม ฉบับราชบัณฑิตยสถาน พ.ศ. ๒๕๕๔ (RID 2554), so the real competition
//! dataset drops into the same `Entry`/`Sense` shape without a rewrite.
//!
//! Key change from Rounds 1–4: an `Entry` now holds a headword + scalar metadata
//! and a list of **`Sense`s**, each carrying its own part-of-speech, subject
//! field, register, definition, examples, classifiers, and — crucially —
//! **per-sense `Provenance`** (source + licence + confidence). This is what lets
//! us merge LEXiTRON + Kaikki + ศัพท์บัญญัติ + RID into one entry while keeping an
//! honest, field-level record of where each fact came from and under what
//! licence.
//!
//! Data provenance of the current practice data (`AGENT_HANDOFF.md` §6):
//! - Word list: PyThaiNLP `words_th.txt`, **CC0-1.0** (from NECTEC LEXiTRON).
//! - Frequencies: PyThaiNLP `tnc_freq.txt`, **CC0-1.0**.
//! The real RID data arrives at the event; see `COMPETITION_DAY.md` (Task 8).

use std::collections::HashMap;

// ── Controlled vocabularies (RID §1.1) ──────────────────────────────────────

/// Part of speech — the 8 RID word classes (ชนิดของคำ). Closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pos {
    Kri,        // ก.  กริยา (verb)
    Nam,        // น.  คำนาม (noun)
    Nibat,      // นิ. นิบาต (particle)
    Buraphabot, // บ.  บุรพบท (preposition)
    Uthan,      // อ.  อุทาน (interjection)
    Santhan,    // สัน สันธาน (conjunction)
    Sapphanam,  // ส.  สรรพนาม (pronoun)
    Wiset,      // ว.  วิเศษณ์ (adjective/adverb)
}

impl Pos {
    /// The RID marker (as printed in the dictionary, e.g. `น.`).
    pub fn marker(self) -> &'static str {
        match self {
            Pos::Kri => "ก.",
            Pos::Nam => "น.",
            Pos::Nibat => "นิ.",
            Pos::Buraphabot => "บ.",
            Pos::Uthan => "อ.",
            Pos::Santhan => "สัน",
            Pos::Sapphanam => "ส.",
            Pos::Wiset => "ว.",
        }
    }

    /// Full Thai name (for the UI badge, e.g. คำนาม).
    pub fn thai_name(self) -> &'static str {
        match self {
            Pos::Kri => "คำกริยา",
            Pos::Nam => "คำนาม",
            Pos::Nibat => "นิบาต",
            Pos::Buraphabot => "บุรพบท",
            Pos::Uthan => "อุทาน",
            Pos::Santhan => "สันธาน",
            Pos::Sapphanam => "สรรพนาม",
            Pos::Wiset => "วิเศษณ์",
        }
    }

    /// Parse a RID marker back into a `Pos`. Accepts the canonical markers above.
    pub fn from_marker(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "ก." => Pos::Kri,
            "น." => Pos::Nam,
            "นิ." => Pos::Nibat,
            "บ." => Pos::Buraphabot,
            "อ." => Pos::Uthan,
            "สัน" => Pos::Santhan,
            "ส." => Pos::Sapphanam,
            "ว." => Pos::Wiset,
            _ => return None,
        })
    }

    /// All 8 values (for tests / iteration).
    pub fn all() -> &'static [Pos] {
        &[
            Pos::Kri, Pos::Nam, Pos::Nibat, Pos::Buraphabot,
            Pos::Uthan, Pos::Santhan, Pos::Sapphanam, Pos::Wiset,
        ]
    }
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.marker())
    }
}

/// Subject field — the 32 RID สาขาวิชา. Closed set, but with an `Other` escape
/// hatch for sources (Kaikki topics, ศัพท์บัญญัติ disciplines) that don't map
/// cleanly onto an RID field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Subject {
    Kot, KanThut, KanMueang, KanSueksa, Kaset, Khanit, Khom, Khemi, Chari, Chiwa,
    Dara, Thorani, Banchi, Pratya, Phrueksa, Phaet, Fisik, Faifa, Phumi, Manut,
    MaeLek, Rekha, Witthaya, Wan, Wai, Satsana, Settha, Sathiti, Sari, Sangkhom,
    Sat, Saeng, Hora, Utu,
    /// A subject that isn't one of the 32 RID fields (e.g. a Kaikki topic).
    Other(String),
}

impl Subject {
    /// The RID short tag as printed in parentheses, e.g. `(ไว)`.
    pub fn tag(&self) -> &str {
        match self {
            Subject::Kot => "กฎ",
            Subject::KanThut => "การทูต",
            Subject::KanMueang => "การเมือง",
            Subject::KanSueksa => "การศึกษา",
            Subject::Kaset => "เกษตร",
            Subject::Khanit => "คณิต",
            Subject::Khom => "คอม",
            Subject::Khemi => "เคมี",
            Subject::Chari => "จริย",
            Subject::Chiwa => "ชีว",
            Subject::Dara => "ดารา",
            Subject::Thorani => "ธรณี",
            Subject::Banchi => "บัญชี",
            Subject::Pratya => "ปรัชญา",
            Subject::Phrueksa => "พฤกษ",
            Subject::Phaet => "แพทย์",
            Subject::Fisik => "ฟิสิกส์",
            Subject::Faifa => "ไฟฟ้า",
            Subject::Phumi => "ภูมิ",
            Subject::Manut => "มานุษย",
            Subject::MaeLek => "แม่เหล็ก",
            Subject::Rekha => "เรขา",
            Subject::Witthaya => "วิทยา",
            Subject::Wan => "วรรณ",
            Subject::Wai => "ไว",
            Subject::Satsana => "ศาสน",
            Subject::Settha => "เศรษฐ",
            Subject::Sathiti => "สถิติ",
            Subject::Sari => "สรีร",
            Subject::Sangkhom => "สังคม",
            Subject::Sat => "สัตว",
            Subject::Saeng => "แสง",
            Subject::Hora => "โหร",
            Subject::Utu => "อุตุ",
            Subject::Other(s) => s.as_str(),
        }
    }

    /// Parse an RID subject tag. Returns `Other` for anything not in the 32.
    pub fn from_marker(s: &str) -> Self {
        match s.trim() {
            "กฎ" => Subject::Kot,
            "การทูต" => Subject::KanThut,
            "การเมือง" => Subject::KanMueang,
            "การศึกษา" => Subject::KanSueksa,
            "เกษตร" => Subject::Kaset,
            "คณิต" => Subject::Khanit,
            "คอม" => Subject::Khom,
            "เคมี" => Subject::Khemi,
            "จริย" => Subject::Chari,
            "ชีว" => Subject::Chiwa,
            "ดารา" => Subject::Dara,
            "ธรณี" => Subject::Thorani,
            "บัญชี" => Subject::Banchi,
            "ปรัชญา" => Subject::Pratya,
            "พฤกษ" => Subject::Phrueksa,
            "แพทย์" => Subject::Phaet,
            "ฟิสิกส์" => Subject::Fisik,
            "ไฟฟ้า" => Subject::Faifa,
            "ภูมิ" => Subject::Phumi,
            "มานุษย" => Subject::Manut,
            "แม่เหล็ก" => Subject::MaeLek,
            "เรขา" => Subject::Rekha,
            "วิทยา" => Subject::Witthaya,
            "วรรณ" => Subject::Wan,
            "ไว" => Subject::Wai,
            "ศาสน" => Subject::Satsana,
            "เศรษฐ" => Subject::Settha,
            "สถิติ" => Subject::Sathiti,
            "สรีร" => Subject::Sari,
            "สังคม" => Subject::Sangkhom,
            "สัตว" => Subject::Sat,
            "แสง" => Subject::Saeng,
            "โหร" => Subject::Hora,
            "อุตุ" => Subject::Utu,
            other => Subject::Other(other.to_string()),
        }
    }

    /// The 34 named RID fields (32 in the plan + the list actually enumerated on
    /// the ORST form; `Other` excluded).
    pub fn all_named() -> Vec<Subject> {
        vec![
            Subject::Kot, Subject::KanThut, Subject::KanMueang, Subject::KanSueksa,
            Subject::Kaset, Subject::Khanit, Subject::Khom, Subject::Khemi,
            Subject::Chari, Subject::Chiwa, Subject::Dara, Subject::Thorani,
            Subject::Banchi, Subject::Pratya, Subject::Phrueksa, Subject::Phaet,
            Subject::Fisik, Subject::Faifa, Subject::Phumi, Subject::Manut,
            Subject::MaeLek, Subject::Rekha, Subject::Witthaya, Subject::Wan,
            Subject::Wai, Subject::Satsana, Subject::Settha, Subject::Sathiti,
            Subject::Sari, Subject::Sangkhom, Subject::Sat, Subject::Saeng,
            Subject::Hora, Subject::Utu,
        ]
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.tag())
    }
}

/// Register / ทะเบียนคำ — the 5 RID usage labels. Closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    Baep,  // แบบ  literary
    Bo,    // โบ   archaic
    Pak,   // ปาก  colloquial
    Racha, // ราชา royal
    Loek,  // เลิก obsolete
}

impl Register {
    pub fn marker(self) -> &'static str {
        match self {
            Register::Baep => "แบบ",
            Register::Bo => "โบ",
            Register::Pak => "ปาก",
            Register::Racha => "ราชา",
            Register::Loek => "เลิก",
        }
    }

    pub fn from_marker(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "แบบ" => Register::Baep,
            "โบ" => Register::Bo,
            "ปาก" => Register::Pak,
            "ราชา" => Register::Racha,
            "เลิก" => Register::Loek,
            _ => return None,
        })
    }

    pub fn all() -> &'static [Register] {
        &[Register::Baep, Register::Bo, Register::Pak, Register::Racha, Register::Loek]
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.marker())
    }
}

// ── Provenance ──────────────────────────────────────────────────────────────

/// Which source a sense/fact came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    Lexitron,
    Kaikki,
    CoinedWord, // ศัพท์บัญญัติ
    Rid,        // real competition data
    HumanSeed,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::Lexitron => "LEXiTRON",
            Source::Kaikki => "Kaikki (Wiktionary)",
            Source::CoinedWord => "ศัพท์บัญญัติ (ORST)",
            Source::Rid => "RID ๒๕๕๔ (ORST)",
            Source::HumanSeed => "ตรวจด้วยมือ",
        }
    }
    /// Merge priority — higher wins for scalar fields (Task 2).
    pub fn priority(self) -> u8 {
        match self {
            Source::Rid => 5,
            Source::HumanSeed => 4,
            Source::CoinedWord => 3,
            Source::Kaikki => 2,
            Source::Lexitron => 1,
        }
    }
}

/// Licence of a source's data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum License {
    Cc0,
    CcBySa,
    NictPermissive,
    OrstEducational,
}

impl License {
    pub fn label(self) -> &'static str {
        match self {
            License::Cc0 => "CC0-1.0",
            License::CcBySa => "CC BY-SA",
            License::NictPermissive => "NICT (permissive)",
            License::OrstEducational => "ORST (educational, non-commercial)",
        }
    }
}

/// Structural confidence of a relation/fact (reused from the relations layer's
/// concept; kept here so `Provenance` is self-contained).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationConfidence {
    Confirmed,
    Unverified,
}

impl RelationConfidence {
    pub fn tag(self) -> &'static str {
        match self {
            RelationConfidence::Confirmed => "ยืนยัน",
            RelationConfidence::Unverified => "ยังไม่ยืนยัน",
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            RelationConfidence::Confirmed => "confirmed",
            RelationConfidence::Unverified => "unverified",
        }
    }
}

/// Per-sense provenance: where the fact came from, its licence, and confidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    pub source: Source,
    pub license: License,
    pub confidence: RelationConfidence,
}

impl Provenance {
    /// The hand-curated seed default.
    pub fn seed() -> Self {
        Provenance {
            source: Source::HumanSeed,
            license: License::Cc0,
            confidence: RelationConfidence::Confirmed,
        }
    }
}

// ── Relations (entry-level cross references; graph layer in relations.rs) ────

/// A dictionary relation kind. Only relations the source data actually carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relation {
    Synonym,   // มีความหมายเหมือนกับ
    Antonym,   // ตรงข้ามกับ
    IsA,       // เป็นชนิดของ
    SeeAlso,   // ดูเพิ่มที่
    Category,  // อยู่ในหมวด
    RelatedTo, // เกี่ยวข้องกับ
}

impl Relation {
    pub fn thai_label(self) -> &'static str {
        match self {
            Relation::Synonym => "มีความหมายเหมือนกับ",
            Relation::Antonym => "ตรงข้ามกับ",
            Relation::IsA => "เป็นชนิดของ",
            Relation::SeeAlso => "ดูเพิ่มที่",
            Relation::Category => "อยู่ในหมวด",
            Relation::RelatedTo => "เกี่ยวข้องกับ",
        }
    }
}

// ── Etymology, Sense, Entry ──────────────────────────────────────────────────

/// An etymology note, e.g. `(ป. ปิตา; ส. ปิตฤ)`. `lang` is the source-language
/// marker (ป.=บาลี, ส.=สันสกฤต, อ.=อังกฤษ, ข.=เขมร); `form` the cited form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Etymology {
    pub lang: String,
    pub form: String,
}

/// One sense of an entry (RID senses are numbered ๑ ๒ ๓ and each has its own
/// POS / subject / register).
#[derive(Debug, Clone)]
pub struct Sense {
    pub pos: Option<Pos>,
    pub subject: Option<Subject>,
    pub register: Option<Register>,
    pub definition: String,
    pub examples: Vec<String>,
    pub classifiers: Vec<String>, // ลักษณนาม
    pub provenance: Provenance,
}

impl Sense {
    /// Convenience constructor for a simple hand-authored sense.
    pub fn simple(pos: Pos, definition: &str) -> Self {
        Sense {
            pos: Some(pos),
            subject: None,
            register: None,
            definition: definition.to_string(),
            examples: Vec::new(),
            classifiers: Vec::new(),
            provenance: Provenance::seed(),
        }
    }
}

/// A dictionary entry, RID-shaped.
#[derive(Debug, Clone)]
pub struct Entry {
    pub headword: String,
    pub homograph: Option<u8>,         // แมว ๑ -> Some(1)
    pub pronunciation: Option<String>, // "[กำ, กำมะ-]"
    pub romanization: Option<String>,  // Royal-Institute, from Kaikki sounds[]
    pub senses: Vec<Sense>,
    pub etymology: Vec<Etymology>,
    pub sub_entries: Vec<String>, // ลูกคำ
    pub see_also: Vec<String>,    // ดู
    /// Entry-level lexical relations (synonym/antonym/…), carried forward from
    /// the audited seed set. Task 4 will route these through Sense graph nodes.
    pub relations: Vec<(Relation, String)>,
    /// R12 BRIDGE — English cognates via the shared Indo-European root, for the
    /// minority of Thai words with Sanskrit/Pali → PIE ancestry. Only present for
    /// entries that survived BRIDGE-1's Kaikki cross-verification. Empty (the
    /// majority case — native Kra-Dai words have no PIE ancestry).
    pub english_cognates: Vec<EnglishCognate>,
}

/// One English cognate sharing an Indo-European root with a Thai loanword.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnglishCognate {
    /// The English word (e.g. "mother").
    pub word: String,
    /// The reconstructed PIE root, DISPLAY-ONLY (e.g. "*méh₂tēr") — shown as
    /// context, not asserted as definitive beyond what BRIDGE-1 corroborated.
    pub pie_root: String,
    /// Provenance: the Kaikki etymology text that corroborated this entry's
    /// source-language claim (so the cognate line is traceable, not bare).
    pub corroboration: String,
}

impl Entry {
    /// A headword-only entry (e.g. a LEXiTRON word with no sense yet).
    pub fn headword_only(word: &str) -> Self {
        Entry {
            headword: word.to_string(),
            homograph: None,
            pronunciation: None,
            romanization: None,
            senses: Vec::new(),
            etymology: Vec::new(),
            sub_entries: Vec::new(),
            see_also: Vec::new(),
            relations: Vec::new(),
            english_cognates: Vec::new(),
        }
    }

    /// The first sense's definition, if any (back-compat convenience for callers
    /// that showed a single definition).
    pub fn primary_definition(&self) -> Option<&str> {
        self.senses.first().map(|s| s.definition.as_str())
    }

    /// The first sense's POS marker, if any.
    pub fn primary_pos_marker(&self) -> Option<&'static str> {
        self.senses.first().and_then(|s| s.pos).map(|p| p.marker())
    }

    /// Highest merge-priority among this entry's senses' sources (RID=5 … LEXiTRON=1).
    /// A headword-only entry (no senses) has priority 0 so it never shadows a
    /// real definition. Used by `Dictionary::get` to pick which homograph wins.
    pub fn best_source_priority(&self) -> u8 {
        self.senses.iter().map(|s| s.provenance.source.priority()).max().unwrap_or(0)
    }
}

// ── Dictionary store ─────────────────────────────────────────────────────────

/// The dictionary: a headword-indexed store of entries plus a frequency table.
/// Keyed by `(headword, homograph)` so homographs stay distinct.
#[derive(Default)]
pub struct Dictionary {
    entries: HashMap<(String, Option<u8>), Entry>,
    order: Vec<(String, Option<u8>)>,
    freq: HashMap<String, u64>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, entry: Entry) {
        let key = (entry.headword.clone(), entry.homograph);
        if !self.entries.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.entries.insert(key, entry);
    }

    /// Look up by headword (returns the first homograph if several).
    pub fn get(&self, word: &str) -> Option<&Entry> {
        // Prefer the highest-priority source among all entries that share this
        // headword (RID > HumanSeed > CoinedWord > Kaikki > LEXiTRON). This
        // matters because RID lists many common words ONLY as homographs
        // (กก ๑/๒/๓, กรรม ๑/๒, …) while Kaikki has a plain no-homograph entry
        // for the same word — a naive "no-homograph key first" lookup shadowed
        // ~644 real RID headwords behind Kaikki (measured, R10 Phase R2). Ties
        // (same priority) keep the exact no-homograph key, else insertion order.
        let mut best: Option<&Entry> = None;
        let mut best_pri = 0u8;
        for (h, homo) in self.order.iter().filter(|(h, _)| h == word) {
            if let Some(e) = self.entries.get(&(h.clone(), *homo)) {
                let pri = e.best_source_priority();
                if best.is_none()
                    || pri > best_pri
                    // on a tie, prefer the plain no-homograph entry for stability
                    || (pri == best_pri && homo.is_none())
                {
                    best = Some(e);
                    best_pri = pri;
                }
            }
        }
        best
    }

    /// All headwords (deduplicated), in insertion order.
    pub fn words(&self) -> impl Iterator<Item = &str> {
        self.order.iter().map(|(h, _)| h.as_str())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

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

    pub fn frequency(&self, word: &str) -> u64 {
        self.freq.get(word).copied().unwrap_or(0)
    }

    /// The words that carry a frequency, ranked by descending frequency
    /// (ties broken alphabetically for determinism). Used for
    /// frequency-weighted coverage: "of the N most common Thai words, how many
    /// does the dictionary define?"
    pub fn freq_ranked_words(&self) -> Vec<&str> {
        let mut v: Vec<(&str, u64)> = self.freq.iter().map(|(w, &c)| (w.as_str(), c)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        v.into_iter().map(|(w, _)| w).collect()
    }

    pub fn all_entries(&self) -> impl Iterator<Item = &Entry> {
        self.order.iter().filter_map(move |k| self.entries.get(k))
    }
}

// ── Seed data ─────────────────────────────────────────────────────────────────

/// The 20 hand-curated seed entries, migrated to the RID-shaped model with
/// `Provenance::seed()` (HumanSeed / CC0 / Confirmed). Every audited relation
/// decision from the 2026-09-13 audit is preserved verbatim.
pub fn seed_entries() -> Vec<Entry> {
    use Relation::*;
    // (headword, pos, definition, relations)
    let mk = |word: &str, pos: Pos, def: &str, rels: &[(Relation, &str)]| Entry {
        headword: word.to_string(),
        homograph: None,
        pronunciation: None,
        romanization: None,
        senses: vec![Sense::simple(pos, def)],
        etymology: Vec::new(),
        sub_entries: Vec::new(),
        see_also: Vec::new(),
        relations: rels.iter().map(|(r, w)| (*r, w.to_string())).collect(),
        english_cognates: Vec::new(),
    };
    vec![
        mk("แมว", Pos::Nam, "สัตว์เลี้ยงลูกด้วยนมชนิดหนึ่ง เลี้ยงไว้ในบ้าน จับหนูเป็นอาหาร",
            &[(IsA, "สัตว์"), (Category, "สัตว์เลี้ยง"), (SeeAlso, "เสือ")]),
        mk("สุนัข", Pos::Nam, "สัตว์เลี้ยงลูกด้วยนมชนิดหนึ่ง เลี้ยงไว้เฝ้าบ้าน; หมา",
            &[(IsA, "สัตว์"), (Synonym, "หมา"), (Category, "สัตว์เลี้ยง")]),
        mk("หมา", Pos::Nam, "สุนัข",
            &[(Synonym, "สุนัข"), (IsA, "สัตว์")]),
        mk("เสือ", Pos::Nam, "สัตว์กินเนื้อขนาดใหญ่ในวงศ์แมว",
            &[(IsA, "สัตว์"), (SeeAlso, "แมว")]),
        mk("สัตว์", Pos::Nam, "สิ่งมีชีวิตที่เคลื่อนไหวได้และกินสิ่งอื่นเป็นอาหาร",
            &[(Category, "สิ่งมีชีวิต")]),
        mk("ใหญ่", Pos::Wiset, "มีขนาดโตกว่าปรกติ",
            &[(Antonym, "เล็ก")]),
        mk("เล็ก", Pos::Wiset, "มีขนาดย่อมกว่าปรกติ",
            &[(Antonym, "ใหญ่")]),
        mk("สุข", Pos::Nam, "ความสบายกายสบายใจ",
            &[(Antonym, "ทุกข์"), (RelatedTo, "ความสุข")]),
        mk("ทุกข์", Pos::Nam, "ความไม่สบายกายไม่สบายใจ",
            &[(Antonym, "สุข")]),
        mk("ครู", Pos::Nam, "ผู้สั่งสอนศิษย์; ผู้ถ่ายทอดความรู้",
            &[(Synonym, "อาจารย์"), (Category, "การศึกษา"), (SeeAlso, "โรงเรียน")]),
        mk("อาจารย์", Pos::Nam, "ผู้สั่งสอนวิชาความรู้ในระดับสูง",
            &[(Synonym, "ครู"), (Category, "การศึกษา")]),
        mk("นักเรียน", Pos::Nam, "ผู้เรียนในโรงเรียน",
            &[(Category, "การศึกษา"), (SeeAlso, "โรงเรียน"), (RelatedTo, "ครู")]),
        mk("โรงเรียน", Pos::Nam, "สถานที่สำหรับสอนและเรียนหนังสือ",
            &[(Category, "การศึกษา"), (SeeAlso, "นักเรียน")]),
        mk("หนังสือ", Pos::Nam, "เอกสารที่เขียนหรือพิมพ์เป็นเล่มสำหรับอ่าน",
            &[(Category, "การศึกษา"), (SeeAlso, "พจนานุกรม")]),
        mk("พจนานุกรม", Pos::Nam, "หนังสือรวบรวมคำและความหมายเรียงตามลำดับตัวอักษร",
            &[(IsA, "หนังสือ"), (Category, "การศึกษา"), (SeeAlso, "คำ")]),
        mk("คำ", Pos::Nam, "เสียงพูดหรือตัวหนังสือที่มีความหมาย",
            &[(SeeAlso, "ความหมาย"), (SeeAlso, "ภาษา")]),
        mk("ความหมาย", Pos::Nam, "สิ่งที่คำหรือข้อความนั้นสื่อให้เข้าใจ",
            &[(SeeAlso, "คำ")]),
        mk("ภาษา", Pos::Nam, "เสียงหรือตัวหนังสือที่ใช้สื่อความหมายกัน",
            &[(SeeAlso, "คำ"), (Category, "การศึกษา")]),
        mk("อ่าน", Pos::Kri, "ดูตัวหนังสือแล้วเข้าใจความหมาย; ออกเสียงตามตัวหนังสือ",
            &[(SeeAlso, "หนังสือ"), (RelatedTo, "เขียน")]),
        mk("เขียน", Pos::Kri, "ทำให้เป็นตัวหนังสือหรือรูปด้วยเครื่องมือ",
            &[(RelatedTo, "อ่าน"), (SeeAlso, "หนังสือ")]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut d = Dictionary::new();
        d.insert(Entry {
            headword: "แมว".into(),
            homograph: None,
            pronunciation: None,
            romanization: None,
            senses: vec![Sense::simple(Pos::Nam, "สัตว์")],
            etymology: vec![],
            sub_entries: vec![],
            see_also: vec![],
            relations: vec![],
            english_cognates: vec![],
        });
        assert_eq!(d.len(), 1);        assert_eq!(d.get("แมว").unwrap().primary_pos_marker(), Some("น."));
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
    fn pos_subject_register_roundtrip_all_values() {
        // Task 1 acceptance: round-trip every controlled-vocabulary value.
        for &p in Pos::all() {
            assert_eq!(Pos::from_marker(p.marker()), Some(p), "POS {:?}", p);
        }
        for s in Subject::all_named() {
            // from_marker(tag) must return the same named variant (not Other).
            let round = Subject::from_marker(s.tag());
            assert_eq!(round, s, "Subject {:?}", s);
            assert!(!matches!(round, Subject::Other(_)));
        }
        for &r in Register::all() {
            assert_eq!(Register::from_marker(r.marker()), Some(r), "Register {:?}", r);
        }
        // Unknown subject falls back to Other.
        assert!(matches!(Subject::from_marker("ไม่มีสาขานี้"), Subject::Other(_)));
    }

    #[test]
    fn seed_is_nonempty_and_relations_resolve() {
        let entries = seed_entries();
        assert!(entries.len() >= 15);
        let words: std::collections::HashSet<_> =
            entries.iter().map(|e| e.headword.as_str()).collect();
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
        assert!(resolved * 2 >= total, "too many dangling relation targets");
    }

    #[test]
    fn seed_entries_carry_seed_provenance() {
        for e in seed_entries() {
            for s in &e.senses {
                assert_eq!(s.provenance.source, Source::HumanSeed);
                assert_eq!(s.provenance.license, License::Cc0);
                assert_eq!(s.provenance.confidence, RelationConfidence::Confirmed);
            }
        }
    }

    #[test]
    fn antonyms_are_only_true_lexical_opposites() {
        let allowed_antonym_pairs: std::collections::HashSet<(&str, &str)> = [
            ("ใหญ่", "เล็ก"), ("เล็ก", "ใหญ่"), ("สุข", "ทุกข์"), ("ทุกข์", "สุข"),
        ].into_iter().collect();
        for e in seed_entries() {
            for (rel, tgt) in &e.relations {
                if *rel == Relation::Antonym {
                    assert!(
                        allowed_antonym_pairs.contains(&(e.headword.as_str(), tgt.as_str())),
                        "'{} ตรงข้ามกับ {}' is not a true lexical antonym", e.headword, tgt
                    );
                }
            }
        }
    }

    #[test]
    fn known_role_pair_is_relatedto_not_antonym() {
        let entries = seed_entries();
        let nakrian = entries.iter().find(|e| e.headword == "นักเรียน").unwrap();
        let (rel, _) = nakrian
            .relations.iter().find(|(_, t)| t == "ครู")
            .expect("นักเรียน should still relate to ครู");
        assert_eq!(*rel, Relation::RelatedTo);
    }

    #[test]
    fn get_prefers_highest_priority_source_across_homographs() {
        // R10 Phase R2 regression: RID lists many common words ONLY as homographs
        // (กก ๑/๒/๓) while Kaikki has a plain no-homograph entry. `get` must return
        // the RID homograph (priority 5), not the shadowing Kaikki entry (priority 2).
        let mut dict = Dictionary::new();
        // Kaikki plain entry (inserted first; lower priority).
        let mut kaikki = Entry::headword_only("กก");
        kaikki.senses.push(Sense {
            pos: Some(Pos::Nam),
            subject: None,
            register: None,
            definition: "kaikki gloss".to_string(),
            examples: vec![],
            classifiers: vec![],
            provenance: Provenance {
                source: Source::Kaikki,
                license: License::CcBySa,
                confidence: RelationConfidence::Unverified,
            },
        });
        dict.insert(kaikki);
        // RID homograph entry (higher priority, but non-None key).
        let mut rid = Entry::headword_only("กก");
        rid.homograph = Some(1);
        rid.senses.push(Sense {
            pos: Some(Pos::Nam),
            subject: None,
            register: None,
            definition: "rid definition".to_string(),
            examples: vec![],
            classifiers: vec![],
            provenance: Provenance {
                source: Source::Rid,
                license: License::OrstEducational,
                confidence: RelationConfidence::Confirmed,
            },
        });
        dict.insert(rid);

        let got = dict.get("กก").expect("กก present");
        assert_eq!(got.primary_definition(), Some("rid definition"));
        assert_eq!(got.senses[0].provenance.source, Source::Rid);
    }
}
