//! Thai Character Cluster (TCC) grouping.
//!
//! Fixes the Day-0 POC bug: the out-of-vocabulary fallback used to split unknown
//! Thai text into raw Unicode codepoints, which shatters Thai Character Clusters
//! (a consonant and its tone mark / dependent vowel get separated). Thai
//! orthography binds certain marks to a base consonant; those must never be
//! split. This module groups a string into TCCs so the segmenter's OOV fallback
//! emits whole clusters instead of loose combining marks.
//!
//! This is a faithful *rule-based* implementation of the essential Thai
//! combining behavior (Theeramunkong et al. 2000 TCC rules), not a port of the
//! full PyThaiNLP regex. It is deterministic and dependency-free.
//!
//! The three essential rules, by Thai Unicode block (U+0E00–U+0E7F):
//! 1. A *leading* vowel (เ แ โ ใ ไ, U+0E40–U+0E44) binds to the *following*
//!    consonant cluster — it is written before but pronounced after.
//! 2. *Following* combining marks — above/below dependent vowels, tone marks,
//!    phinthu, and thanthakhat (U+0E31, U+0E34–U+0E3A, U+0E47–U+0E4E) — bind to
//!    the *preceding* base consonant.
//! 3. SARA AM (ำ, U+0E33) binds to the preceding base.

/// Is `c` a Thai consonant (ก U+0E01 .. ฮ U+0E2E)?
fn is_thai_consonant(c: char) -> bool {
    ('\u{0E01}'..='\u{0E2E}').contains(&c)
}

/// Is `c` a Thai leading (pre-posed) vowel: เ แ โ ใ ไ (U+0E40..=U+0E44)?
fn is_leading_vowel(c: char) -> bool {
    ('\u{0E40}'..='\u{0E44}').contains(&c)
}

/// Is `c` a combining mark that binds to the *preceding* base consonant?
/// Covers MAI HAN-AKAT (U+0E31), the above/below dependent vowels
/// (U+0E34..=U+0E3A), SARA AM (U+0E33), and the tone marks / phinthu /
/// thanthakhat / nikhahit (U+0E47..=U+0E4E).
fn is_following_mark(c: char) -> bool {
    matches!(c,
        '\u{0E31}'
        | '\u{0E33}'
        | '\u{0E34}'..='\u{0E3A}'
        | '\u{0E47}'..='\u{0E4E}'
    )
}

/// Split `text` into Thai Character Clusters.
///
/// Non-Thai characters (spaces, Latin, digits, punctuation) are emitted as
/// single-character clusters — this module's job is only to keep Thai clusters
/// intact, not to tokenize other scripts.
pub fn tcc_clusters(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let mut cluster = String::new();
        let c = chars[i];

        if is_leading_vowel(c) {
            // Rule 1: leading vowel + following consonant (+ its bound marks).
            cluster.push(c);
            i += 1;
            if i < chars.len() && is_thai_consonant(chars[i]) {
                cluster.push(chars[i]);
                i += 1;
                // Absorb any marks bound to that consonant.
                while i < chars.len() && is_following_mark(chars[i]) {
                    cluster.push(chars[i]);
                    i += 1;
                }
            }
        } else if is_thai_consonant(c) {
            // Rule 2/3: consonant + any following bound marks.
            cluster.push(c);
            i += 1;
            while i < chars.len() && is_following_mark(chars[i]) {
                cluster.push(chars[i]);
                i += 1;
            }
        } else {
            // Non-Thai or an orphan mark: emit it alone so we always progress.
            cluster.push(c);
            i += 1;
        }

        out.push(cluster);
    }
    out
}

/// Byte offsets (into `text`) at which a TCC cluster *starts*, always including 0
/// and never including `text.len()`. Used by the segmenter to know where it may
/// legally place a word boundary during OOV fallback: boundaries mid-cluster are
/// illegal.
pub fn tcc_boundary_offsets(text: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut byte_pos = 0;
    for cluster in tcc_clusters(text) {
        offsets.push(byte_pos);
        byte_pos += cluster.len();
    }
    offsets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_consonant_with_tone_mark_together() {
        // น + ้ (MAI THO) must stay one cluster, not two.
        let clusters = tcc_clusters("น้");
        assert_eq!(clusters, vec!["น้".to_string()]);
    }

    #[test]
    fn leading_vowel_binds_to_following_consonant() {
        // เ + ด must be one cluster ("เด"), the classic bug case.
        let clusters = tcc_clusters("เด");
        assert_eq!(clusters, vec!["เด".to_string()]);
    }

    #[test]
    fn the_bug_case_dek_noi() {
        // "เด็กน้อย" previously shattered into เ|ด|็|ก|น|้|อ|ย (8 pieces, marks
        // orphaned). Correct TCC grouping keeps เด็ | ก | น้ | อ | ย together as
        // whole clusters — crucially no orphaned tone mark or leading vowel.
        let clusters = tcc_clusters("เด็กน้อย");
        // No cluster may be a lone combining mark or a lone leading vowel.
        for cl in &clusters {
            assert_ne!(cl, "็", "orphaned tone mark — the bug");
            assert_ne!(cl, "้", "orphaned tone mark — the bug");
            assert_ne!(cl, "เ", "orphaned leading vowel — the bug");
        }
        // Reconstructs exactly.
        assert_eq!(clusters.concat(), "เด็กน้อย");
        // เด็ is a single 3-codepoint cluster (leading vowel + consonant + mark).
        assert!(clusters.contains(&"เด็".to_string()));
        assert!(clusters.contains(&"น้".to_string()));
    }

    #[test]
    fn sara_am_binds_to_base() {
        // ทำ = ท + ำ (SARA AM) must be one cluster.
        let clusters = tcc_clusters("ทำ");
        assert_eq!(clusters, vec!["ทำ".to_string()]);
    }

    #[test]
    fn boundary_offsets_align_with_char_boundaries() {
        let text = "เด็กน้อย";
        for off in tcc_boundary_offsets(text) {
            assert!(text.is_char_boundary(off));
        }
    }

    #[test]
    fn non_thai_passthrough() {
        let clusters = tcc_clusters("ab ก");
        assert_eq!(clusters, vec!["a", "b", " ", "ก"]);
    }
}
