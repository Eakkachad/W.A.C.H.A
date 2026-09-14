// D2/Q1: word-level F1 (AttaCut protocol) on wisesight1000 (CC0), measured for
// BOTH segmentation modes (greedy longest-match vs Q1 maximal matching). Also
// reports char-level boundary-F1 for each. Our own numbers only.
use wacha::segmenter::{Segmenter, SegMode};

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
fn starts_from_tokens(tokens: &[String]) -> (String, Vec<usize>) {
    let mut raw = String::new();
    let mut starts = Vec::new();
    let mut cpos = 0usize;
    for (i, t) in tokens.iter().enumerate() {
        if i > 0 { starts.push(cpos); }
        cpos += t.chars().count();
        raw.push_str(t);
    }
    (raw, starts)
}

fn measure(seg: &Segmenter, mode: SegMode, label: &str, lines: &[String]) {
    use std::collections::HashSet;
    let (mut wl_f1s, mut b_f1s): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
    let (mut w_tp, mut w_fp, mut w_fn) = (0u64, 0u64, 0u64);
    let (mut b_tp, mut b_fp, mut b_fn) = (0u64, 0u64, 0u64);
    for line in lines {
        if line.trim().is_empty() { continue; }
        let gold: Vec<String> = line.split('|').map(|s| s.to_string()).collect();
        let (raw, gold_spans) = spans_from_tokens(&gold);
        if raw.is_empty() { continue; }
        let (_r, gold_starts) = starts_from_tokens(&gold);
        let ours: Vec<String> = seg.segment_with(&raw, mode).into_iter().map(|t| t.text).collect();
        let (_r2, our_spans) = spans_from_tokens(&ours);
        let (_r3, our_starts) = starts_from_tokens(&ours);
        // word-level
        let g: HashSet<(usize, usize)> = gold_spans.iter().copied().collect();
        let o: HashSet<(usize, usize)> = our_spans.iter().copied().collect();
        let tp = o.intersection(&g).count() as f64;
        let (fp, fnn) = (o.len() as f64 - tp, g.len() as f64 - tp);
        w_tp += tp as u64; w_fp += fp as u64; w_fn += fnn as u64;
        let p = if tp + fp > 0.0 { tp / (tp + fp) } else { 1.0 };
        let r = if tp + fnn > 0.0 { tp / (tp + fnn) } else { 1.0 };
        wl_f1s.push(if p + r > 0.0 { 2.0 * p * r / (p + r) } else { 0.0 });
        // boundary
        let gb: HashSet<usize> = gold_starts.iter().copied().collect();
        let ob: HashSet<usize> = our_starts.iter().copied().collect();
        let btp = ob.intersection(&gb).count() as f64;
        let (bfp, bfnn) = (ob.len() as f64 - btp, gb.len() as f64 - btp);
        b_tp += btp as u64; b_fp += bfp as u64; b_fn += bfnn as u64;
        let bp = if btp + bfp > 0.0 { btp / (btp + bfp) } else { 1.0 };
        let br = if btp + bfnn > 0.0 { btp / (btp + bfnn) } else { 1.0 };
        b_f1s.push(if bp + br > 0.0 { 2.0 * bp * br / (bp + br) } else { 0.0 });
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let std = |v: &[f64], m: f64| (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt();
    let wm = mean(&wl_f1s);
    let bm = mean(&b_f1s);
    let (wtp, wfp, wfn) = (w_tp as f64, w_fp as f64, w_fn as f64);
    let (btp, bfp, bfn) = (b_tp as f64, b_fp as f64, b_fn as f64);
    let micro = |tp: f64, fp: f64, fnn: f64| { let p = tp/(tp+fp); let r = tp/(tp+fnn); (p, r, 2.0*p*r/(p+r)) };
    let (wp, wr, wf) = micro(wtp, wfp, wfn);
    let (bp, br, bf) = micro(btp, bfp, bfn);
    println!("--- {label} ({} samples) ---", wl_f1s.len());
    println!("  word-level  F1: per-sample {:.4} ± {:.4}  | micro P/R/F1 {:.4}/{:.4}/{:.4}", wm, std(&wl_f1s, wm), wp, wr, wf);
    println!("  boundary    F1: per-sample {:.4} ± {:.4}  | micro P/R/F1 {:.4}/{:.4}/{:.4}", bm, std(&b_f1s, bm), bp, br, bf);
}

fn main() {
    let text = std::fs::read_to_string("../data/eval/wisesight1000.label").unwrap();
    let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let words: Vec<String> = std::fs::read_to_string("../data/words_th.txt").unwrap()
        .lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let mut all = words.clone();
    for e in wacha::dictionary::seed_entries() { all.push(e.headword.clone()); }
    eprintln!("building segmenter ({} words)...", all.len());
    let seg = Segmenter::from_words(all.iter());
    println!("wisesight1000 — our own numbers, both modes (AttaCut protocol):");
    measure(&seg, SegMode::Greedy, "GREEDY longest-match", &lines);
    measure(&seg, SegMode::MaximalMatching, "MAXIMAL matching (newmm-style DP)", &lines);
    println!("(newmm published word-level F1 on Wisesight-1000 = 0.74; ours is our word list/setup, not a reproduction.)");
}
