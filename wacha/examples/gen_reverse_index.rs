// C (WASM): build the reverse-dictionary index natively and serialise it to a
// compact postcard blob embedded in the WASM build, so the offline app can do
// reverse lookup without rebuilding the index in the browser.
use wacha::Engine;
fn main() {
    let dir = std::path::Path::new("../data");
    // PUBLIC WASM asset: exclude the ORST-educational-licensed sources (RID 2554
    // + ศัพท์บัญญัติ) — not cleared for public redistribution (R10 R3 / R9 D1).
    let engine = Engine::load_from_dir_opts(dir, false, |_| {}).expect("engine");
    let idx = engine.build_reverse_index();
    eprintln!(
        "reverse index: {} docs / {} terms / ~{:.2} MB (in-mem)",
        idx.doc_count(), idx.term_count(), idx.approx_bytes() as f64 / 1e6
    );
    let bytes = postcard::to_stdvec(&idx).expect("serialize");
    std::fs::write("../wacha-wasm/assets/reverse.idx", &bytes).expect("write");
    eprintln!("wrote {} bytes to wacha-wasm/assets/reverse.idx", bytes.len());
}
