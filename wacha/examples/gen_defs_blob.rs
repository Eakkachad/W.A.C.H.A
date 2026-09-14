// W2 + L1: build a compact definitions blob for the WASM build.
// Format (little-endian), sorted by headword for binary search, NO JSON:
//   [u32 count]
//   count × record:
//     [u16 hw_len][hw utf8]
//     [u16 def_len][def utf8]
//     [u8 src_code]
//     [u16 ex_len][ex utf8]   -- L1: up to 3 usage examples joined by \x1f
// src_code: 0 seed/ตรวจด้วยมือ, 1 Kaikki, 2 CoinedWord, 3 RID, 4 other.
use std::io::Write;
use wacha::Engine;
fn src_code(label: &str) -> u8 {
    if label.contains("ตรวจ") { 0 }
    else if label.contains("Kaikki") { 1 }
    else if label.contains("ศัพท์บัญญัติ") { 2 }
    else if label.contains("RID") { 3 }
    else { 4 }
}
fn main() {
    let dir = std::path::Path::new("../data");
    let engine = Engine::load_from_dir(dir, |_| {}).expect("engine");
    let defs = engine.export_definitions_with_examples(3);
    let with_ex = defs.iter().filter(|(_, _, _, ex)| !ex.is_empty()).count();
    eprintln!("defined headwords: {} ({} carry >=1 usage example)", defs.len(), with_ex);
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&(defs.len() as u32).to_le_bytes());
    for (hw, def, src, ex) in &defs {
        let hb = hw.as_bytes(); let db = def.as_bytes();
        buf.extend_from_slice(&(hb.len() as u16).to_le_bytes());
        buf.extend_from_slice(hb);
        buf.extend_from_slice(&(db.len() as u16).to_le_bytes());
        buf.extend_from_slice(db);
        buf.push(src_code(src));
        // Examples joined by unit-separator; empty string == no examples.
        let joined = ex.join("\u{1f}");
        let eb = joined.as_bytes();
        buf.extend_from_slice(&(eb.len() as u16).to_le_bytes());
        buf.extend_from_slice(eb);
    }
    let mut f = std::fs::File::create("../wacha-wasm/assets/defs.blob").unwrap();
    f.write_all(&buf).unwrap();
    eprintln!("wrote {} bytes to wacha-wasm/assets/defs.blob", buf.len());
}
