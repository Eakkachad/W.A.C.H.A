use std::time::Instant;
use wacha::Engine;
use std::fs;

fn main() {
    let dir = "../data";
    let words_txt = fs::read_to_string(format!("{dir}/words_th.txt")).unwrap();
    eprintln!("read words file: {} bytes", words_txt.len());

    let t0 = Instant::now();
    let engine = Engine::build(words_txt.lines().map(|s| s.to_string()), wacha::dictionary::seed_entries(), None);
    eprintln!("engine built in {:?}, word_count={}", t0.elapsed(), engine.word_count());

    let t1 = Instant::now();
    let toks = engine.segment("แมว");
    eprintln!("segment(single word) took {:?}, tokens={:?}", t1.elapsed(), toks);

    let t2 = Instant::now();
    let toks2 = engine.segment("นักเรียนอ่านหนังสือที่โรงเรียน");
    eprintln!("segment(sentence) took {:?}, tokens={:?}", t2.elapsed(), toks2);
}
