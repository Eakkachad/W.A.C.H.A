// S1b differential: byte DatrieVocab vs symbol SymbolVocab on the FULL
// production vocab union (words_th.txt ∪ dictionary headwords). Reports every
// word where the two tries disagree on longest_prefix end offset — the gate is
// ZERO mismatches (byte-identical). Also segments the PITCH.md demo sentences.
//
// Run: cargo run --release --example s1b_diff -- ../data
use std::path::Path;
use wacha::symbol_trie;
use wacha::Engine;

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "../data".to_string());
    let dir = Path::new(&dir);

    // Reconstruct the exact segmenter vocab union: words_th.txt ∪ dict headwords.
    let words_txt = std::fs::read_to_string(dir.join("words_th.txt")).expect("words_th.txt");
    let engine = Engine::load_from_dir(dir, |_| {}).expect("engine");
    let mut union: Vec<String> = words_txt.lines().map(|s| s.to_string()).collect();
    union.extend(engine.dict_headwords());

    // Dedup count (matches Segmenter::from_words dedup).
    let distinct: std::collections::HashSet<&str> =
        union.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    println!("union words (raw {}, distinct {})", union.len(), distinct.len());

    let diffs = symbol_trie::differential(union.iter());
    println!("differential mismatches: {}", diffs.len());
    for d in diffs.iter().take(40) {
        println!("  MISMATCH {:?}  byte_end={:?}  sym_end={:?}", d.word, d.byte_end, d.sym_end);
    }
    if diffs.len() > 40 {
        println!("  ... and {} more", diffs.len() - 40);
    }

    // Also confirm the เปล/เปร cluster words (the R6 repro family) specifically.
    let repro: Vec<&String> = diffs
        .iter()
        .map(|d| &d.word)
        .filter(|w| w.starts_with("เปล") || w.starts_with("เปร"))
        .collect();
    println!("เปล/เปร-family mismatches: {}", repro.len());

    if diffs.is_empty() {
        println!("RESULT: BYTE-IDENTICAL ✓ (gate passes)");
    } else {
        println!("RESULT: DIVERGENT ✗ (gate fails — do not merge)");
    }
}
