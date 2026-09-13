use wacha::segmenter::Segmenter;
fn main() {
    let words: Vec<String> = std::fs::read_to_string("../data/words_th.txt").unwrap()
        .lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    // Also include seed headwords so they segment as whole words (Engine::build does this).
    let mut all = words.clone();
    for e in wacha::dictionary::seed_entries() { all.push(e.headword.clone()); }
    let t = std::time::Instant::now();
    let seg = Segmenter::from_words(all.iter());
    eprintln!("built reduced segmenter ({} words) in {:?}", seg.word_count(), t.elapsed());
    let bytes = seg.to_cache_bytes().unwrap();
    std::fs::write("../wacha-wasm/assets/words_th.seg", &bytes).unwrap();
    eprintln!("wrote {} bytes to wacha-wasm/assets/words_th.seg", bytes.len());
}
