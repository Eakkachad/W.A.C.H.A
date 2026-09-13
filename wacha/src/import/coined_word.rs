//! ศัพท์บัญญัติ (ORST coined-word) importer — Round 5, Task 6.
//!
//! Parses the cached HTML fragments from `coined-word.orst.go.th`
//! (`data/coined_word_cache/<english>.html`, fetched once by
//! `scripts/fetch_coined_word.sh`) into RID-shaped [`Entry`]s.
//!
//! The source is a **term-equivalence database, not a definitional dictionary**:
//! each record is `English term ↔ Thai term(s)` within a **discipline** (สาขา).
//! The mapping is many-to-many — one English word (`field`) has different Thai
//! equivalents per discipline (สนาม / เขตข้อมูล / ฟีลด์ / …). That disambiguation
//! is the dataset's whole value, so we preserve it:
//!
//! - one [`Sense`] per (Thai term, discipline), with `subject` set and
//!   `Provenance { CoinedWord, OrstEducational, Confirmed }` (ORST authored
//!   these — the highest-precision relations in the system);
//! - the `definition` records the English equivalent + discipline, e.g.
//!   `"field (ศัพท์บัญญัติ · คอมพิวเตอร์และเทคโนโลยีสารสนเทศ)"`;
//! - Thai terms that are equivalents of the **same English word** are linked
//!   with [`Relation::Synonym`] so the relation graph gains a `CoinedWord`
//!   sense group ("คำอื่นที่บัญญัติจากคำอังกฤษเดียวกัน").
//!
//! Dependency-light: the HTML is parsed with a small hand-written scanner (no
//! scraper crate), tolerant of the site's irregular `<b>`/`&nbsp;` markup.

use crate::dictionary::{
    Entry, License, Pos, Provenance, Relation, RelationConfidence, Sense, Source, Subject,
};
use crate::import::{ImportResult, Importer};
use std::collections::BTreeMap;
use std::path::Path;

/// The 40-discipline list is hard-coded upstream in the fetch script's `book_id`
/// space; here we simply trust the discipline heading text in each panel (the
/// site's own label), mapping it to [`Subject`] (`Other` for the many that
/// aren't one of the 32 short RID สาขาวิชา tags — an honest, lossless fallback).
pub struct CoinedWordImporter;

/// One parsed (discipline, thai_term) row for an English query.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CoinedRow {
    discipline: String,
    thai: String,
}

impl Importer for CoinedWordImporter {
    fn name(&self) -> &'static str {
        "coined_word"
    }
    fn source(&self) -> Source {
        Source::CoinedWord
    }

    /// `path` is the cache **directory** (`data/coined_word_cache/`); every
    /// `*.html` file in it is one English query's response.
    fn load(&self, path: &Path) -> ImportResult<Vec<Entry>> {
        let mut entries: BTreeMap<String, Entry> = BTreeMap::new();
        // Deterministic file order.
        let mut files: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map(|x| x == "html").unwrap_or(false))
            .collect();
        files.sort();

        for file in files {
            let english = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let html = std::fs::read_to_string(&file)?;
            let rows = parse_html(&html);

            // Collect the distinct Thai terms this English word maps to (across
            // all disciplines) so we can cross-link them as CoinedWord synonyms.
            let mut thai_terms: Vec<String> = Vec::new();
            for row in &rows {
                if !thai_terms.contains(&row.thai) {
                    thai_terms.push(row.thai.clone());
                }
            }

            for row in &rows {
                let subject = Subject::from_marker(&row.discipline);
                // If from_marker didn't recognize it (returns Other only for
                // unknown), keep the full discipline name as Other.
                let subject = match subject {
                    Subject::Other(_) => Subject::Other(row.discipline.clone()),
                    known => known,
                };
                let definition = format!("{} (ศัพท์บัญญัติ · {})", english, row.discipline);
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
                    .entry(row.thai.clone())
                    .or_insert_with(|| Entry::headword_only(&row.thai));
                entry.senses.push(sense);
                // Link to the other Thai equivalents of the SAME English word.
                for other in &thai_terms {
                    if other != &row.thai {
                        let rel = (Relation::Synonym, other.clone());
                        if !entry.relations.contains(&rel) {
                            entry.relations.push(rel);
                        }
                    }
                }
            }
        }
        Ok(entries.into_values().collect())
    }
}

/// Reproduce the §1.2 table for one English query from its cached HTML:
/// an ordered list of (discipline, [Thai terms]) — the closing demo beat
/// ("one English word, N Thai equivalents, disambiguated by ORST's own
/// discipline tags"). Returns `None` if the term isn't cached.
pub fn field_view(cache_dir: &Path, english: &str) -> Option<Vec<(String, Vec<String>)>> {
    let file = cache_dir.join(format!("{english}.html"));
    let html = std::fs::read_to_string(file).ok()?;
    let rows = parse_html(&html);
    if rows.is_empty() {
        return None;
    }
    // Group by discipline, preserving first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut by_disc: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for r in rows {
        let e = by_disc.entry(r.discipline.clone()).or_default();
        if !order.contains(&r.discipline) {
            order.push(r.discipline.clone());
        }
        if !e.contains(&r.thai) {
            e.push(r.thai);
        }
    }
    Some(order.into_iter().map(|d| {
        let terms = by_disc.remove(&d).unwrap_or_default();
        (d, terms)
    }).collect())
}

/// Parse one cached HTML fragment into (discipline, thai_term) rows.
///
/// The markup, per discipline, looks like:
/// ```html
/// <div class="panel-title">…<b> คอมพิวเตอร์และเทคโนโลยีสารสนเทศ </b></div>
/// <div class="panel-body"><b><span…>field</span></b></b> …
///     <b>๑. เขตข้อมูล</b> … <b>๒. สนาม</b> …</div>
/// ```
/// We scan for each `panel-title`'s bold discipline name, then the following
/// `panel-body`'s bold Thai chunks (skipping the echoed English query span,
/// `หมวดย่อย` labels, and numbered-sense prefixes), splitting on `;`/`,` and
/// stripping bracketed scope notes.
fn parse_html(html: &str) -> Vec<CoinedRow> {
    let mut rows = Vec::new();
    // Split into per-discipline panels. Each inner discipline panel starts at a
    // `panel-heading`. We walk heading→body pairs.
    for panel in html.split("panel-heading").skip(1) {
        // Discipline = the first <b>…</b> after the heading (inside panel-title).
        let Some(discipline) = first_bold(panel) else { continue };
        let discipline = clean(&discipline);
        if discipline.is_empty() {
            continue;
        }
        // Body = everything up to the panel's close; find the panel-body chunk.
        let body = match panel.find("panel-body") {
            Some(i) => &panel[i..],
            None => continue,
        };
        for b in all_bold(body) {
            let t = clean(&b);
            // Skip the echoed English query (it sits inside a coloured <span>),
            // the หมวดย่อย label lines, and empties.
            if t.is_empty() || t.starts_with("หมวดย่อย") {
                continue;
            }
            // Skip a chunk that is purely Latin/ASCII (the echoed English term).
            if t.chars().all(|c| c.is_ascii()) {
                continue;
            }
            // Strip a leading Thai-numeral sense marker (๑./๒./…) or Arabic (1.).
            let t = strip_sense_number(&t);
            // Split synonym lists on ; and , then strip [scope notes].
            for part in t.split([';', ',']) {
                let term = strip_scope_note(part).trim().to_string();
                if !term.is_empty() && term.chars().any(is_thai) {
                    rows.push(CoinedRow {
                        discipline: discipline.clone(),
                        thai: term,
                    });
                }
            }
        }
    }
    // Dedup (discipline, thai) while preserving order.
    let mut seen = std::collections::HashSet::new();
    rows.retain(|r| seen.insert((r.discipline.clone(), r.thai.clone())));
    rows
}

fn is_thai(c: char) -> bool {
    ('\u{0E00}'..='\u{0E7F}').contains(&c)
}

/// The text of the first `<b>…</b>` in `s`.
fn first_bold(s: &str) -> Option<String> {
    let start = s.find("<b>")? + 3;
    let end = s[start..].find("</b>")? + start;
    Some(s[start..end].to_string())
}

/// The text of every `<b>…</b>` in `s`.
fn all_bold(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find("<b>") {
        let after = &rest[i + 3..];
        if let Some(j) = after.find("</b>") {
            out.push(after[..j].to_string());
            rest = &after[j + 4..];
        } else {
            break;
        }
    }
    out
}

/// Remove HTML tags, `&nbsp;`, and collapse whitespace.
fn clean(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Strip a leading Thai-numeral (๑. ๒. …) or Arabic (1.) sense marker.
fn strip_sense_number(s: &str) -> String {
    let s = s.trim_start();
    let thai_digits = ['๐', '๑', '๒', '๓', '๔', '๕', '๖', '๗', '๘', '๙'];
    let mut chars = s.chars().peekable();
    let mut prefix_len = 0;
    let mut saw_digit = false;
    while let Some(&c) = chars.peek() {
        if thai_digits.contains(&c) || c.is_ascii_digit() {
            saw_digit = true;
            prefix_len += c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }
    if saw_digit {
        // consume an optional '.' and following spaces
        let mut rest = &s[prefix_len..];
        rest = rest.trim_start_matches('.').trim_start();
        return rest.to_string();
    }
    s.to_string()
}

/// Strip a trailing bracketed scope note, e.g. `ฟีลด์ [ในพีชคณิตนามธรรม]` → `ฟีลด์`.
fn strip_scope_note(s: &str) -> String {
    match s.find('[') {
        Some(i) => s[..i].to_string(),
        None => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A trimmed copy of the real `field.html` response structure (two
    // disciplines: a simple single-term one and a numbered-sub-sense one with a
    // หมวดย่อย label).
    const FIELD_HTML: &str = r#"<div class="panel panel-info"><div class='panel-title'><br>&nbsp;&nbsp;<b>ผลการค้นหา "<span style='color: #003eff;'>field</span>"</b></div><div class="panel-body" ><br>
<div class="panel panel-info">
    <div class="panel-heading"><div class="panel-title"><i class='fa fa-book'></i><b> วิทยาศาสตร์ </b></div></div>
    <div class="panel-body"><b><span style="color: #003eff;">field</span></b></b>&nbsp;&nbsp;&nbsp;&nbsp;<b>สนาม</b><br><br></div></div>
<div class="panel panel-info">
    <div class="panel-heading"><div class="panel-title"><i class='fa fa-book'></i><b> คอมพิวเตอร์และเทคโนโลยีสารสนเทศ </b></div></div>
    <div class="panel-body"><b><span style="color: #003eff;">field</span></b></b><br><br>&nbsp;&nbsp;<b>๑. เขตข้อมูล</b><br>&nbsp;&nbsp;<b>หมวดย่อย &nbsp; </b>Computer/IT<br>&nbsp;&nbsp;<b>๒. สนาม</b><br></div></div>
<div class="panel panel-info">
    <div class="panel-heading"><div class="panel-title"><i class='fa fa-book'></i><b> คณิตศาสตร์ </b></div></div>
    <div class="panel-body"><b><span style="color: #003eff;">field</span></b></b>&nbsp;&nbsp;&nbsp;&nbsp;<b>ฟีลด์ [ในพีชคณิตนามธรรม]</b><br><br></div></div>
</div></div>"#;

    #[test]
    fn parses_disciplines_and_thai_terms() {
        let rows = parse_html(FIELD_HTML);
        // วิทยาศาสตร์ -> สนาม
        assert!(rows.iter().any(|r| r.discipline == "วิทยาศาสตร์" && r.thai == "สนาม"));
        // computer -> เขตข้อมูล and สนาม (numbered sub-senses, prefix stripped)
        assert!(rows
            .iter()
            .any(|r| r.discipline == "คอมพิวเตอร์และเทคโนโลยีสารสนเทศ" && r.thai == "เขตข้อมูล"));
        assert!(rows
            .iter()
            .any(|r| r.discipline == "คอมพิวเตอร์และเทคโนโลยีสารสนเทศ" && r.thai == "สนาม"));
        // คณิตศาสตร์ -> ฟีลด์ (bracketed scope note stripped)
        assert!(rows.iter().any(|r| r.discipline == "คณิตศาสตร์" && r.thai == "ฟีลด์"));
        // The scope note must NOT be part of the term.
        assert!(!rows.iter().any(|r| r.thai.contains('[')));
        // The English echo must never become a term.
        assert!(!rows.iter().any(|r| r.thai == "field"));
        // หมวดย่อย label line must be skipped.
        assert!(!rows.iter().any(|r| r.thai.starts_with("หมวดย่อย")));
    }

    #[test]
    fn strip_sense_number_handles_thai_numerals() {
        assert_eq!(strip_sense_number("๑. เขตข้อมูล"), "เขตข้อมูล");
        assert_eq!(strip_sense_number("๒. สนาม"), "สนาม");
        assert_eq!(strip_sense_number("สนาม"), "สนาม");
    }

    #[test]
    fn strip_scope_note_removes_brackets() {
        assert_eq!(strip_scope_note("ฟีลด์ [ในพีชคณิตนามธรรม]").trim(), "ฟีลด์");
        assert_eq!(strip_scope_note("สนาม").trim(), "สนาม");
    }

    #[test]
    fn load_dir_builds_coinedword_entries_with_provenance() {
        // Write FIELD_HTML to a temp cache dir and load it.
        let dir = std::env::temp_dir().join("wacha_coined_test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("field.html"), FIELD_HTML).unwrap();
        let entries = CoinedWordImporter.load(&dir).unwrap();
        // สนาม must be an entry with a CoinedWord/OrstEducational/Confirmed sense.
        let sanam = entries.iter().find(|e| e.headword == "สนาม").expect("สนาม entry");
        assert!(sanam.senses.iter().any(|s| {
            s.provenance.source == Source::CoinedWord
                && s.provenance.license == License::OrstEducational
                && s.provenance.confidence == RelationConfidence::Confirmed
        }));
        // สนาม should link to เขตข้อมูล / ฟีลด์ (equivalents of the same English word).
        assert!(sanam
            .relations
            .iter()
            .any(|(r, t)| *r == Relation::Synonym && (t == "เขตข้อมูล" || t == "ฟีลด์")));
        std::fs::remove_dir_all(&dir).ok();
    }
}
