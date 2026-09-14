//! Sound-symbolism scorer (R11 Phase SOUND).
//!
//! A fully deterministic (no ML) heuristic that maps a Thai word's *sound* to a
//! hard↔soft character axis, in the project's "modelless, explainable" spirit.
//!
//! **Honesty (mandatory, mirrored in the UI):** cross-linguistic bouba/kiki-type
//! sound-symbolism effects are well-replicated in general phonetic research
//! (Styles & Gawne 2020; Lockwood & Dingemanse 2015), but Thai-specific evidence
//! is thin. This is "a pattern from general phonetic-symbolism research, applied
//! to Thai" — NOT proven Thai psycholinguistics. Every surface that shows the
//! badge must carry [`HEDGE`].
//!
//! Input is the plain Thai-script respelling already stored in `Entry`
//! (`pronunciation`, e.g. `[กอ]`, `[กะตุก]`) or, as a fallback, the headword
//! itself — real samples are Thai script, not IPA (verified against
//! `data/rid/dict_2554.txt`).

/// The mandatory UI hedge — must appear wherever the badge appears.
pub const HEDGE: &str =
    "รูปแบบจากงานวิจัยเรื่องสัทสัญลักษณ์ทั่วไป ยังไม่ใช่ข้อพิสูจน์เฉพาะภาษาไทย";

/// Plosive/stop consonants (hard).
const PLOSIVES: &[char] = &['ก', 'ต', 'ป', 'ข', 'ค', 'ฆ', 'ท', 'ธ', 'พ', 'ภ', 'ฏ', 'ฐ', 'ฑ', 'ฒ', 'ฎ', 'บ', 'ด', 'จ', 'ฉ', 'ช'];
/// Nasals / liquids / glides (soft).
const SONORANTS: &[char] = &['ง', 'น', 'ม', 'ย', 'ร', 'ล', 'ว', 'ญ', 'ณ', 'ฬ'];
/// Cluster-forming second members (กร, ปล, ตร, …).
const CLUSTER2: &[char] = &['ร', 'ล', 'ว'];
/// Back / rounded vowels (soft-leaning): โ- -ู -อ -ู -ุ.
const BACK_ROUND: &[char] = &['โ', 'อ', 'ุ', 'ู'];
/// Front / unrounded vowels (hard-leaning): อิ เ- แ- -ี.
const FRONT_UNROUND: &[char] = &['ิ', 'ี', 'เ', 'แ', 'ะ'];

/// The result: a signed score, a 1..=5 bucket, a Thai label, and the top
/// contributing features (the "why", same explain-everything discipline).
#[derive(Debug, Clone, PartialEq)]
pub struct SoundProfile {
    /// Raw average score per syllable (positive = harder, negative = softer).
    pub score: f32,
    /// 1 (นุ่ม/อ่อนโยน) .. 5 (แข็ง/หนักแน่น).
    pub bucket: u8,
    /// Thai label for the bucket.
    pub label: &'static str,
    /// Up to 2 contributing features, most-weighty first (the "why").
    pub why: Vec<String>,
}

fn bucket_label(bucket: u8) -> &'static str {
    match bucket {
        5 => "แข็ง/หนักแน่น",
        4 => "ค่อนข้างแข็ง",
        3 => "กลาง ๆ",
        2 => "ค่อนข้างนุ่ม",
        _ => "นุ่ม/อ่อนโยน",
    }
}

/// Score a Thai respelling (or headword). Splits on commas/brackets and averages
/// over the syllable-ish chunks it finds. Deterministic and allocation-light.
pub fn score(input: &str) -> SoundProfile {
    // Strip surrounding brackets; take the first pronunciation variant (before a comma).
    let cleaned = input.trim().trim_start_matches('[').trim_end_matches(']');
    let first = cleaned.split(',').next().unwrap_or(cleaned).trim();
    // Syllable-ish chunks: split on the respelling's hyphen/space separators.
    let chunks: Vec<&str> = first.split(|c| c == '-' || c == ' ' || c == '.').filter(|s| !s.is_empty()).collect();
    let chunks = if chunks.is_empty() { vec![first] } else { chunks };

    let mut total = 0f32;
    // Track feature tallies for the "why".
    let mut feat: Vec<(i32, &'static str)> = vec![
        (0, "พยัญชนะกัก (stop) — แข็ง"),
        (0, "เสียงนาสิก/ของเหลว (ง น ม ย ร ล ว) — นุ่ม"),
        (0, "สระสั้น — แข็ง"),
        (0, "สระยาว — นุ่ม"),
        (0, "ควบกล้ำ — แข็ง"),
        (0, "สระหลัง/มน (โ อ ู) — นุ่ม"),
        (0, "สระหน้า/แบน (อิ เ แ) — แข็ง"),
    ];

    for chunk in &chunks {
        let mut s = 0i32;
        let cs: Vec<char> = chunk.chars().collect();
        for (i, &c) in cs.iter().enumerate() {
            if PLOSIVES.contains(&c) {
                s += 2;
                feat[0].0 += 1;
            }
            if SONORANTS.contains(&c) {
                s -= 2;
                feat[1].0 += 1;
            }
            if BACK_ROUND.contains(&c) {
                s -= 1;
                feat[5].0 += 1;
            }
            if FRONT_UNROUND.contains(&c) {
                s += 1;
                feat[6].0 += 1;
            }
            // onset cluster: a plosive immediately followed by ร/ล/ว
            if i + 1 < cs.len() && PLOSIVES.contains(&c) && CLUSTER2.contains(&cs[i + 1]) {
                s += 1;
                feat[4].0 += 1;
            }
        }
        // Vowel length: a long vowel is spelled with า/ี/ู/ื/ๅ or explicit อา etc.
        let has_long = chunk.contains('า') || chunk.contains('ี') || chunk.contains('ื') || chunk.contains('ู');
        if has_long {
            s -= 1;
            feat[3].0 += 1;
        } else {
            s += 1;
            feat[2].0 += 1;
        }
        total += s as f32;
    }

    let avg = total / chunks.len() as f32;
    // Map avg (roughly -4..+6) to 1..=5 buckets.
    let bucket = if avg >= 4.0 {
        5
    } else if avg >= 2.0 {
        4
    } else if avg > -0.5 {
        3
    } else if avg > -2.5 {
        2
    } else {
        1
    };

    // "why": top 2 features by absolute tally.
    let mut nonzero: Vec<&(i32, &'static str)> = feat.iter().filter(|(n, _)| *n > 0).collect();
    nonzero.sort_by(|a, b| b.0.cmp(&a.0));
    let why: Vec<String> = nonzero.iter().take(2).map(|(_, name)| name.to_string()).collect();

    SoundProfile { score: avg, bucket, label: bucket_label(bucket), why }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_and_short_vowels_score_harder_than_nasals_and_long_vowels() {
        // Contrastive pair, direction intuitively unambiguous:
        //  hard: กะตุก (stops ก/ต/ก, short vowels) vs soft: มาลี (nasal/liquid ม/ล, long า/ี)
        let hard = score("กะตุก");
        let soft = score("มาลี");
        assert!(
            hard.bucket >= soft.bucket,
            "กะตุก ({}, b{}) should be >= มาลี ({}, b{})",
            hard.score, hard.bucket, soft.score, soft.bucket
        );
        // and they should not be identical — the heuristic must actually separate them
        assert!(hard.score > soft.score, "hard {} !> soft {}", hard.score, soft.score);
    }

    #[test]
    fn a_second_contrastive_pair() {
        // ตึก (stops, short) harder than งูใหญ่-ish นอน (nasals, long-ish)
        let hard = score("ตึก");
        let soft = score("นอน");
        assert!(hard.score > soft.score, "ตึก {} !> นอน {}", hard.score, soft.score);
    }

    #[test]
    fn bracketed_and_multivariant_input_is_handled() {
        // real RID respelling shape: "[กะตุก, กะตุกะ-]" — takes the first variant, strips brackets
        let p = score("[กะตุก, กะตุกะ-]");
        assert!(p.bucket >= 1 && p.bucket <= 5);
        assert!(!p.why.is_empty());
    }

    #[test]
    fn hedge_is_the_verbatim_required_string() {
        assert_eq!(HEDGE, "รูปแบบจากงานวิจัยเรื่องสัทสัญลักษณ์ทั่วไป ยังไม่ใช่ข้อพิสูจน์เฉพาะภาษาไทย");
    }
}
