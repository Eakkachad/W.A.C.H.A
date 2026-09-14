//! Deterministic intent router (R12 Phase INTENT-1).
//!
//! Turns "one free-text box" into the right job WITHOUT a trained classifier:
//! a small, layered, fully-explainable pipeline. This module is layer 1 — literal
//! keyword/pattern rules, one per intent, checked in priority order. Layer 2 (the
//! `thai2fit_wv` cosine fallback) lives on `Engine` (INTENT-2); layer 3 is the
//! unchanged general lookup→reverse default.
//!
//! Design decision (settled, not re-litigated — see NEXT_STEPS_R12 §0): NO ML
//! model for intent. Every decision here is a literal rule with a human-readable
//! `reason`, so it is testable per-rule and explainable in the UI.

/// The five job modes (mirroring `wacha/web/index.html`'s cards) + the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Naming,
    Writing,
    Specialized,
    Roots,
    Translit,
    General,
}

impl Intent {
    /// The web mode key (matches index.html `data-mode` / `MODES`).
    pub fn mode_key(self) -> &'static str {
        match self {
            Intent::Naming => "naming",
            Intent::Writing => "writing",
            Intent::Specialized => "specialized",
            Intent::Roots => "roots",
            Intent::Translit => "translit",
            Intent::General => "general",
        }
    }
    /// Short Thai label for the confirmation line ("เราคิดว่าคุณอยาก [X]").
    pub fn thai_label(self) -> &'static str {
        match self {
            Intent::Naming => "ตั้งชื่อ",
            Intent::Writing => "หาคำ/คำคล้องจอง/ระดับภาษา",
            Intent::Specialized => "ศัพท์เฉพาะทาง",
            Intent::Roots => "รากคำ/วิวัฒนาการ",
            Intent::Translit => "คำทับศัพท์",
            Intent::General => "ค้นหาทั่วไป",
        }
    }
}

/// How the intent was decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// A keyword/pattern rule fired (layer 1) — certain, explainable.
    Rule,
    /// The thai2fit_wv cosine fallback chose it (layer 2) — softer.
    Vector,
    /// Nothing matched; default General behavior (layer 3).
    Default,
}

/// The classification result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentGuess {
    pub intent: Intent,
    /// Which rule/keyword fired (powers the UI "เพราะ…" explanation + tests).
    pub reason: String,
    pub confidence: Confidence,
}

/// Naming keywords/patterns.
const NAMING: &[&str] = &["ตั้งชื่อ", "ชื่อลูก", "ชื่อบริษัท", "ชื่อสัตว์เลี้ยง", "ชื่อร้าน", "อยากได้ชื่อ", "ชื่อที่แปลว่า", "ชื่อมงคล", "ตั้งชื่อว่า"];
/// Writing (register / rhyme).
const WRITING: &[&str] = &["ทางการ", "ราชาศัพท์", "ภาษาปาก", "โบราณ", "คล้องจอง", "สัมผัส", "แต่งเพลง", "แต่งกลอน", "คำสุภาพ", "เขียนให้"];
/// Specialized domains.
const SPECIALIZED: &[&str] = &["ทางการแพทย์", "ศัพท์แพทย์", "จิตวิทยา", "ปรัชญา", "ศัพท์บัญญัติ", "ศัพท์เฉพาะ", "แพทยศาสตร์"];
/// Roots / etymology / evolution.
const ROOTS: &[&str] = &["รากศัพท์", "รากคำ", "มาจากไหน", "มาจากภาษา", "ประวัติคำ", "วิวัฒนาการ", "ยุคไหน", "สมัยก่อนแปลว่า", "เปลี่ยนความหมาย"];
/// Transliteration.
const TRANSLIT: &[&str] = &["ทับศัพท์", "สะกดยังไง", "สะกดอย่างไร", "ภาษาอังกฤษเขียน", "เขียนเป็นไทย", "คำทับศัพท์"];

fn contains_any(q: &str, kws: &[&str]) -> Option<String> {
    kws.iter().find(|k| q.contains(**k)).map(|k| k.to_string())
}

fn has_latin(q: &str) -> bool {
    q.chars().any(|c| c.is_ascii_alphabetic())
}

/// Layer 1: classify by keyword/pattern rules, in priority order. Returns a
/// `Rule`-confidence guess if any rule fires, else `General`/`Default` (caller
/// may then invoke the vector fallback before accepting the default).
pub fn classify_intent(query: &str) -> IntentGuess {
    let q = query.trim();
    if q.is_empty() {
        return IntentGuess { intent: Intent::General, reason: "empty query".into(), confidence: Confidence::Default };
    }

    // Priority order: the more specific/explicit intents first. Naming, Roots,
    // Specialized, Writing are phrase-driven; Translit's Latin-script rule is
    // last among the "sure" rules because a Thai naming request could mention an
    // English word — but a query that is DExplicit about ทับศัพท์ still wins via
    // its keyword before the bare Latin-char heuristic.
    if let Some(k) = contains_any(q, TRANSLIT) {
        return rule(Intent::Translit, format!("มีคำว่า “{k}”"));
    }
    if let Some(k) = contains_any(q, NAMING) {
        return rule(Intent::Naming, format!("มีคำว่า “{k}”"));
    }
    if let Some(k) = contains_any(q, ROOTS) {
        return rule(Intent::Roots, format!("มีคำว่า “{k}”"));
    }
    if let Some(k) = contains_any(q, SPECIALIZED) {
        return rule(Intent::Specialized, format!("มีคำว่า “{k}”"));
    }
    if let Some(k) = contains_any(q, WRITING) {
        return rule(Intent::Writing, format!("มีคำว่า “{k}”"));
    }
    // Bare Latin-script input (a loanword / English word) → transliteration.
    if has_latin(q) {
        return rule(Intent::Translit, "มีตัวอักษรภาษาอังกฤษ".to_string());
    }
    // No rule fired — caller may try the vector fallback; default is General.
    IntentGuess { intent: Intent::General, reason: "ไม่มีกฎใดตรง".into(), confidence: Confidence::Default }
}

fn rule(intent: Intent, reason: String) -> IntentGuess {
    IntentGuess { intent, reason, confidence: Confidence::Rule }
}

/// Seed WORDS per intent for the R12 INTENT-2 thai2fit_wv centroid fallback.
/// Single words (thai2fit is a word2vec model) chosen to be in-vocabulary and
/// representative of each job. The fallback averages these into a centroid per
/// intent and compares the query's own averaged vector against them.
pub fn intent_seed_words() -> Vec<(Intent, Vec<&'static str>)> {
    vec![
        (Intent::Naming, vec!["ชื่อ", "ตั้งชื่อ", "นาม", "มงคล"]),
        (Intent::Writing, vec!["ทางการ", "สุภาพ", "กลอน", "สัมผัส", "ราชาศัพท์"]),
        (Intent::Specialized, vec!["แพทย์", "จิตวิทยา", "ปรัชญา", "วิชาการ", "ศัพท์"]),
        (Intent::Roots, vec!["รากศัพท์", "ที่มา", "ประวัติ", "วิวัฒนาการ", "ภาษาบาลี"]),
        (Intent::Translit, vec!["ทับศัพท์", "สะกด", "อังกฤษ", "ภาษาต่างประเทศ"]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // Per-rule: one query that should hit, one near-miss that shouldn't (falls to General).
    #[test]
    fn each_rule_hits_and_near_misses_fall_through() {
        assert_eq!(classify_intent("อยากตั้งชื่อลูกสาว").intent, Intent::Naming);
        assert_eq!(classify_intent("คำคล้องจองกับ บ้าน").intent, Intent::Writing);
        assert_eq!(classify_intent("ศัพท์แพทย์ของอาการนี้").intent, Intent::Specialized);
        assert_eq!(classify_intent("รากศัพท์ของคำว่า กรรม").intent, Intent::Roots);
        assert_eq!(classify_intent("computer สะกดยังไง").intent, Intent::Translit);
        // near-misses: bare Thai words with no trigger → General
        assert_eq!(classify_intent("แมว").intent, Intent::General);
        assert_eq!(classify_intent("ความสุข").intent, Intent::General);
    }

    #[test]
    fn latin_script_routes_to_translit() {
        assert_eq!(classify_intent("algorithm").intent, Intent::Translit);
        assert_eq!(classify_intent("internet").confidence, Confidence::Rule);
    }

    #[test]
    fn empty_is_general() {
        assert_eq!(classify_intent("").intent, Intent::General);
        assert_eq!(classify_intent("   ").intent, Intent::General);
    }

    #[test]
    fn reason_is_populated() {
        let g = classify_intent("อยากได้ชื่อที่แปลว่าเข้มแข็ง");
        assert_eq!(g.intent, Intent::Naming);
        assert!(g.reason.contains("อยากได้ชื่อ") || g.reason.contains("ชื่อที่แปลว่า"));
    }

    // Table-driven regression fixture: >=20 real-shaped queries with expected intent.
    // A future rule change that silently breaks any of these will fail here.
    #[test]
    fn regression_fixture_20_real_queries() {
        let cases: &[(&str, Intent)] = &[
            ("อยากตั้งชื่อลูกชายให้แปลว่าดวงอาทิตย์", Intent::Naming),
            ("ตั้งชื่อบริษัทเทคโนโลยี", Intent::Naming),
            ("ชื่อสัตว์เลี้ยงน่ารักๆ", Intent::Naming),
            ("อยากได้ชื่อที่แปลว่าน้ำ", Intent::Naming),
            ("ชื่อมงคลสำหรับลูกสาว", Intent::Naming),
            ("หาคำคล้องจองกับคำว่า รัก", Intent::Writing),
            ("อยากเขียนให้ดูทางการ", Intent::Writing),
            ("ราชาศัพท์ของคำว่า กิน", Intent::Writing),
            ("คำสุภาพแทนคำว่า ตาย", Intent::Writing),
            ("แต่งกลอนต้องใช้คำสัมผัส", Intent::Writing),
            ("ศัพท์แพทย์ของอาการปวดหัว", Intent::Specialized),
            ("ศัพท์บัญญัติทางจิตวิทยา", Intent::Specialized),
            ("คำศัพท์ปรัชญาเรื่องความจริง", Intent::Specialized),
            ("รากศัพท์ของคำว่า เทวดา", Intent::Roots),
            ("คำว่า กรรม มาจากภาษาอะไร", Intent::Roots),
            ("ประวัติคำว่า โทรศัพท์", Intent::Roots),
            ("คำนี้สมัยก่อนแปลว่าอะไร วิวัฒนาการ", Intent::Roots),
            ("computer ทับศัพท์ว่าอะไร", Intent::Translit),
            ("cappuccino สะกดยังไง", Intent::Translit),
            ("marketing เขียนเป็นไทย", Intent::Translit),
            // bare words → General (the safe default)
            ("แมว", Intent::General),
            ("ประชาธิปไตย", Intent::General),
        ];
        let mut hits = 0;
        for (q, want) in cases {
            let got = classify_intent(q).intent;
            if got == *want {
                hits += 1;
            } else {
                eprintln!("MISS: {q:?} → {got:?}, wanted {want:?}");
            }
        }
        // Every fixture case must classify as expected (this is a regression guard).
        assert_eq!(hits, cases.len(), "{}/{} fixture queries classified correctly", hits, cases.len());
    }
}
