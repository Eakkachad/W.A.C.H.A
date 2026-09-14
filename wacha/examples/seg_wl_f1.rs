// D2: WORD-LEVEL F1 of our greedy segmenter on wisesight1000 (CC0), under the
// AttaCut protocol (arXiv:1911.07056 §4.2). A predicted word counts as a true
// positive ONLY if its exact (start_char, end_char) span matches a gold word's
// span — both boundaries must agree. This is strictly harder than the
// character-level boundary metric (D1): a single wrong internal boundary breaks
// TWO words, not one. We report per-sample mean±std AND micro, BESIDE the D1
// boundary figure, so the two are never confused.
//
// Reports OUR numbers only — never newmm's — and our word list / setup differ
// from the published baselines, so this is not a like-for-like ranking claim.
use wacha::segmenter::Segmenter;

/// (raw text, Vec of (start_char, end_char) spans) for a token list.
fn spans_from_tokens(tokens: &[String]) -> (String, Vec<(usize, usize)>) {
    let mut raw = String::new();
    let mut spans = Vec::new();
    let mut cpos = 0usize;
    for t in tokens {
        let n = t.chars().count();
        if n == 0 { continue; }
        spans.push((cpos, cpos + n));
        cpos += n;
        raw.push_str(t);
    }
    (raw, spans)
}

fn main() {
    let text = std::fs::read_to_string("../data/eval/wisesight1000.label").unwrap();
    let words: Vec<String> = std::fs::read_to_string("../data/words_th.txt").unwrap()
        .lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let mut all = words.clone();
    for e in wacha::dictionary::seed_entries() { all.push(e.headword.clone()); }
    eprintln!("building segmenter ({} words)...", all.len());
    let seg = Segmenter::from_words(all.iter());

    let mut f1s: Vec<f64> = Vec::new();
    let (mut sum_tp, mut sum_fp, mut sum_fn) = (0u64, 0u64, 0u64);
    for line in text.lines() {
        if line.trim().is_empty() { continue; }
        let gold: Vec<String> = line.split('|').map(|s| s.to_string()).collect();
        let (raw, gold_spans) = spans_from_tokens(&gold);
        if raw.is_empty() { continue; }
        let toks = seg.segment(&raw);
        let our_tokens: Vec<String> = toks.into_iter().map(|t| t.text).collect();
        let (_r2, our_spans) = spans_from_tokens(&our_tokens);

        use std::collections::HashSet;
        let g: HashSet<(usize, usize)> = gold_spans.iter().copied().collect();
        let o: HashSet<(usize, usize)> = our_spans.iter().copied().collect();
        let tp = o.intersection(&g).count() as f64;
        let fp = (o.len() as f64) - tp;
        let fnn = (g.len() as f64) - tp;
        sum_tp += tp as u64; sum_fp += fp as u64; sum_fn += fnn as u64;
        let prec = if tp + fp > 0.0 { tp / (tp + fp) } else { 1.0 };
        let rec = if tp + fnn > 0.0 { tp / (tp + fnn) } else { 1.0 };
        let f1 = if prec + rec > 0.0 { 2.0 * prec * rec / (prec + rec) } else { 0.0 };
        f1s.push(f1);
    }
    let n = f1s.len() as f64;
    let mean = f1s.iter().sum::<f64>() / n;
    let std = (f1s.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n).sqrt();
    let (tp, fp, fnn) = (sum_tp as f64, sum_fp as f64, sum_fn as f64);
    let mp = tp / (tp + fp); let mr = tp / (tp + fnn); let mf = 2.0 * mp * mr / (mp + mr);
    println!("wisesight1000 WORD-LEVEL F1 (AttaCut protocol, greedy longest-match, our segmenter, {} samples):", f1s.len());
    println!("  per-sample mean±std F1 = {:.4} ± {:.4}", mean, std);
    println!("  micro P={:.4} R={:.4} F1={:.4}", mp, mr, mf);
    println!("  (compare D1 char-level boundary F1 = 0.8015 ± 0.1660 — word-level is strictly harder & LOWER)");
}
