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
        "segmenter: {} words | dictionary: {} entries | graph: {} entities, {} triples",
        engine.word_count(),
        engine.entry_count(),
        engine.relation_entity_count(),
        engine.relation_triple_count(),
    );
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
        }
        None => {
            println!("\n(ไม่พบนิยามของคำนี้ในพจนานุกรม — no dictionary entry)");
        }
    }

    if r.related.is_empty() {
        println!("\nคำที่เกี่ยวข้อง: (ไม่มีข้อมูลความสัมพันธ์ — no relationship data)");
    } else {
        println!("\nคำที่เกี่ยวข้อง (related words, ranked; with explanation):");
        for (i, rw) in r.related.iter().enumerate() {
            println!("  {}. {}  (score {:.3})", i + 1, rw.word, rw.score);
            for edge in &rw.path {
                println!("        ↳ {edge}");
            }
        }
    }
    println!();
}
