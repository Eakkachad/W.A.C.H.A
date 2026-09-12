//! Day-0 feasibility POC for the ORST Dictionary Reimagined Hackathon hybrid plan.
//!
//! Two independent checks, each answering a yes/no feasibility question from
//! `dict-hackathon/AGENT_HANDOFF.md`:
//!
//! 1. Can `katgpt-tokenizer`'s `DatrieVocab::longest_prefix` drive a real Thai
//!    greedy longest-match word segmenter, built directly from a word list?
//! 2. Does AXIOM's vendored `graph.rs` (`KnowledgeGraph`) give sane, explainable
//!    multi-hop output on hand-built Thai word-relationship triples?

mod graph;

use katgpt_tokenizer::DatrieVocab;
use std::collections::HashMap;

/// Greedy longest-match segmenter over a `DatrieVocab`, operating on raw UTF-8
/// bytes (Thai has no spaces between words, so this is the entire problem).
/// Falls back to a single Thai "character" (a UTF-8 codepoint start byte run)
/// when no dictionary entry matches at the current position, so unknown words
/// never cause an infinite loop or a panic — that fallback behavior is itself
/// one of the things this POC exists to observe.
fn segment(vocab: &DatrieVocab, text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut pos = 0;
    let mut out = Vec::new();
    while pos < bytes.len() {
        if let Some((_start, end)) = vocab.longest_prefix(bytes, pos) {
            out.push(String::from_utf8_lossy(&bytes[pos..end]).into_owned());
            pos = end;
        } else {
            // OOV fallback: consume exactly one UTF-8 codepoint.
            let mut end = pos + 1;
            while end < bytes.len() && (bytes[end] & 0b1100_0000) == 0b1000_0000 {
                end += 1;
            }
            out.push(String::from_utf8_lossy(&bytes[pos..end]).into_owned());
            pos = end;
        }
    }
    out
}

fn run_segmenter_poc() {
    println!("=== POC 1: katgpt-tokenizer Datrie longest-match Thai segmenter ===\n");

    // A tiny hand-typed "dictionary" — stand-in for a real RID-derived word
    // list (still unresolved, see AGENT_HANDOFF.md §6). Real Thai words only.
    let words = [
        "นักเรียน", "อ่าน", "หนังสือ", "ที่", "โรงเรียน", "แมว", "กิน", "ปลา", "ใหญ่",
        "เล็ก", "บ้าน", "คน", "รัก", "ชอบ", "สุนัข", "สวน", "ครู", "สอน", "ภาษา", "ไทย",
        "คำ", "ความหมาย", "พจนานุกรม",
    ];
    let mut vocab_map: HashMap<Vec<u8>, usize> = HashMap::new();
    for (i, w) in words.iter().enumerate() {
        vocab_map.insert(w.as_bytes().to_vec(), i);
    }
    let vocab = DatrieVocab::build(&vocab_map);
    println!("Built DatrieVocab: {} entries, {} bytes\n", words.len(), vocab.inner_bytes());

    let test_sentences = [
        // All words in-vocab.
        "นักเรียนอ่านหนังสือที่โรงเรียน",
        "แมวใหญ่ชอบกินปลาที่บ้าน",
        "ครูสอนภาษาไทยที่โรงเรียน",
        // Deliberately includes an out-of-vocabulary word ("เด็กน้อย" — neither
        // "เด็ก" nor "น้อย" is in the word list above) to observe fallback behavior.
        "เด็กน้อยรักสุนัข",
    ];

    for s in test_sentences {
        let segments = segment(&vocab, s);
        println!("input : {s}");
        println!("output: {}", segments.join(" | "));
        println!();
    }
}

fn run_graph_poc() {
    println!("=== POC 2: vendored AXIOM graph.rs — explainable word relationships ===\n");

    let mut kg = graph::KnowledgeGraph::new();
    // Hand-built triples standing in for RID-derived relations (synonym,
    // antonym, category, usage-context) — proves add_triple/PPR/BFS work on
    // Thai UTF-8 strings without any of AXIOM's English-only NLU layer.
    let triples: &[(&str, &str, &str)] = &[
        ("แมว", "เป็นชนิดของ", "สัตว์"),
        ("สุนัข", "เป็นชนิดของ", "สัตว์"),
        ("เสือ", "เป็นชนิดของ", "สัตว์"),
        ("ใหญ่", "ตรงข้ามกับ", "เล็ก"),
        ("สุข", "ตรงข้ามกับ", "เศร้า"),
        ("ครู", "สอนที่", "โรงเรียน"),
        ("นักเรียน", "เรียนที่", "โรงเรียน"),
        ("หนังสือ", "ใช้ที่", "โรงเรียน"),
        ("พจนานุกรม", "อธิบาย", "คำ"),
        ("คำ", "มี", "ความหมาย"),
        ("ภาษาไทย", "ใช้", "คำ"),
        ("แมว", "ชอบ", "ปลา"),
        ("แมว", "อาศัยอยู่ที่", "บ้าน"),
        ("สุนัข", "อาศัยอยู่ที่", "บ้าน"),
    ];
    for (s, r, o) in triples {
        kg.add_triple(s, r, o);
    }
    println!(
        "Built KnowledgeGraph: {} entities, {} triples\n",
        kg.entity_count(),
        triples.len()
    );

    // Query: "how is แมว (cat) related to things, ranked by relevance?"
    let seed_word = "แมว";
    let seed_id = kg.entity_id(seed_word).expect("seed word must be in graph");
    let scores = kg.personalized_pagerank(&[seed_id], 20);

    let mut ranked: Vec<(usize, f32)> = scores.iter().copied().enumerate().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Top entities by Personalized PageRank, seeded on \"{seed_word}\":");
    for (id, score) in ranked.iter().take(6) {
        if *id == seed_id {
            continue;
        }
        println!("  {:<12} score={:.4}", kg.entity_name(*id), score);
    }
    println!();

    println!("Explainable path — BFS subgraph (2 hops) from \"{seed_word}\":");
    for t in kg.bfs_subgraph(&[seed_id], 2) {
        println!(
            "  {} --{}--> {}",
            kg.entity_name(t.subject_id),
            kg.relation_name(t.relation_id),
            kg.entity_name(t.object_id)
        );
    }
}

fn main() {
    run_segmenter_poc();
    println!();
    run_graph_poc();
}
