//! Loose rhyme finder (R11 Phase WRITE-2).
//!
//! **Scope (deliberately bounded):** this is a LOOSE rhyme matcher — same final
//! vowel + final-consonant class — NOT classical เอก/โท tone-and-meter matching
//! (that was flagged as a scope trap this round; we do not build it). Operates on
//! the plain Thai-script respelling already stored in `Entry.pronunciation`
//! (samples like `กอ`, `กะตุก`, `กะถินนะทาน` — verified against data/rid/).
//!
//! The rhyme KEY is the final syllable's rime: (final-consonant-class, vowel-shape)
//! of the LAST syllable-ish chunk. Two words share a key ⇒ they loosely rhyme.

use std::collections::BTreeMap;

/// Thai final-consonant sound classes (มาตราตัวสะกด). Many letters map to the
/// same spoken final; we group by spoken class so respelling variants rhyme.
fn final_class(c: char) -> Option<&'static str> {
    match c {
        'ก' | 'ข' | 'ค' | 'ฆ' => Some("แม่กก"),
        'ด' | 'ต' | 'ถ' | 'ท' | 'ธ' | 'จ' | 'ช' | 'ซ' | 'ฎ' | 'ฏ' | 'ฐ' | 'ฑ' | 'ฒ' | 'ส' | 'ศ' | 'ษ' => Some("แม่กด"),
        'บ' | 'ป' | 'พ' | 'ฟ' | 'ภ' => Some("แม่กบ"),
        'น' | 'ณ' | 'ร' | 'ล' | 'ฬ' | 'ญ' => Some("แม่กน"),
        'ง' => Some("แม่กง"),
        'ม' => Some("แม่กม"),
        'ย' => Some("แม่เกย"),
        'ว' => Some("แม่เกอว"),
        _ => None,
    }
}

/// Extract a loose rhyme key from a Thai respelling: the final rime = the trailing
/// vowel run + final-consonant class of the last syllable-ish chunk.
/// Returns None if the input is too short/empty to key meaningfully.
pub fn rhyme_key(pron: &str) -> Option<String> {
    let cleaned = pron.trim().trim_start_matches('[').trim_end_matches(']');
    let first = cleaned.split(',').next().unwrap_or(cleaned).trim().trim_end_matches('-');
    // Last syllable-ish chunk (respellings sometimes hyphenate; else whole word).
    let chunk = first.split(|c| c == '-' || c == ' ').filter(|s| !s.is_empty()).last().unwrap_or(first);
    let chars: Vec<char> = chunk.chars().filter(|c| !is_tone_mark(*c)).collect();
    if chars.len() < 2 {
        return None;
    }
    // Final consonant (last char if it's a consonant with a spoken final class).
    let last = *chars.last().unwrap();
    let (final_key, vowel_region): (&str, &[char]) = match final_class(last) {
        Some(cls) => (cls, &chars[..chars.len() - 1]),
        None => ("แม่ ก กา", &chars[..]), // no final consonant → live open syllable
    };
    // Vowel shape of the rime: long if the vowel region carries า/ี/ู/ื/ๅ/อ/โ/ใ/ไ/เ.
    let vr: String = vowel_region.iter().collect();
    let long = vr.contains('า') || vr.contains('ี') || vr.contains('ู') || vr.contains('ื')
        || vr.contains('อ') || vr.contains('โ') || vr.contains('ื');
    // The last vowel character present (approximate nucleus) for a tighter key.
    let nucleus = vowel_region.iter().rev().find(|c| is_vowel(**c)).copied().unwrap_or('อ');
    Some(format!("{}|{}|{}", final_key, nucleus, if long { "L" } else { "S" }))
}

fn is_tone_mark(c: char) -> bool {
    matches!(c, '\u{0E48}'..='\u{0E4B}') // ่ ้ ๊ ๋
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'ะ' | 'ั' | 'า' | 'ำ' | 'ิ' | 'ี' | 'ึ' | 'ื' | 'ุ' | 'ู' | 'เ' | 'แ' | 'โ' | 'ใ' | 'ไ' | 'อ' | 'ๅ')
}

/// A rhyme index: rhyme-key → headwords sharing it.
#[derive(Debug, Default, Clone)]
pub struct RhymeIndex {
    by_key: BTreeMap<String, Vec<String>>,
    word_key: BTreeMap<String, String>,
}

impl RhymeIndex {
    /// Build from (headword, pronunciation) pairs. Words with no usable key are skipped.
    pub fn build<I: IntoIterator<Item = (String, String)>>(pairs: I) -> Self {
        let mut idx = RhymeIndex::default();
        for (hw, pron) in pairs {
            let basis = if pron.trim().is_empty() { hw.clone() } else { pron };
            if let Some(k) = rhyme_key(&basis) {
                let bucket = idx.by_key.entry(k.clone()).or_default();
                if !bucket.contains(&hw) {
                    bucket.push(hw.clone());
                }
                idx.word_key.insert(hw, k);
            }
        }
        idx
    }

    pub fn key_count(&self) -> usize {
        self.by_key.len()
    }
    pub fn word_count(&self) -> usize {
        self.word_key.len()
    }

    /// Words that loosely rhyme with `word` (share its rhyme key), excluding itself.
    pub fn rhymes_of(&self, word: &str) -> Vec<String> {
        let Some(k) = self.word_key.get(word) else { return Vec::new() };
        self.by_key
            .get(k)
            .map(|v| v.iter().filter(|w| *w != word).cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_rime_shares_a_key() {
        // กด / บด (both แม่กด, short อ) should share a key and thus rhyme.
        assert_eq!(rhyme_key("กด"), rhyme_key("บด"));
        // กอ / ขอ (open, long อ) share a key.
        assert_eq!(rhyme_key("กอ"), rhyme_key("ขอ"));
    }

    #[test]
    fn different_final_class_differs() {
        // กด (แม่กด) vs กง (แม่กง) must NOT share a key.
        assert_ne!(rhyme_key("กด"), rhyme_key("กง"));
    }

    #[test]
    fn index_returns_rhymes_excluding_self() {
        let idx = RhymeIndex::build(vec![
            ("กด".into(), "กด".into()),
            ("บด".into(), "บด".into()),
            ("กง".into(), "กง".into()),
        ]);
        let r = idx.rhymes_of("กด");
        assert!(r.contains(&"บด".to_string()));
        assert!(!r.contains(&"กด".to_string())); // excludes itself
        assert!(!r.contains(&"กง".to_string())); // different final class
    }

    #[test]
    fn too_short_has_no_key() {
        assert!(rhyme_key("").is_none());
    }
}
