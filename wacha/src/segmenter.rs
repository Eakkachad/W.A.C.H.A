//! Thai word segmenter: the vendored `Datrie` double-array trie (`crate::datrie`,
//! originally from `katgpt-tokenizer`) drives a
//! greedy longest-match over a compiled word list. Where no dictionary word
//! matches, it falls back to whole Thai Character Clusters (see [`crate::tcc`])
//! instead of raw codepoints — the Day-0 bug fix.
//!
//! This is the "katgpt-rs's tokenizer reads the dictionary" half of the hybrid:
//! the dictionary's own word list *is* the segmentation engine.

use crate::datrie::DatrieVocab;
use crate::tcc;
use std::collections::HashMap;

/// A segmenter built from a Thai word list.
///
/// The built `DatrieVocab` can be cached to disk (see [`Segmenter::save_cache`]
/// / [`Segmenter::load_cache`]): building the trie from the full 62k-word list
/// takes ~43s (Thai's narrow UTF-8 byte range causes heavy double-array
/// collision cascades — see `PROGRESS.md` 2026-09-05), but a cached load is
/// milliseconds. The cost is one-time *construction*, not per-query.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Segmenter {
    vocab: DatrieVocab,
    word_count: usize,
}

/// One output token from segmentation, tagged with whether it was a known
/// dictionary word or an out-of-vocabulary character-cluster fallback. The demo
/// uses this to visually distinguish "the dictionary recognized this" from "this
/// is unknown text kept intact as clusters."
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub in_vocab: bool,
}

impl Segmenter {
    /// Build a segmenter from an iterator of words (e.g. lines of `words_th.txt`).
    /// Empty and whitespace-only entries are skipped.
    pub fn from_words<I, S>(words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut vocab_map: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut idx = 0usize;
        for w in words {
            let w = w.as_ref().trim();
            if w.is_empty() {
                continue;
            }
            // Keep the first occurrence's id; skip exact duplicates.
            vocab_map.entry(w.as_bytes().to_vec()).or_insert_with(|| {
                let i = idx;
                idx += 1;
                i
            });
        }
        let word_count = vocab_map.len();
        let vocab = DatrieVocab::build(&vocab_map);
        Self { vocab, word_count }
    }

    /// Serialize the built segmenter (the double-array trie + word count) to a
    /// compact binary blob via `postcard`. Write this to disk to skip the ~43s
    /// rebuild on the next run.
    pub fn to_cache_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }

    /// Reconstruct a segmenter from bytes produced by [`Segmenter::to_cache_bytes`].
    /// Milliseconds instead of the ~43s full build.
    pub fn from_cache_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }

    /// Save the built segmenter to `path` (creates/overwrites the file).
    pub fn save_cache(&self, path: &std::path::Path) -> std::io::Result<()> {
        let bytes = self
            .to_cache_bytes()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, bytes)
    }

    /// Load a segmenter from a cache file at `path`.
    pub fn load_cache(path: &std::path::Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        Self::from_cache_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Number of distinct words compiled into the trie.
    pub fn word_count(&self) -> usize {
        self.word_count
    }

    /// Is `word` an exact entry in the word list?
    pub fn contains(&self, word: &str) -> bool {
        // longest_prefix from position 0 that consumes the whole word means it's
        // a full entry (a prefix-only match would end short).
        let bytes = word.as_bytes();
        match self.vocab.longest_prefix(bytes, 0) {
            Some((_s, end)) => end == bytes.len(),
            None => false,
        }
    }

    /// Segment `text` into tokens (greedy longest match, TCC-aware OOV fallback).
    pub fn segment(&self, text: &str) -> Vec<Token> {
        let bytes = text.as_bytes();
        let mut pos = 0;
        let mut out = Vec::new();
        while pos < bytes.len() {
            if let Some((_start, end)) = self.vocab.longest_prefix(bytes, pos) {
                if end > pos {
                    out.push(Token {
                        text: String::from_utf8_lossy(&bytes[pos..end]).into_owned(),
                        in_vocab: true,
                    });
                    pos = end;
                    continue;
                }
            }
            // OOV fallback: emit exactly one *Thai Character Cluster*, never a
            // bare codepoint. This is the bug fix: previously we sliced a single
            // UTF-8 codepoint here, which orphaned tone marks / leading vowels.
            let end = self.next_cluster_end(bytes, pos);
            out.push(Token {
                text: String::from_utf8_lossy(&bytes[pos..end]).into_owned(),
                in_vocab: false,
            });
            pos = end;
        }
        out
    }

    /// Convenience: segmented surface strings only.
    pub fn segment_words(&self, text: &str) -> Vec<String> {
        self.segment(text).into_iter().map(|t| t.text).collect()
    }

    /// Byte offset of the end of the TCC cluster starting at `pos`. Guarantees
    /// progress (always `> pos`) and a valid UTF-8 boundary.
    fn next_cluster_end(&self, bytes: &[u8], pos: usize) -> usize {
        // Decode from `pos` to a UTF-8 string, cluster it, take the first
        // cluster's byte length. Cheap because the tail after the first cluster
        // is not materialized further here.
        let tail = &bytes[pos..];
        let tail_str = String::from_utf8_lossy(tail);
        let clusters = tcc::tcc_clusters(&tail_str);
        if let Some(first) = clusters.first() {
            // first.len() is a byte length within tail_str; since from_utf8_lossy
            // may substitute replacement chars for invalid bytes, clamp to a real
            // char boundary in the original tail to stay safe.
            let want = first.len().max(1);
            let mut end = pos + want;
            if end > bytes.len() {
                end = bytes.len();
            }
            // Snap to a UTF-8 boundary if lossy substitution shifted the length.
            while end < bytes.len() && (bytes[end] & 0b1100_0000) == 0b1000_0000 {
                end += 1;
            }
            end.max(pos + 1)
        } else {
            // Degenerate: advance one codepoint.
            let mut end = pos + 1;
            while end < bytes.len() && (bytes[end] & 0b1100_0000) == 0b1000_0000 {
                end += 1;
            }
            end
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_segmenter() -> Segmenter {
        Segmenter::from_words([
            "นักเรียน", "อ่าน", "หนังสือ", "ที่", "โรงเรียน", "แมว", "กิน", "ปลา",
            "ใหญ่", "ชอบ", "บ้าน", "ครู", "สอน", "ภาษา", "ไทย", "รัก", "สุนัข",
        ])
    }

    #[test]
    fn segments_known_sentence_exactly() {
        let seg = tiny_segmenter();
        let words = seg.segment_words("นักเรียนอ่านหนังสือที่โรงเรียน");
        assert_eq!(words, vec!["นักเรียน", "อ่าน", "หนังสือ", "ที่", "โรงเรียน"]);
    }

    #[test]
    fn oov_fallback_keeps_clusters_intact_not_codepoints() {
        let seg = tiny_segmenter();
        // "เด็ก" and "น้อย" are NOT in the tiny vocab -> OOV path exercised.
        let toks = seg.segment("เด็กน้อยรักสุนัข");
        // The bug was: OOV produced เ | ด | ็ | ก | น | ้ | อ | ย (orphaned marks).
        // Assert no token is a lone combining mark or lone leading vowel.
        for t in &toks {
            assert_ne!(t.text, "็", "orphaned tone mark: the bug is back");
            assert_ne!(t.text, "้", "orphaned tone mark: the bug is back");
            assert_ne!(t.text, "เ", "orphaned leading vowel: the bug is back");
        }
        // Known words at the end are still recognized as in-vocab.
        let rak = toks.iter().find(|t| t.text == "รัก").unwrap();
        assert!(rak.in_vocab);
        let sunak = toks.iter().find(|t| t.text == "สุนัข").unwrap();
        assert!(sunak.in_vocab);
        // Round-trips exactly.
        let joined: String = toks.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(joined, "เด็กน้อยรักสุนัข");
    }

    #[test]
    fn oov_cluster_contains_bound_mark() {
        let seg = tiny_segmenter();
        let toks = seg.segment("เด็ก");
        // "เด็" should appear as a single 3-codepoint cluster (leading vowel +
        // consonant + tone mark), proving the fix.
        assert!(toks.iter().any(|t| t.text == "เด็"));
    }

    #[test]
    fn contains_reports_membership() {
        let seg = tiny_segmenter();
        assert!(seg.contains("แมว"));
        assert!(!seg.contains("ไดโนเสาร์"));
    }

    #[test]
    fn cache_roundtrip_preserves_segmentation() {
        let seg = tiny_segmenter();
        let bytes = seg.to_cache_bytes().expect("serialize");
        let restored = Segmenter::from_cache_bytes(&bytes).expect("deserialize");
        // Same word count and identical segmentation after a round-trip.
        assert_eq!(restored.word_count(), seg.word_count());
        let a = seg.segment_words("นักเรียนอ่านหนังสือที่โรงเรียน");
        let b = restored.segment_words("นักเรียนอ่านหนังสือที่โรงเรียน");
        assert_eq!(a, b);
        assert!(restored.contains("แมว"));
    }
}
