//! RID (พจนานุกรม ฉบับราชบัณฑิตยสถาน ๒๕๕๔) importer — Round 5, Task 8.
//!
//! This is the **competition-day adapter**: when the organizer hands out the
//! real RID dataset, it drops into `data/rid/` and this importer ingests it.
//! Until then it is developed against hand-copied fixtures in
//! `wacha/tests/fixtures/rid/` (fair-use test data, 5–10 entries — NOT a bulk
//! harvest of `dictionary.orst.go.th`).
//!
//! The importer parses a **stable text projection** of the on-screen RID entry
//! layout (§1.1 of `NEXT_STEPS_R5.md`): headword + Thai-numeral homograph,
//! bracketed คำอ่าน, per-sense `[POS]` markers, `(สาขาวิชา)` tags, `{register}`,
//! Thai-numeral sense numbers, `(ป. …; ส. …)` etymology, `ลูกคำ`, and `ดู`
//! cross-references. If the real data ships a different serialization, only
//! [`RidImporter::parse_entry`] (the adapter fn) needs editing — see
//! `COMPETITION_DAY.md`.

use crate::dictionary::{
    Entry, Etymology, License, Pos, Provenance, Relation, RelationConfidence, Sense, Subject,
    Register, Source,
};
use crate::import::{ImportError, ImportResult, Importer};
use std::path::Path;

pub struct RidImporter;

impl Importer for RidImporter {
    fn name(&self) -> &'static str {
        "rid"
    }
    fn source(&self) -> Source {
        Source::Rid
    }

    /// `path` may be a single `.txt` file or a directory of them. Each file is
    /// a sequence of blank-line-separated entry blocks.
    fn load(&self, path: &Path) -> ImportResult<Vec<Entry>> {
        let mut texts: Vec<String> = Vec::new();
        if path.is_dir() {
            let mut files: Vec<_> = std::fs::read_dir(path)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().map(|x| x == "txt").unwrap_or(false))
                .collect();
            files.sort();
            for f in files {
                texts.push(std::fs::read_to_string(f)?);
            }
        } else {
            texts.push(std::fs::read_to_string(path)?);
        }

        let mut entries = Vec::new();
        let mut total_blocks = 0usize;
        let mut n_errors = 0usize;
        let mut skipped: Vec<String> = Vec::new();
        for text in &texts {
            for block in split_blocks(text) {
                total_blocks += 1;
                match Self::parse_entry(&block) {
                    Ok(Some(e)) => entries.push(e),
                    Ok(None) => {}
                    Err(e) => {
                        n_errors += 1;
                        if skipped.len() < 5 {
                            skipped.push(format!("{}\n-> {e}", block.lines().next().unwrap_or("")));
                        }
                    }
                }
            }
        }
        if total_blocks > 0 && n_errors * 20 > total_blocks {
            // >5% of blocks unparseable — treat as a real format mismatch.
            return Err(format!(
                "RID load: {n_errors}/{total_blocks} blocks failed to parse (>5%) — \
                 likely a grammar mismatch. First failures:\n{}",
                skipped.join("\n")
            )
            .into());
        }
        if n_errors > 0 {
            eprintln!("RID: skipped {n_errors}/{total_blocks} malformed blocks (kept {} entries)", entries.len());
        }
        Ok(entries)
    }
}

impl RidImporter {
    /// **The adapter.** Parse one RID entry block (the documented line grammar)
    /// into an [`Entry`]. Edit THIS function first if the organizer's format
    /// differs from the fixtures. Returns `Ok(None)` for a comment/empty block.
    pub fn parse_entry(block: &str) -> Result<Option<Entry>, ImportError> {
        let mut headword: Option<String> = None;
        let mut homograph: Option<u8> = None;
        let mut reading: Option<String> = None;
        let mut etymology: Vec<Etymology> = Vec::new();
        let mut sub_entries: Vec<String> = Vec::new();
        let mut see_also: Vec<String> = Vec::new();
        let mut relations: Vec<(Relation, String)> = Vec::new();
        let mut senses: Vec<Sense> = Vec::new();

        for raw in block.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(rest) = line.strip_prefix("HEADWORD:") {
                let (hw, homo) = parse_headword(rest.trim());
                headword = Some(hw);
                homograph = homo;
            } else if let Some(rest) = line.strip_prefix("READING:") {
                let r = rest.trim().trim_start_matches('[').trim_end_matches(']').trim();
                if !r.is_empty() {
                    reading = Some(format!("[{r}]"));
                }
            } else if let Some(rest) = line.strip_prefix("ETYM:") {
                etymology.extend(parse_etymology(rest.trim()));
            } else if let Some(rest) = line.strip_prefix("SUBENTRIES:") {
                for w in rest.split(',') {
                    let w = w.trim();
                    if !w.is_empty() {
                        sub_entries.push(w.to_string());
                    }
                }
            } else if let Some(rest) = line.strip_prefix("SENSE:") {
                let (sense, xref, parenthetical) = parse_sense(rest.trim())?;
                if let Some(x) = xref {
                    if !see_also.contains(&x) {
                        see_also.push(x.clone());
                    }
                    // Only a PRIMARY (non-parenthetical) reference becomes a graph
                    // SeeAlso relation. Parenthetical "(ดู X)" is a supplementary
                    // pointer — kept in see_also for display, but not a ranked
                    // relation competing with audited synonyms (R10 Phase R2).
                    if !parenthetical {
                        let rel = (Relation::SeeAlso, x);
                        if !relations.contains(&rel) {
                            relations.push(rel);
                        }
                    }
                }
                senses.push(sense);
            }
            // Unknown line labels are ignored (forward-compatible).
        }

        let Some(headword) = headword else {
            return Ok(None); // no HEADWORD -> not an entry block
        };

        Ok(Some(Entry {
            headword,
            homograph,
            pronunciation: reading,
            romanization: None,
            senses,
            etymology,
            sub_entries,
            see_also,
            relations,
        }))
    }
}

/// Split a file into blank-line-separated blocks.
fn split_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut cur = String::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            if !cur.trim().is_empty() {
                blocks.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push_str(line);
            cur.push('\n');
        }
    }
    if !cur.trim().is_empty() {
        blocks.push(cur);
    }
    blocks
}

/// `"แมว ๑"` -> ("แมว", Some(1)); `"ปิตุ"` -> ("ปิตุ", None).
fn parse_headword(s: &str) -> (String, Option<u8>) {
    let thai_digits = [
        ('๐', 0u8), ('๑', 1), ('๒', 2), ('๓', 3), ('๔', 4),
        ('๕', 5), ('๖', 6), ('๗', 7), ('๘', 8), ('๙', 9),
    ];
    // A trailing token that is a single Thai numeral = homograph number.
    if let Some((head, last)) = s.rsplit_once(' ') {
        let last = last.trim();
        if last.chars().count() == 1 {
            if let Some((_, n)) = thai_digits.iter().find(|(c, _)| Some(*c) == last.chars().next()) {
                return (head.trim().to_string(), Some(*n));
            }
        }
    }
    (s.trim().to_string(), None)
}

/// Parse `(ป. ปิตา; ส. ปิตฤ)` into Etymology entries. Language markers:
/// ป.=บาลี ส.=สันสกฤต อ.=อังกฤษ ข.=เขมร.
fn parse_etymology(s: &str) -> Vec<Etymology> {
    let inner = s.trim().trim_start_matches('(').trim_end_matches(')');
    let mut out = Vec::new();
    for part in inner.split(';') {
        let part = part.trim();
        if let Some((lang, form)) = part.split_once(' ') {
            let lang = lang.trim().trim_end_matches('.').to_string();
            let form = form.trim().to_string();
            if !form.is_empty() {
                out.push(Etymology { lang, form });
            }
        }
    }
    out
}

/// Parse a SENSE line: `[POS] (สาขาวิชา) {register} (๑) <def> [ดู <xref>]`.
/// Returns the Sense and an optional `ดู` cross-reference target.
fn parse_sense(s: &str) -> Result<(Sense, Option<String>, bool), ImportError> {
    let mut rest = s.trim();
    let mut pos: Option<Pos> = None;
    let mut subject: Option<Subject> = None;
    let mut register: Option<Register> = None;

    // [POS]
    if let Some(end) = rest.strip_prefix('[').and_then(|r| r.find(']').map(|i| (r, i))) {
        let (r, i) = end;
        let marker = r[..i].trim();
        pos = Pos::from_marker(marker);
        rest = r[i + 1..].trim();
    }
    // (สาขาวิชา) OR (register-in-parens like (ไว)) — try subject/register.
    // The layout uses (X) for subject fields; a numbered sense is (๑). We only
    // treat a leading (X) as a tag if X is NOT a Thai numeral.
    while rest.starts_with('(') {
        if let Some(i) = rest.find(')') {
            let tag = rest[1..i].trim();
            if is_thai_numeral(tag) {
                // sense number — drop it, keep going to the definition
                rest = rest[i + 1..].trim();
                continue;
            }
            // register?
            if let Some(reg) = Register::from_marker(tag) {
                register = Some(reg);
            } else if subject.is_none() {
                subject = Some(Subject::from_marker(tag));
            }
            rest = rest[i + 1..].trim();
        } else {
            break;
        }
    }

    // `ดู <xref>` cross-reference. Two legitimate shapes in the real data:
    //   (a) pure redirect  — the sense IS "ดู X" (no standalone definition), and
    //   (b) trailing ref   — "def... ดู X" at the END of the sense.
    // We DON'T treat every mid-prose "ดู" as a reference (it is also the ordinary
    // verb "to look"); we only accept "ดู" whose following FIRST token is a clean
    // Thai headword and take just that token (dropping the multi-target / stray-
    // punctuation tail that used to inject garbage like "ยาม)." — measured, R10).
    let mut xref = None;
    // Whether the reference was PARENTHETICAL "(ดู X)" — a supplementary "compare"
    // pointer — vs a primary redirect. Parenthetical refs are shown in the entry
    // (see_also) but do NOT become first-class graph relations that compete with
    // audited synonyms in the ranked related-words list (R10 Phase R2: RID
    // homographs like ครู๒ "(ดู ยาม)" otherwise displaced ครู→ผู้สอน from top-5).
    let mut xref_parenthetical = false;
    if let Some(idx) = rest.find("ดู ") {
        // Parenthetical if the char just before "ดู" is '(' (ignoring a space).
        let before = rest[..idx].trim_end();
        xref_parenthetical = before.ends_with('(');
        let tail = rest[idx + "ดู ".len()..].trim();
        // First token = up to the first separator (space, comma, paren, ., ;).
        // A trailing hyphen is part of the ref (RID prefix-forms: กฤด-, กฐิน-).
        let first: String = tail
            .chars()
            .take_while(|c| {
                !c.is_whitespace()
                    && *c != ','
                    && *c != '('
                    && *c != ')'
                    && *c != '.'
                    && *c != ';'
            })
            .collect();
        let first = first.trim();
        let core = first.trim_end_matches('-');
        // Accept only a real Thai headword (Thai letters + optional trailing
        // hyphen, no numerals/markup, ≥2 chars).
        let is_thai_word = !core.is_empty()
            && core.chars().count() >= 2
            && core.chars().all(|c| {
                ('\u{0E00}'..='\u{0E7F}').contains(&c) && !('๐'..='๙').contains(&c)
            });
        if is_thai_word {
            xref = Some(first.to_string());
            // Drop the "ดู …" clause from the definition; a pure redirect (idx==0)
            // then leaves an empty definition, which is the intended redirect sense.
            rest = rest[..idx].trim();
            // For a parenthetical "(ดู X)", also drop the now-dangling "(".
            if xref_parenthetical {
                rest = rest.trim_end_matches('(').trim();
            }
        }
    }

    let definition = rest.trim().to_string();
    if definition.is_empty() && xref.is_none() {
        return Err("SENSE has no definition or cross-reference".into());
    }

    Ok((
        Sense {
            pos,
            subject,
            register,
            definition,
            examples: Vec::new(),
            classifiers: Vec::new(),
            provenance: Provenance {
                source: Source::Rid,
                license: License::OrstEducational,
                confidence: RelationConfidence::Confirmed,
            },
        },
        xref,
        xref_parenthetical,
    ))
}

fn is_thai_numeral(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| ('๐'..='๙').contains(&c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rid")
    }

    fn load() -> Vec<Entry> {
        RidImporter.load(&fixture_path()).expect("fixtures load")
    }

    #[test]
    fn loads_all_fixture_entries() {
        let e = load();
        // 6 headword blocks in the fixture.
        assert_eq!(e.len(), 6, "expected 6 fixture entries, got {}", e.len());
    }

    #[test]
    fn parses_homograph_numbers() {
        let e = load();
        let maew: Vec<_> = e.iter().filter(|x| x.headword == "แมว").collect();
        assert_eq!(maew.len(), 2, "แมว ๑ and แมว ๒");
        assert!(maew.iter().any(|x| x.homograph == Some(1)));
        assert!(maew.iter().any(|x| x.homograph == Some(2)));
    }

    #[test]
    fn parses_multi_sense_entry() {
        let e = load();
        let kam = e.iter().find(|x| x.headword == "กรรม").expect("กรรม");
        assert_eq!(kam.senses.len(), 2, "กรรม has 2 senses");
        // second sense carries the (ไว) subject tag.
        assert_eq!(kam.senses[1].subject, Some(Subject::Wai));
    }

    #[test]
    fn parses_etymology() {
        let e = load();
        let pitu = e.iter().find(|x| x.headword == "ปิตุ").expect("ปิตุ");
        // (ป. ปิตา; ส. ปิตฤ) -> two etymology entries.
        assert_eq!(pitu.etymology.len(), 2);
        assert!(pitu.etymology.iter().any(|et| et.lang == "ป" && et.form == "ปิตา"));
        assert!(pitu.etymology.iter().any(|et| et.lang == "ส" && et.form == "ปิตฤ"));
    }

    #[test]
    fn parses_subentries_luk_kham() {
        let e = load();
        let maew1 = e.iter().find(|x| x.headword == "แมว" && x.homograph == Some(1)).unwrap();
        assert!(maew1.sub_entries.contains(&"แมวคราว".to_string()));
        assert!(maew1.sub_entries.contains(&"แมวดาว".to_string()));
    }

    #[test]
    fn parses_du_cross_reference() {
        let e = load();
        let bai = e.iter().find(|x| x.headword == "ใบขนุน").expect("ใบขนุน");
        // "ดู กระดังงาจีน" -> see_also + a SeeAlso relation.
        assert!(bai.see_also.contains(&"กระดังงาจีน".to_string()));
        assert!(bai
            .relations
            .iter()
            .any(|(r, t)| *r == Relation::SeeAlso && t == "กระดังงาจีน"));
    }

    #[test]
    fn pos_and_reading_parsed() {
        let e = load();
        let khian = e.iter().find(|x| x.headword == "เขียน").unwrap();
        assert_eq!(khian.pronunciation.as_deref(), Some("[เขียน]"));
        assert_eq!(khian.senses[0].pos, Some(Pos::Kri));
        assert_eq!(khian.senses[0].provenance.source, Source::Rid);
    }
}
