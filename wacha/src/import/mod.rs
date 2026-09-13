//! Multi-source ingestion (Round 5, Task 2).
//!
//! Every data source (LEXiTRON, Kaikki, ศัพท์บัญญัติ, and the real RID data at
//! the event) is loaded through one [`Importer`] trait into the shared
//! RID-shaped [`Entry`] model, then unified by [`merge`]. This is the layer that
//! makes competition-day ingestion a config change instead of a rewrite: drop
//! the organizer's file in, point an importer at it, merge.
//!
//! Dependency-light by design: errors are `Box<dyn Error>` (no `anyhow`), file
//! reading is `std::fs` / streaming.

use crate::dictionary::{Entry, Source};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

pub mod kaikki;

/// A boxed, thread-safe error (dependency-free stand-in for `anyhow::Error`).
pub type ImportError = Box<dyn Error + Send + Sync>;
pub type ImportResult<T> = Result<T, ImportError>;

/// A data source that yields RID-shaped [`Entry`]s.
pub trait Importer {
    /// Human-readable source name (for logs).
    fn name(&self) -> &'static str;
    /// The [`Source`] tag stamped on this importer's entries (also drives merge
    /// priority and licence).
    fn source(&self) -> Source;
    /// Load entries from `path` (a file or directory, importer's choice).
    fn load(&self, path: &Path) -> ImportResult<Vec<Entry>>;
}

/// LEXiTRON word list (`words_th.txt`): one headword per line, no senses.
/// These are the segmentation vocabulary; they carry no definition until a
/// richer source (Kaikki/RID) merges senses into them.
pub struct LexitronImporter;

impl Importer for LexitronImporter {
    fn name(&self) -> &'static str {
        "lexitron"
    }
    fn source(&self) -> Source {
        Source::Lexitron
    }
    fn load(&self, path: &Path) -> ImportResult<Vec<Entry>> {
        let text = std::fs::read_to_string(path)?;
        let mut out = Vec::new();
        for line in text.lines() {
            let w = line.trim();
            if !w.is_empty() {
                out.push(Entry::headword_only(w));
            }
        }
        Ok(out)
    }
}

/// The 20 hand-curated seed entries (in-memory, no file).
pub struct SeedImporter;

impl Importer for SeedImporter {
    fn name(&self) -> &'static str {
        "seed"
    }
    fn source(&self) -> Source {
        Source::HumanSeed
    }
    fn load(&self, _path: &Path) -> ImportResult<Vec<Entry>> {
        Ok(crate::dictionary::seed_entries())
    }
}

/// Merge entries from several sources into a unified list, keyed by
/// `(headword, homograph)`.
///
/// Rules (Task 2):
/// - **Senses concatenate**, never overwrite (a word can hold LEXiTRON + Kaikki
///   + RID senses side by side, each with its own provenance).
/// - **Scalar fields** (`pronunciation`, `romanization`) fill only if currently
///   `None`, taking the highest-priority source that provides one.
/// - **Relations / sub_entries / see_also** are unioned (dedup, order-stable).
/// - Source priority: **Rid > HumanSeed > CoinedWord > Kaikki > Lexitron**.
///
/// **Deterministic:** input source-lists are processed in descending priority
/// order, and within each merged entry the senses are sorted by
/// `(source_priority DESC, pos marker, subject tag)`. Two runs over the same
/// inputs produce byte-identical output.
pub fn merge(mut sources: Vec<Vec<Entry>>) -> Vec<Entry> {
    // Tag each source-list with the priority of its entries. An importer's
    // entries all share one source, but to be safe we read each entry's own
    // top sense source (falling back to a per-list max) — simplest correct
    // approach: sort the *lists* by the max priority of any entry they contain,
    // descending, so higher-priority sources are seen first.
    sources.sort_by(|a, b| list_priority(b).cmp(&list_priority(a)));

    // (headword, homograph) -> merged Entry, plus insertion order for stability.
    let mut merged: BTreeMap<(String, Option<u8>), Entry> = BTreeMap::new();
    let mut order: Vec<(String, Option<u8>)> = Vec::new();

    for list in sources {
        for entry in list {
            let key = (entry.headword.clone(), entry.homograph);
            match merged.get_mut(&key) {
                None => {
                    order.push(key.clone());
                    merged.insert(key, entry);
                }
                Some(existing) => merge_into(existing, entry),
            }
        }
    }

    // Emit in first-seen order (which is priority-desc, then within-list order),
    // with senses sorted deterministically inside each entry.
    let mut out: Vec<Entry> = order
        .into_iter()
        .filter_map(|k| merged.remove(&k))
        .collect();
    for e in &mut out {
        sort_senses(e);
    }
    out
}

/// Priority of a source-list = max sense-source priority found in it (an
/// importer's entries all share one source, but headword-only entries have no
/// sense — fall back to a small floor so LEXiTRON sorts last).
fn list_priority(list: &[Entry]) -> u8 {
    list.iter()
        .flat_map(|e| e.senses.iter())
        .map(|s| s.provenance.source.priority())
        .max()
        .unwrap_or(0)
}

/// Fold `incoming` into `existing` (same headword+homograph).
fn merge_into(existing: &mut Entry, incoming: Entry) {
    // Senses always concatenate.
    existing.senses.extend(incoming.senses);
    // Scalars fill only if empty.
    if existing.pronunciation.is_none() {
        existing.pronunciation = incoming.pronunciation;
    }
    if existing.romanization.is_none() {
        existing.romanization = incoming.romanization;
    }
    // Unions (dedup, order-stable).
    dedup_extend_string(&mut existing.sub_entries, incoming.sub_entries);
    dedup_extend_string(&mut existing.see_also, incoming.see_also);
    for r in incoming.relations {
        if !existing.relations.contains(&r) {
            existing.relations.push(r);
        }
    }
    // Etymology: union by (lang, form).
    for et in incoming.etymology {
        if !existing.etymology.iter().any(|e| e.lang == et.lang && e.form == et.form) {
            existing.etymology.push(et);
        }
    }
}

/// Order an entry's senses deterministically: highest-priority source first,
/// and — crucially — **preserve each source's own sense order within a tier**
/// (a *stable* sort, no alphabetical tiebreak). Wiktionary/RID list their senses
/// most-important-first; alphabetizing by definition text (as an earlier version
/// did) buried the canonical sense (e.g. `บ้าน`'s "ที่อยู่อาศัย" behind
/// "ถิ่นที่มีมนุษย์อยู่"). Input order is deterministic (sources merged in
/// priority order), so a stable sort is deterministic too.
fn sort_senses(e: &mut Entry) {
    e.senses.sort_by(|a, b| {
        b.provenance
            .source
            .priority()
            .cmp(&a.provenance.source.priority())
    });
}

fn dedup_extend_string(dst: &mut Vec<String>, src: Vec<String>) {
    for s in src {
        if !dst.contains(&s) {
            dst.push(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::{Pos, Provenance, RelationConfidence, License, Sense};

    fn seed_sense(def: &str) -> Sense {
        Sense::simple(Pos::Nam, def)
    }
    fn kaikki_sense(def: &str) -> Sense {
        Sense {
            pos: Some(Pos::Nam),
            subject: None,
            register: None,
            definition: def.to_string(),
            examples: vec![],
            classifiers: vec![],
            provenance: Provenance {
                source: Source::Kaikki,
                license: License::CcBySa,
                confidence: RelationConfidence::Unverified,
            },
        }
    }

    #[test]
    fn lexitron_loads_headwords_only() {
        // write a tiny temp file
        let dir = std::env::temp_dir();
        let p = dir.join("wacha_lexitron_test.txt");
        std::fs::write(&p, "แมว\nสุนัข\n\n  หมา  \n").unwrap();
        let entries = LexitronImporter.load(&p).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].headword, "แมว");
        assert!(entries[0].senses.is_empty());
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn merge_lexitron_headword_with_seed_yields_one_entry_with_senses() {
        let lex = vec![Entry::headword_only("ครู")];
        let seed = vec![{
            let mut e = Entry::headword_only("ครู");
            e.senses.push(seed_sense("ผู้สั่งสอนศิษย์"));
            e
        }];
        let out = merge(vec![lex, seed]);
        let kru: Vec<_> = out.iter().filter(|e| e.headword == "ครู").collect();
        assert_eq!(kru.len(), 1, "must unify into one entry");
        assert_eq!(kru[0].senses.len(), 1, "seed sense present, no duplicate empty");
    }

    #[test]
    fn senses_concatenate_across_sources() {
        let seed = vec![{
            let mut e = Entry::headword_only("บ้าน");
            e.senses.push(seed_sense("ที่อยู่อาศัย"));
            e
        }];
        let kaikki = vec![{
            let mut e = Entry::headword_only("บ้าน");
            e.senses.push(kaikki_sense("บ้านเรือน (Kaikki)"));
            e
        }];
        let out = merge(vec![kaikki, seed]);
        let ban = out.iter().find(|e| e.headword == "บ้าน").unwrap();
        assert_eq!(ban.senses.len(), 2, "both sources' senses kept");
        // HumanSeed (priority 4) sorts before Kaikki (priority 2).
        assert_eq!(ban.senses[0].provenance.source, Source::HumanSeed);
        assert_eq!(ban.senses[1].provenance.source, Source::Kaikki);
    }

    #[test]
    fn merge_is_deterministic_across_runs() {
        let mk = || {
            let seed = vec![{
                let mut e = Entry::headword_only("คำ");
                e.senses.push(seed_sense("เสียงพูดที่มีความหมาย"));
                e
            }];
            let kaikki = vec![
                {
                    let mut e = Entry::headword_only("คำ");
                    e.senses.push(kaikki_sense("word (Kaikki)"));
                    e
                },
                Entry::headword_only("ใหม่"),
            ];
            let lex = vec![Entry::headword_only("คำ"), Entry::headword_only("เก่า")];
            merge(vec![lex, kaikki, seed])
        };
        let a = mk();
        let b = mk();
        // Byte-identical: same order, same headwords, same sense order/defs.
        let fa: Vec<(String, Vec<String>)> = a
            .iter()
            .map(|e| (e.headword.clone(), e.senses.iter().map(|s| s.definition.clone()).collect()))
            .collect();
        let fb: Vec<(String, Vec<String>)> = b
            .iter()
            .map(|e| (e.headword.clone(), e.senses.iter().map(|s| s.definition.clone()).collect()))
            .collect();
        assert_eq!(fa, fb, "two merge runs must be byte-identical");
    }
}
