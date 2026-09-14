// A3 (SYNTHETIC — not measured on real data): scale-headroom curve. Generate a
// synthetic Thai-like word list at multiples of the real RID scale (~40k
// headwords) and measure the dominant ingestion cost — the byte-trie segmenter
// build — plus the resulting trie array size, at 1×/2×/5×/10×.
//
// This proves the ingestion path SCALES; it is NOT a measurement on the real
// RID (which we were not given). Labeled SYNTHETIC in BENCHMARKS §5.
//
// Run: cargo run --release --example a3_scale
use std::time::Instant;
use wacha::segmenter::Segmenter;

/// Deterministic LCG so the synthetic corpus is reproducible (no rand dep).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
}

fn main() {
    // Thai consonant + vowel building blocks for plausible-shaped headwords.
    let cons: Vec<char> = "กขคงจฉชซญฐณดตถทธนบปผฝพฟภมยรลวศษสหอฮ".chars().collect();
    let vows: Vec<char> = "ะัาำิีึืุูเแโใไ".chars().collect();

    // Real RID scale ≈ 40,000 headwords (organizer figure). Base multiples.
    const BASE: usize = 40_000;
    let scales = [1usize, 2, 5, 10];

    println!("=== A3 SYNTHETIC scale-headroom (byte segmenter build) ===");
    println!("base = {BASE} synthetic headwords (≈ real RID scale); NOT real RID data");
    println!("{:>4}  {:>10}  {:>12}  {:>14}  {:>12}", "×", "words", "build_ms", "trie_MB", "ms/1k_words");

    for &s in &scales {
        let n = BASE * s;
        let mut rng = Lcg(0xA3_2026_u64 ^ (s as u64));
        // Generate n distinct synthetic words (2–5 syllables each).
        let mut set = std::collections::HashSet::with_capacity(n);
        let mut words: Vec<String> = Vec::with_capacity(n);
        while words.len() < n {
            let syls = 2 + (rng.next() % 4) as usize; // 2..=5
            let mut w = String::new();
            for _ in 0..syls {
                w.push(cons[(rng.next() as usize) % cons.len()]);
                w.push(vows[(rng.next() as usize) % vows.len()]);
            }
            if set.insert(w.clone()) {
                words.push(w);
            }
        }

        let t = Instant::now();
        let seg = Segmenter::from_words(words.iter());
        let build_ms = t.elapsed().as_secs_f64() * 1000.0;
        let trie_mb = seg.trie_bytes() as f64 / 1e6;
        println!(
            "{:>4}  {:>10}  {:>12.0}  {:>14.2}  {:>12.2}",
            s, n, build_ms, trie_mb, build_ms / (n as f64 / 1000.0)
        );
    }
    println!("(build is the dominant day-of ingestion cost; segmentation/lookup are ms.");
    println!(" S1b's dense-alphabet trie would cut this ~7× but is STOPPED on its correctness gate.)");
}
