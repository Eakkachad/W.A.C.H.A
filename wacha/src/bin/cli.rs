//! `wacha` CLI — วาจา (WACHA), the demoable interface for the hybrid Thai dictionary.
//!
//! The full user journey: type a Thai word, see it segmented, get its
//! definition, and see the words most related to it *with an explanation* of the
//! connection (Personalized PageRank + BFS path over dictionary-derived triples).
//!
//! Usage:
//!   wacha                         # interactive REPL (seed data)
//!   wacha --data DIR              # interactive REPL (real word list from DIR)
//!   wacha lookup แมว              # one-shot lookup
//!   wacha segment "แมวใหญ่ชอบกินปลา"   # one-shot segmentation
//!   wacha stats                   # engine stats
//!
//! --data DIR expects `words_th.txt` (one word per line) and optionally
//! `tnc_freq.txt` (word<TAB>count). Both are CC0 PyThaiNLP corpora (see
//! dictionary.rs provenance).

use wacha::{segmenter::Token, Engine, Lookup};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse an optional `--data DIR` flag anywhere in the args.
    let mut data_dir: Option<String> = None;
    let mut positional: Vec<String> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--data" => {
                if i + 1 < args.len() {
                    data_dir = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("error: --data requires a directory path");
                    std::process::exit(2);
                }
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => {
                positional.push(other.to_string());
                i += 1;
            }
        }
    }

    let engine = match build_engine(data_dir.as_deref()) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("error building engine: {err}");
            std::process::exit(1);
        }
    };

    // Dispatch on the first positional as a subcommand.
    match positional.split_first() {
        None => repl(&engine),
        Some((cmd, rest)) => match cmd.as_str() {
            "stats" => print_stats(&engine),
            "audit" => print_audit(&engine),
            "corroboration" => print_corroboration(&engine),
            "patk" => print_patk(&engine, rest),
            "field" => {
                let english = rest.join(" ");
                if english.is_empty() {
                    eprintln!("usage: wacha field <english term>  (e.g. field, computer)");
                    std::process::exit(2);
                }
                print_field(data_dir.as_deref(), &english);
            }
            "segment" => {
                let text = rest.join(" ");
                if text.is_empty() {
                    eprintln!("usage: wacha segment <thai text>");
                    std::process::exit(2);
                }
                print_segmentation(&engine.segment(&text));
            }
            "lookup" => {
                let word = rest.join(" ");
                if word.is_empty() {
                    eprintln!("usage: wacha lookup <word>");
                    std::process::exit(2);
                }
                print_lookup(&engine, &engine.lookup(&word, 8), &word);
            }
            // Bare word with no subcommand -> treat as a lookup.
            other => {
                let word = std::iter::once(other.to_string())
                    .chain(rest.iter().cloned())
                    .collect::<Vec<_>>()
                    .join(" ");
                print_lookup(&engine, &engine.lookup(&word, 8), &word);
            }
        },
    }
}

fn build_engine(data_dir: Option<&str>) -> Result<Engine, String> {
    match data_dir {
        None => Ok(Engine::seed_only()),
        Some(dir) => Engine::load_from_dir(Path::new(dir), |m| eprintln!("{m}"))
            .map_err(|e| format!("loading engine from {dir}: {e}")),
    }
}

fn repl(engine: &Engine) {
    println!("=== ORST Dictionary Reimagined — hybrid engine ===");
    print_stats(engine);
    println!("\nType a Thai word and press enter. Commands: :seg <text>, :quit\n");

    let stdin = io::stdin();
    loop {
        print!("คำ> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("input error: {e}");
                break;
            }
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == ":quit" || line == ":q" {
            break;
        }
        if let Some(text) = line.strip_prefix(":seg ") {
            print_segmentation(&engine.segment(text.trim()));
            continue;
        }
        print_lookup(engine, &engine.lookup(line, 8), line);
    }
    println!("bye");
}

fn print_help() {
    println!(
        "วาจา (WACHA) — hybrid Thai dictionary (segmenter + explainable relationship graph)\n\n\
         USAGE:\n\
         \twacha [--data DIR] [SUBCOMMAND]\n\n\
         SUBCOMMANDS:\n\
         \t(none)                interactive REPL\n\
         \tlookup <word>         segment + define + related words\n\
         \tsegment <thai text>   segment text into words\n\
         \tstats                 engine statistics\n\n\
         OPTIONS:\n\
         \t--data DIR   load words_th.txt (+ optional tnc_freq.txt) from DIR\n\
         \t-h, --help   this help"
    );
}

fn print_stats(engine: &Engine) {
    println!(
        "segmenter: {} words | dictionary: {} entries | graph: {} entities, {} triples | learner content: {} words",
        engine.word_count(),
        engine.entry_count(),
        engine.relation_entity_count(),
        engine.relation_triple_count(),
        engine.learner_count(),
    );
    println!("graph sense nodes: {}", engine.relation_sense_count());
    let (defined, searchable, pct) = engine.definition_coverage();
    println!(
        "definition coverage: {defined}/{searchable} searchable words = {pct:.1}% have ≥1 definition"
    );
    for n in [100usize, 1000, 5000] {
        let (d, c, p) = engine.frequency_weighted_coverage(n);
        println!("frequency-weighted coverage @ top-{n}: {d}/{c} = {p:.1}% (denominator = {c} most-frequent words)");
    }
}

/// A2: the 47 hand-audited seed↔WordNet pairs from
/// `wacha/data/seed_wordnet_audit_2026-09-13.md`, with their KEEP/CUT verdict.
/// `true` = KEEP (a genuine relation the engine should still return),
/// `false` = CUT (must be absent). Ordered as in the audit doc.
const AUDIT_PAIRS: &[(&str, &str, bool)] = &[
    ("ภาษา", "การสื่อสารด้วยภาษา", true),
    ("เขียน", "ขีดเขียน", true),
    ("ครู", "ครูบาอาจารย์", true),
    ("ครู", "ผู้สอน", true),
    ("ครู", "ผู้สาธิตวิธีการ", false),
    ("ครู", "ผู้ให้ความรู้", true),
    ("ครู", "อ.", true),
    ("ครู", "อาจารย์", true),
    ("อาจารย์", "ครูบาอาจารย์", true),
    ("พจนานุกรม", "ดิก", true),
    ("พจนานุกรม", "ดิกชันนารี", true),
    ("เขียน", "ทำหนังสือ", true),
    ("นักเรียน", "นร.", true),
    ("นักเรียน", "นศ.", false),
    ("นักเรียน", "นักวิชาการ", false),
    ("นักเรียน", "นักศึกษา", false),
    ("นักเรียน", "นิสิต", false),
    ("นักเรียน", "นิสิตนักศึกษา", false),
    ("นักเรียน", "ผู้ศึกษา", true),
    ("นักเรียน", "ผู้เรียน", true),
    ("นักเรียน", "เด็กนักเรียน", true),
    ("เล็ก", "น้อย", true),
    ("พจนานุกรม", "ปทานุกรม", true),
    ("เขียน", "ประพันธ์", true),
    ("อาจารย์", "ผู้สอน", true),
    ("อาจารย์", "ผู้ให้ความรู้", true),
    ("ภาษา", "ภาษาธรรมชาติ", true),
    ("โรงเรียน", "ร.ร.", true),
    ("เขียน", "รจนา", true),
    ("โรงเรียน", "รร.", true),
    ("หนังสือ", "สมุด", true),
    ("สัตว์", "สัตว์ป่า", true),
    ("สัตว์", "สัตว์เดียรัจฉาน", true),
    ("สัตว์", "สิ่งมีชีวิต", true),
    ("สัตว์", "เดียรัจฉาน", true),
    ("สุนัข", "หมา", true),
    ("สุนัข", "หมาบ้าน", true),
    ("หนังสือ", "หนังสือหนังหา", true),
    ("หนังสือ", "เล่ม", true),
    ("หมา", "หมาบ้าน", true),
    ("ใหญ่", "หลัก", false),
    ("อาจารย์", "อ.", true),
    ("โรงเรียน", "อาคารเรียน", true),
    ("เขียน", "เขียนหนังสือ", true),
    ("เขียน", "แต่ง", true),
    ("เสือ", "เสือโคร่ง", true),
    ("แมว", "แมวบ้าน", true),
];

/// A2: measure KEEP-recall and CUT-absence against the 47 audited pairs, plus
/// the cross-sense recount (A2.4). Every number here is reproducible from the
/// live engine — this is what `scripts/verify_r5.sh` calls.
fn print_audit(engine: &Engine) {
    // A2.4 — cross-sense candidate recount (must be 0).
    println!("cross_sense_pairs = {}", engine.cross_sense_pair_count());

    const TOP_K: usize = 50; // deep enough to catch a pair if it exists at all
    let (mut keep_total, mut keep_hit) = (0usize, 0usize);
    let (mut cut_total, mut cut_absent) = (0usize, 0usize);
    for &(a, b, keep) in AUDIT_PAIRS {
        // A relation is "present" if b is among a's related results OR a is
        // among b's (relations are symmetric; either direction counts).
        let present = engine.related_contains(a, b, TOP_K) || engine.related_contains(b, a, TOP_K);
        if keep {
            keep_total += 1;
            if present {
                keep_hit += 1;
            }
        } else {
            cut_total += 1;
            if !present {
                cut_absent += 1;
            }
        }
    }
    let keep_pct = if keep_total > 0 { 100.0 * keep_hit as f64 / keep_total as f64 } else { 0.0 };
    let cut_pct = if cut_total > 0 { 100.0 * cut_absent as f64 / cut_total as f64 } else { 0.0 };
    println!("KEEP_recall = {keep_hit}/{keep_total} = {keep_pct:.1}%");
    println!("CUT_absence = {cut_absent}/{cut_total} = {cut_pct:.1}%");
    // Per-pair detail (so a reviewer can see exactly which pair moved).
    for &(a, b, keep) in AUDIT_PAIRS {
        let present = engine.related_contains(a, b, TOP_K) || engine.related_contains(b, a, TOP_K);
        let verdict = if keep { "KEEP" } else { "CUT " };
        let ok = if keep == present { "ok " } else { "MISS" };
        let state = if present { "present" } else { "absent " };
        println!("  [{ok}] {verdict} {a} ⟷ {b}: {state}");
    }
}

/// The §1.2 closing-demo table: one English word → Thai equivalents grouped by
/// ORST discipline, read from the offline cache (data/coined_word_cache/).
fn print_field(data_dir: Option<&str>, english: &str) {
    let Some(dir) = data_dir else {
        eprintln!("field requires --data DIR (needs DIR/coined_word_cache/)");
        std::process::exit(2);
    };
    let cache = Path::new(dir).join("coined_word_cache");
    match wacha::import::coined_word::field_view(&cache, english) {
        Some(rows) => {
            println!("\n──────── ศัพท์บัญญัติ: \"{english}\" ────────");
            println!("คำอังกฤษเดียว → หลายคำไทย จำแนกตามสาขาวิชา (ที่มา: ราชบัณฑิตยสภา)\n");
            for (disc, terms) in &rows {
                println!("  {:<40} {}", disc, terms.join(", "));
            }
            println!("\n({} สาขาวิชา · ที่มา ORST · educational/non-commercial)", rows.len());
        }
        None => {
            eprintln!("no cached ศัพท์บัญญัติ result for \"{english}\" (run scripts/fetch_coined_word.sh)");
            std::process::exit(1);
        }
    }
}

/// Phase N: cross-source corroboration — source-overlap table + a seeded,
/// stratified-by-tier sample for hand-audit. Tiers are PRE-REGISTERED in
/// `relations::corroboration_tier` (defined before any audit).
fn print_corroboration(engine: &Engine) {
    use std::collections::BTreeMap;
    let pairs = engine.enumerate_relation_pairs();
    println!("total distinct related pairs: {}", pairs.len());

    // Source-overlap table: count pairs by their exact attesting-source SET.
    let mut by_set: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_tier: BTreeMap<u8, usize> = BTreeMap::new();
    // "unique to one source" vs "corroborated (≥2 sources)".
    let (mut unique, mut corroborated) = (0usize, 0usize);
    let mut per_source_total: BTreeMap<&str, usize> = BTreeMap::new();
    for p in &pairs {
        let names = wacha::relations::RelationEngine::source_set_names(p.src_set);
        let key = names.join("+");
        *by_set.entry(key).or_default() += 1;
        *by_tier.entry(p.tier).or_default() += 1;
        if p.src_set.count_ones() >= 2 { corroborated += 1; } else { unique += 1; }
        for n in &names { *per_source_total.entry(n).or_default() += 1; }
    }
    println!("\n[source-overlap] pairs by attesting-source set:");
    for (k, v) in &by_set {
        println!("  {:<28} {}", k, v);
    }
    println!("\n[source-overlap] pairs each source participates in (any tier):");
    for (k, v) in &per_source_total {
        println!("  {:<12} {}", k, v);
    }
    println!("\n[source-overlap] single-source: {unique}  ·  multi-source (≥2, corroborated): {corroborated}");

    println!("\n[tiers] pairs per pre-registered corroboration tier:");
    let tname = |t: u8| match t { 3 => "ORST-attested", 2 => "multi-source(≥2)", 1 => "single-source-corroborated(synset≥3)", _ => "isolated-pair" };
    for (t, v) in by_tier.iter().rev() {
        println!("  tier {t} {:<38} {v}", tname(*t));
    }

    // Stratified sample: up to 40 pairs per tier, deterministic (FIXED SEED).
    // Deterministic LCG (no rand dep). Seed pre-registered = 0xN6_2026.
    const SEED: u64 = 0x4e36_2026;
    const PER_TIER: usize = 40;
    println!("\n[sample] stratified by tier, {PER_TIER}/tier, FIXED SEED {SEED:#x} — hand-audit these:");
    for tier in (0u8..=3).rev() {
        let mut idxs: Vec<usize> = pairs.iter().enumerate().filter(|(_, p)| p.tier == tier).map(|(i, _)| i).collect();
        // Deterministic shuffle: sort by a hash of (seed, index).
        idxs.sort_by_key(|&i| {
            let mut h = SEED ^ (i as u64).wrapping_mul(0x9E3779B97F4A7C15);
            h ^= h >> 33; h = h.wrapping_mul(0xff51afd7ed558ccd); h ^= h >> 33;
            h
        });
        let take = idxs.len().min(PER_TIER);
        println!("\n--- tier {tier} ({}) — n_available={}, sampled={} ---", tname(tier), idxs.len(), take);
        for &i in idxs.iter().take(take) {
            let p = &pairs[i];
            let names = wacha::relations::RelationEngine::source_set_names(p.src_set).join("+");
            println!("  {} ⟷ {}   [{}]", p.a, p.b, names);
        }
    }
}

/// T1 precision@5 sampling: a deterministic seeded sample of query words that
/// have ≥5 related results, each with its current top-5 ranked list, for
/// hand-audit. The seed here is DIFFERENT from the tier-audit seed so precision
/// is measured on data the bands were NOT fitted to. Usage: `patk [n] [seed]`.
fn print_patk(engine: &Engine, args: &[String]) {
    let n: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(30);
    let seed: u64 = args.get(1).and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x5237_2026); // R7 held-out seed (≠ 0x4e362026 tier-fit seed)
    // Candidate query words: those with ≥5 related results. Enumerate over the
    // relation graph's words deterministically.
    let mut candidates: Vec<String> = engine.relation_words_with_min_related(5);
    candidates.sort();
    // Deterministic shuffle by hashing (seed, index).
    let mut idx: Vec<usize> = (0..candidates.len()).collect();
    idx.sort_by_key(|&i| {
        let mut h = seed ^ (i as u64).wrapping_mul(0x9E3779B97F4A7C15);
        h ^= h >> 33; h = h.wrapping_mul(0xff51afd7ed558ccd); h ^= h >> 33; h
    });
    println!("precision@5 sample: n={n} seed={seed:#x} (candidates with ≥5 related: {})", candidates.len());
    for &i in idx.iter().take(n) {
        let q = &candidates[i];
        let rel = engine.related_ranked(q, 5);
        let items: Vec<String> = rel.iter().map(|r| format!("{}[{}]", r.word, r.source_short())).collect();
        println!("  {q}: {}", items.join(", "));
    }
}

fn print_segmentation(tokens: &[Token]) {
    let rendered: Vec<String> = tokens
        .iter()
        .map(|t| if t.in_vocab { t.text.clone() } else { format!("[{}]", t.text) })
        .collect();
    println!("segmented: {}", rendered.join(" | "));
    let oov = tokens.iter().filter(|t| !t.in_vocab).count();
    if oov > 0 {
        println!("  ({oov} out-of-vocabulary cluster(s), shown in [brackets])");
    }
}

fn print_lookup(_engine: &Engine, r: &Lookup, query: &str) {
    println!("\n──────── {query} ────────");
    print_segmentation(&r.segmentation);

    match &r.entry {
        Some(e) => {
            println!("\nคำ: {}  ({})", e.word, e.pos);
            println!("นิยาม: {}", e.definition);
            if !e.classifiers.is_empty() {
                println!("ลักษณนาม: {}", e.classifiers.join(", "));
            }
            let mut tags = Vec::new();
            if let Some(sub) = &e.subject {
                tags.push(format!("สาขา: {sub}"));
            }
            if let Some(reg) = &e.register {
                tags.push(format!("ทะเบียนคำ: {reg}"));
            }
            if !tags.is_empty() {
                println!("{}", tags.join("  ·  "));
            }
            if !e.source.is_empty() {
                println!("  (ที่มานิยาม: {} · {})", e.source, e.license);
            }
        }
        None => {
            println!("\n(ไม่พบนิยามของคำนี้ในพจนานุกรม — no dictionary entry)");
        }
    }

    // Offline-precomputed learner content — a clearly-labeled enrichment, shown
    // separately from the formal definition, with its honest provenance.
    if let Some(l) = &r.learner {
        println!("\nคำอธิบายง่าย: {}", l.simple);
        println!("ตัวอย่างประโยค: {}", l.example);
        println!("  (เนื้อหาสำหรับผู้เรียน · จัดทำล่วงหน้าออฟไลน์ · ที่มา: {})", l.source);
    }

    if r.related.is_empty() {
        println!("\nคำที่เกี่ยวข้อง: (ไม่มีข้อมูลความสัมพันธ์ — no relationship data)");
    } else {
        println!("\nคำที่เกี่ยวข้อง (related words, ranked; with explanation):");
        let mut any_wordnet = false;
        let mut any_unverified = false;
        for (i, rw) in r.related.iter().enumerate() {
            if rw.source == wacha::relations::RelationSource::WordNet {
                any_wordnet = true;
            }
            // Only mark low-confidence (Unverified) relations — Confirmed is the
            // quiet default so the output isn't cluttered.
            let conf_mark = if rw.confidence == wacha::relations::RelationConfidence::Unverified {
                any_unverified = true;
                "  ⚠ ยังไม่ยืนยัน"
            } else {
                ""
            };
            println!(
                "  {}. {}  (score {:.3})  [{}]{}",
                i + 1,
                rw.word,
                rw.score,
                rw.source.tag(),
                conf_mark
            );
            for edge in &rw.path {
                println!("        ↳ {edge}");
            }
        }
        // Honest legend: WordNet relations are auto-extracted and not hand-checked.
        if any_wordnet {
            println!(
                "\n  [ตรวจแล้ว] = ความสัมพันธ์ที่ตรวจสอบด้วยมือ · \
                 [WordNet (อัตโนมัติ)] = สกัดจาก Thai WordNet โดยอัตโนมัติ ยังไม่ได้ตรวจทีละคู่ อาจมีคู่ที่ไม่แม่นยำ"
            );
        }
        if any_unverified {
            println!(
                "  ⚠ ยังไม่ยืนยัน = คู่คำโดดเดี่ยวใน WordNet (ไม่มีชุดคำอื่นยืนยันซ้ำ) — \
                 อาจถูกต้องหรือไม่ก็ได้ ควรตรวจก่อนเชื่อ"
            );
        }
    }
    println!();
}
