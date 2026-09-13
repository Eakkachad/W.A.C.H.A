// D1: boundary-F1 of our greedy segmenter on wisesight1000 (CC0).
// Protocol: char-level is_beginning. For each sample, reconstruct raw text from
// the gold (strip '|'), segment with our engine, compare the SET of word-start
// char indices (excluding position 0, which is trivially a boundary) — standard
// boundary detection P/R/F1. Report per-sample mean±std.
use wacha::segmenter::Segmenter;
fn boundaries_from_tokens(tokens: &[String]) -> (String, Vec<usize>) {
    // returns (raw text, set of char-start indices for tokens[1..])
    let mut raw = String::new();
    let mut starts = Vec::new();
    let mut cpos = 0usize; // char index
    for (i, t) in tokens.iter().enumerate() {
        if i > 0 { starts.push(cpos); }
        cpos += t.chars().count();
        raw.push_str(t);
    }
    (raw, starts)
}
fn main() {
    let text = std::fs::read_to_string("../data/eval/wisesight1000.label").unwrap();
    // Build the real engine's segmenter (words_th + seed headwords), like production.
    let words: Vec<String> = std::fs::read_to_string("../data/words_th.txt").unwrap()
        .lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let mut all = words.clone();
    for e in wacha::dictionary::seed_entries() { all.push(e.headword.clone()); }
    eprintln!("building segmenter ({} words)...", all.len());
    let seg = Segmenter::from_words(all.iter());

    let mut f1s: Vec<f64> = Vec::new();
    let mut sum_tp=0u64; let mut sum_fp=0u64; let mut sum_fn=0u64;
    for line in text.lines() {
        if line.trim().is_empty() { continue; }
        // gold tokens: split on '|'. Note "| |" encodes a space token " ".
        let gold: Vec<String> = line.split('|').map(|s| s.to_string()).collect();
        let (raw, gold_starts) = boundaries_from_tokens(&gold);
        if raw.is_empty() { continue; }
        // our segmentation
        let toks = seg.segment(&raw);
        let our_tokens: Vec<String> = toks.into_iter().map(|t| t.text).collect();
        let (_raw2, our_starts) = boundaries_from_tokens(&our_tokens);
        use std::collections::HashSet;
        let g: HashSet<usize> = gold_starts.iter().copied().collect();
        let o: HashSet<usize> = our_starts.iter().copied().collect();
        let tp = o.intersection(&g).count() as f64;
        let fp = o.difference(&g).count() as f64;
        let fnn = g.difference(&o).count() as f64;
        sum_tp += tp as u64; sum_fp += fp as u64; sum_fn += fnn as u64;
        let prec = if tp+fp>0.0 { tp/(tp+fp) } else { 1.0 };
        let rec = if tp+fnn>0.0 { tp/(tp+fnn) } else { 1.0 };
        let f1 = if prec+rec>0.0 { 2.0*prec*rec/(prec+rec) } else { 0.0 };
        f1s.push(f1);
    }
    let n = f1s.len() as f64;
    let mean = f1s.iter().sum::<f64>()/n;
    let var = f1s.iter().map(|x|(x-mean).powi(2)).sum::<f64>()/n;
    let std = var.sqrt();
    // micro (corpus-level) F1
    let (tp,fp,fnn)=(sum_tp as f64,sum_fp as f64,sum_fn as f64);
    let mp=tp/(tp+fp); let mr=tp/(tp+fnn); let mf=2.0*mp*mr/(mp+mr);
    println!("wisesight1000 boundary-F1 (greedy longest-match, our segmenter, {} samples):", f1s.len());
    println!("  per-sample mean±std F1 = {:.4} ± {:.4}", mean, std);
    println!("  micro P={:.4} R={:.4} F1={:.4}", mp, mr, mf);
}
