// S1b timing: cold build of the byte DatrieVocab vs the symbol SymbolVocab on
// the full production vocab union. Reports build time and trie array bytes.
use std::path::Path;
use std::collections::HashMap;
use std::time::Instant;
use wacha::datrie::DatrieVocab;
use wacha::symbol_trie::SymbolVocab;
use wacha::Engine;

fn main() {
    let dir = Path::new("../data");
    let words_txt = std::fs::read_to_string(dir.join("words_th.txt")).expect("words_th.txt");
    let engine = Engine::load_from_dir(dir, |_| {}).expect("engine");
    let mut union: Vec<String> = words_txt.lines().map(|s| s.to_string()).collect();
    union.extend(engine.dict_headwords());

    let mut vocab_map: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut idx = 0usize;
    for w in &union {
        let w = w.trim();
        if w.is_empty() { continue; }
        vocab_map.entry(w.as_bytes().to_vec()).or_insert_with(|| { let i = idx; idx += 1; i });
    }
    println!("vocab words (distinct): {}", vocab_map.len());

    let t = Instant::now();
    let byte = DatrieVocab::build(&vocab_map);
    let byte_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("byte  DatrieVocab build: {:.1} ms  arrays {:.2} MB", byte_ms, byte.inner_bytes() as f64 / 1e6);

    let t = Instant::now();
    let sym = SymbolVocab::build(&vocab_map);
    let sym_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("symbol SymbolVocab build: {:.1} ms  arrays {:.2} MB  alphabet {}", sym_ms, sym.inner_bytes() as f64 / 1e6, sym.alphabet_len());

    println!("speedup: {:.2}x", byte_ms / sym_ms);
}
