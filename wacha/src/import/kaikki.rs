//! Kaikki (Wiktionary) Thai importer (Round 5, Task 3).
//!
//! Takes definitions from 20 → 25,000+ words, and brings ลักษณนาม (classifiers),
//! Royal-Institute romanization, usage examples, and register tags with it.
//!
//! **Streams the file line by line** — the fallback all-languages dump is 1.6 GB
//! uncompressed, so never read it whole. Each line is one JSON object; a
//! malformed line is skipped, not fatal. Every sense is stamped
//! `Provenance { Kaikki, CcBySa, Unverified }` — a community source, not
//! presented at seed confidence.

use crate::dictionary::{
    Entry, Etymology, License, Pos, Provenance, Register, RelationConfidence, Sense, Source, Subject,
};
use crate::import::{ImportResult, Importer};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct KaikkiImporter;

impl Importer for KaikkiImporter {
    fn name(&self) -> &'static str {
        "kaikki"
    }
    fn source(&self) -> Source {
        Source::Kaikki
    }
    fn load(&self, path: &Path) -> ImportResult<Vec<Entry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // Skip malformed lines without panicking.
            let v: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(entry) = entry_from_value(&v) {
                out.push(entry);
            }
        }
        Ok(out)
    }
}

/// Map a Kaikki `pos` string to our closed `Pos` set. Returns `None` for kinds
/// with no RID equivalent (name/classifier/phrase/…); the sense is still kept.
fn map_pos(pos: &str) -> Option<Pos> {
    match pos {
        "noun" | "name" | "adj_noun" => Some(Pos::Nam),
        "verb" => Some(Pos::Kri),
        "adj" | "adv" => Some(Pos::Wiset),
        "pron" => Some(Pos::Sapphanam),
        "prep" => Some(Pos::Buraphabot),
        "conj" => Some(Pos::Santhan),
        "intj" => Some(Pos::Uthan),
        "particle" => Some(Pos::Nibat),
        _ => None, // classifier, num, character, phrase, proverb, punct, prefix, abbrev, unknown
    }
}

/// Map Kaikki register-ish tags to RID [`Register`].
fn map_register(tags: &[String]) -> Option<Register> {
    for t in tags {
        let r = match t.as_str() {
            "colloquial" | "informal" | "slang" => Some(Register::Pak),
            "dated" | "archaic" => Some(Register::Bo),
            "obsolete" => Some(Register::Loek),
            "polite" | "honorific" | "royal" => Some(Register::Racha),
            "literary" | "formal" => Some(Register::Baep),
            _ => None,
        };
        if r.is_some() {
            return r;
        }
    }
    None
}

/// Map a Kaikki topic to an RID [`Subject`]; unmapped topics become
/// `Subject::Other(topic)` so nothing is silently dropped.
fn map_subject(topics: &[String]) -> Option<Subject> {
    let first = topics.first()?;
    Some(match first.as_str() {
        "computing" | "computer" | "internet" => Subject::Khom,
        "mathematics" | "math" | "arithmetic" | "algebra" | "geometry" => Subject::Khanit,
        "grammar" | "linguistics" | "grammatical" => Subject::Wai,
        "physics" => Subject::Fisik,
        "chemistry" => Subject::Khemi,
        "biology" => Subject::Chiwa,
        "medicine" | "medical" | "anatomy" => Subject::Phaet,
        "law" | "legal" => Subject::Kot,
        "botany" | "plant" => Subject::Phrueksa,
        "zoology" | "animal" => Subject::Sat,
        "astronomy" => Subject::Dara,
        "geography" | "geology" => Subject::Phumi,
        "economics" | "finance" => Subject::Settha,
        "religion" | "Buddhism" => Subject::Satsana,
        "politics" => Subject::KanMueang,
        "military" => Subject::Other("ทหาร".to_string()),
        "music" => Subject::Other("ดนตรี".to_string()),
        other => Subject::Other(other.to_string()),
    })
}

fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string())
}

fn string_list(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|arr| arr.iter().filter_map(|e| e.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

/// Royal-Institute romanization from `sounds[]` (tags contains "Royal-Institute").
fn royal_institute_roman(v: &Value) -> Option<String> {
    let sounds = v.get("sounds")?.as_array()?;
    for s in sounds {
        let tags = string_list(s, "tags");
        if tags.iter().any(|t| t == "Royal-Institute") {
            if let Some(r) = str_field(s, "roman") {
                return Some(r);
            }
        }
    }
    None
}

fn entry_from_value(v: &Value) -> Option<Entry> {
    let word = str_field(v, "word")?;
    if word.trim().is_empty() {
        return None;
    }
    let pos_str = str_field(v, "pos").unwrap_or_default();
    let pos = map_pos(&pos_str);
    let is_proper_name = pos_str == "name";

    let romanization = royal_institute_roman(v);
    // etymology_texts is a LIST (never a singular etymology_text field).
    let etymology: Vec<Etymology> = string_list(v, "etymology_texts")
        .into_iter()
        .map(|t| Etymology { lang: String::new(), form: t })
        .collect();

    let prov = Provenance {
        source: Source::Kaikki,
        license: License::CcBySa,
        confidence: RelationConfidence::Unverified,
    };

    let mut senses = Vec::new();
    if let Some(arr) = v.get("senses").and_then(|s| s.as_array()) {
        for s in arr {
            // definition = first gloss
            let glosses = string_list(s, "glosses");
            let Some(def) = glosses.into_iter().next() else {
                continue;
            };
            if def.trim().is_empty() {
                continue;
            }
            let examples: Vec<String> = s
                .get("examples")
                .and_then(|e| e.as_array())
                .map(|arr| arr.iter().filter_map(|e| str_field(e, "text")).collect())
                .unwrap_or_default();
            let classifiers: Vec<String> = s
                .get("classifiers")
                .and_then(|c| c.as_array())
                .map(|arr| arr.iter().filter_map(|c| str_field(c, "classifier")).collect())
                .unwrap_or_default();
            let tags = string_list(s, "tags");
            let topics = string_list(s, "topics");
            senses.push(Sense {
                pos,
                subject: map_subject(&topics),
                register: map_register(&tags),
                definition: def,
                examples,
                classifiers,
                provenance: prov.clone(),
            });
        }
    }

    // A proper-name entry with no usable sense still gets a marker sense so the
    // UI can distinguish it; a common word with no sense is dropped (adds no
    // definition value).
    if senses.is_empty() {
        if is_proper_name {
            senses.push(Sense {
                pos,
                subject: Some(Subject::Other("วิสามานยนาม".to_string())), // proper name
                register: None,
                definition: format!("วิสามานยนาม (proper name): {word}"),
                examples: vec![],
                classifiers: vec![],
                provenance: prov,
            });
        } else {
            return None;
        }
    }

    let mut e = Entry::headword_only(&word);
    e.romanization = romanization;
    e.etymology = etymology;
    e.senses = senses;
    // synonyms/derived/related -> relations (Synonym/RelatedTo) for the graph.
    for syn in kaikki_relation_words(v, "synonyms") {
        e.relations.push((crate::dictionary::Relation::Synonym, syn));
    }
    for rel in kaikki_relation_words(v, "related") {
        e.relations.push((crate::dictionary::Relation::RelatedTo, rel));
    }
    Some(e)
}

/// Pull `word` strings out of a top-level relation array (synonyms/derived/related).
fn kaikki_relation_words(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| str_field(e, "word"))
                .filter(|w| !w.trim().is_empty())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_line_is_skipped_not_panicked() {
        // Write a file with one good + one broken line; loader must skip the bad.
        let dir = std::env::temp_dir();
        let p = dir.join("wacha_kaikki_test.jsonl");
        let good = r#"{"word":"แมว","lang_code":"th","pos":"noun","senses":[{"glosses":["สัตว์เลี้ยงชนิดหนึ่ง"]}]}"#;
        std::fs::write(&p, format!("{good}\n{{ this is not json\n\n")).unwrap();
        let entries = KaikkiImporter.load(&p).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].headword, "แมว");
        assert_eq!(entries[0].primary_definition(), Some("สัตว์เลี้ยงชนิดหนึ่ง"));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn royal_institute_romanization_extracted() {
        let v: Value = serde_json::from_str(
            r#"{"word":"พจนานุกรม","pos":"noun",
                "sounds":[{"tags":["romanization","Royal-Institute"],"roman":"phot-cha-na-nu-krom"},
                          {"ipa":"/pʰót.t͡ɕʰā/"}],
                "senses":[{"glosses":["dictionary"]}]}"#,
        )
        .unwrap();
        let e = entry_from_value(&v).unwrap();
        assert_eq!(e.romanization.as_deref(), Some("phot-cha-na-nu-krom"));
    }

    #[test]
    fn etymology_texts_is_a_list() {
        // Must read etymology_texts (list); a singular etymology_text does NOT exist.
        let v: Value = serde_json::from_str(
            r#"{"word":"พจนานุกรม","pos":"noun",
                "etymology_texts":["พจน + อนุกรม","(borrowed)"],
                "senses":[{"glosses":["dictionary"]}]}"#,
        )
        .unwrap();
        let e = entry_from_value(&v).unwrap();
        assert_eq!(e.etymology.len(), 2);
        assert_eq!(e.etymology[0].form, "พจน + อนุกรม");
    }

    #[test]
    fn classifiers_and_kaikki_provenance() {
        let v: Value = serde_json::from_str(
            r#"{"word":"บ้าน","pos":"noun",
                "senses":[{"glosses":["ที่อยู่อาศัย"],"classifiers":[{"classifier":"หลัง"}]}]}"#,
        )
        .unwrap();
        let e = entry_from_value(&v).unwrap();
        assert_eq!(e.senses[0].classifiers, vec!["หลัง".to_string()]);
        assert_eq!(e.senses[0].provenance.source, Source::Kaikki);
        assert_eq!(e.senses[0].provenance.license, License::CcBySa);
        assert_eq!(e.senses[0].provenance.confidence, RelationConfidence::Unverified);
    }

    #[test]
    fn common_word_without_sense_is_dropped() {
        let v: Value =
            serde_json::from_str(r#"{"word":"เฉย","pos":"noun","senses":[]}"#).unwrap();
        assert!(entry_from_value(&v).is_none());
    }
}
