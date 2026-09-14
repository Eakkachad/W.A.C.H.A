// W3: generate the prebuilt global-PageRank blob for the WASM build. Must build
// the engine EXACTLY as the WASM does (embedded seg cache + seed entries via
// build_from_segmenter) so the graph content hash matches at load time.
use wacha::Engine;
fn main() {
    let seg_bytes = std::fs::read("../wacha-wasm/assets/words_th.seg").expect("seg cache");
    let words_txt = std::fs::read_to_string("../data/words_th.txt").expect("words_th");
    let words: Vec<String> = words_txt.lines().filter(|l| !l.trim().is_empty()).map(|s| s.to_string()).collect();
    let entries = wacha::dictionary::seed_entries();
    let seg = wacha::segmenter::Segmenter::from_cache_bytes(&seg_bytes).expect("seg parse");
    // Same constructor family the WASM uses (build_from_segmenter) — recompute PR here.
    let engine = Engine::build_from_segmenter(words.iter().map(|s| s.as_str()), entries, None, seg);
    let blob = engine.dump_pagerank_cache_bytes();
    std::fs::write("../wacha-wasm/assets/pagerank.blob", &blob).expect("write");
    eprintln!("wrote {} bytes to wacha-wasm/assets/pagerank.blob (entities={})", blob.len(), engine.relation_entity_count());
}
