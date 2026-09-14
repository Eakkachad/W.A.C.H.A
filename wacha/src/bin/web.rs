//! `wacha-web` — a minimal, dependency-free web UI for the วาจา (WACHA) hybrid
//! Thai dictionary engine, for the ORST hackathon demo.
//!
//! Zero external web/async dependencies: a small blocking HTTP/1.1 server built
//! on `std::net::TcpListener` with a thread per connection. The `Engine` is
//! built **once at startup** (via the shared cache-aware [`Engine::load_from_dir`])
//! and shared across all requests behind an `Arc` — never rebuilt per request.
//!
//! Routes:
//!   GET  /                      → the embedded single-page UI (HTML+JS+CSS)
//!   GET  /api/lookup?q=<word>    → JSON: { query, segmentation[], entry?, related[] }
//!   GET  /healthz                → "ok"
//!
//! Usage:
//!   wacha-web                    # seed dictionary, binds 127.0.0.1:8080
//!   wacha-web --data ../data     # real 62k-word list (uses the trie cache)
//!   wacha-web --data ../data --port 9000

use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use wacha::{Engine, Lookup};

fn main() {
    let mut data_dir: Option<String> = None;
    let mut port: u16 = 8080;
    let mut host: String = String::from("127.0.0.1");
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--data" if i + 1 < args.len() => {
                data_dir = Some(args[i + 1].clone());
                i += 2;
            }
            "--port" if i + 1 < args.len() => {
                port = args[i + 1].parse().unwrap_or_else(|_| {
                    eprintln!("invalid --port; using 8080");
                    8080
                });
                i += 2;
            }
            "--host" if i + 1 < args.len() => {
                host = args[i + 1].clone();
                i += 2;
            }
            "-h" | "--help" => {
                println!(
                    "usage: wacha-web [--data DIR] [--host ADDR] [--port N]\n\n\
                     --host  bind address (default 127.0.0.1, localhost only).\n\
                     \t        For Tailscale access, pass your tailnet IP, e.g.\n\
                     \t        --host 100.76.70.14  (bind only the Tailscale interface),\n\
                     \t        or --host 0.0.0.0 to bind all interfaces.\n\
                     --port  TCP port (default 8080)."
                );
                return;
            }
            other => {
                eprintln!("ignoring unknown arg: {other}");
                i += 1;
            }
        }
    }

    // Build the engine ONCE, before accepting connections.
    eprintln!("building engine…");
    let engine = match &data_dir {
        None => {
            eprintln!("(no --data: using built-in seed dictionary)");
            Engine::seed_only()
        }
        Some(dir) => match Engine::load_from_dir(Path::new(dir), |m| eprintln!("{m}")) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("failed to load engine from {dir}: {err}");
                std::process::exit(1);
            }
        },
    };
    eprintln!(
        "engine ready: {} words | {} entries | {} graph entities, {} triples",
        engine.word_count(),
        engine.entry_count(),
        engine.relation_entity_count(),
        engine.relation_triple_count(),
    );

    let engine = Arc::new(engine);
    // Reverse-dictionary index (Phase C) — built once at startup, shared.
    let t_rev = std::time::Instant::now();
    let rindex = engine.build_reverse_index();
    println!(
        "reverse index: {} docs / {} terms / ~{:.2} MB in {} ms",
        rindex.doc_count(),
        rindex.term_count(),
        rindex.approx_bytes() as f64 / 1e6,
        t_rev.elapsed().as_millis()
    );
    let rindex = Arc::new(rindex);
    // R11 WRITE-2: loose rhyme index — built once at startup, shared.
    let t_rh = std::time::Instant::now();
    let rhyme = engine.build_rhyme_index();
    println!(
        "rhyme index: {} words / {} keys in {} ms",
        rhyme.word_count(),
        rhyme.key_count(),
        t_rh.elapsed().as_millis()
    );
    let rhyme = Arc::new(rhyme);
    let addr = format!("{host}:{port}");
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("could not bind {addr}: {e}");
            std::process::exit(1);
        }
    };
    println!("วาจา (WACHA) web UI listening on http://{addr}  (Ctrl-C to stop)");
    if host != "127.0.0.1" && host != "localhost" {
        println!(
            "note: bound to a non-localhost address ({host}). This server has no auth; \
             only expose it on a trusted network (e.g. your Tailscale tailnet), not the public internet."
        );
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let engine = Arc::clone(&engine);
                let rindex = Arc::clone(&rindex);
                let rhyme = Arc::clone(&rhyme);
                thread::spawn(move || {
                    if let Err(e) = handle(stream, &engine, &rindex, &rhyme) {
                        eprintln!("connection error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
}

fn handle(mut stream: TcpStream, engine: &Engine, rindex: &wacha::reverse::ReverseIndex, rhyme: &wacha::rhyme::RhymeIndex) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    // Parse the request line: METHOD PATH HTTP/1.1
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(()); // client closed
    }
    let mut parts = request_line.split_whitespace();
    let _method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    // Drain remaining headers (we don't need them, but must consume to be a
    // well-behaved HTTP/1.1 peer). Stop at the blank line.
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(v) = trimmed.strip_prefix("Content-Length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    if content_length > 0 {
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body).ok();
    }

    // Route.
    let (status, content_type, body) = route(path, engine, rindex, rhyme);
    let response = format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {len}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Connection: close\r\n\
         \r\n",
        len = body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.write_all(&body)?;
    stream.flush()?;
    Ok(())
}

fn route(path: &str, engine: &Engine, rindex: &wacha::reverse::ReverseIndex, rhyme: &wacha::rhyme::RhymeIndex) -> (&'static str, &'static str, Vec<u8>) {
    if path == "/" || path.starts_with("/?") {
        return ("200 OK", "text/html; charset=utf-8", INDEX_HTML.as_bytes().to_vec());
    }
    if path == "/healthz" {
        return ("200 OK", "text/plain; charset=utf-8", b"ok".to_vec());
    }
    if let Some(qs) = path.strip_prefix("/api/lookup") {
        let query = extract_query_param(qs, "q").unwrap_or_default();
        let json = lookup_json(engine, &query);
        return ("200 OK", "application/json; charset=utf-8", json.into_bytes());
    }
    if let Some(qs) = path.strip_prefix("/api/reverse") {
        let query = extract_query_param(qs, "q").unwrap_or_default();
        let json = reverse_json(engine, rindex, &query);
        return ("200 OK", "application/json; charset=utf-8", json.into_bytes());
    }
    if let Some(qs) = path.strip_prefix("/api/translit") {
        let query = extract_query_param(qs, "q").unwrap_or_default();
        let json = translit_json(engine, &query);
        return ("200 OK", "application/json; charset=utf-8", json.into_bytes());
    }
    if let Some(qs) = path.strip_prefix("/api/evolution") {
        let query = extract_query_param(qs, "q").unwrap_or_default();
        let json = evolution_json(engine, &query);
        return ("200 OK", "application/json; charset=utf-8", json.into_bytes());
    }
    if let Some(qs) = path.strip_prefix("/api/rhyme") {
        let query = extract_query_param(qs, "q").unwrap_or_default();
        let json = rhyme_json(engine, rhyme, &query);
        return ("200 OK", "application/json; charset=utf-8", json.into_bytes());
    }
    if let Some(qs) = path.strip_prefix("/api/register") {
        let reg = extract_query_param(qs, "reg").unwrap_or_default();
        let query = extract_query_param(qs, "q").unwrap_or_default();
        let json = register_json(engine, rindex, &reg, &query);
        return ("200 OK", "application/json; charset=utf-8", json.into_bytes());
    }
    if let Some(qs) = path.strip_prefix("/api/intent") {
        let query = extract_query_param(qs, "q").unwrap_or_default();
        let json = intent_json(engine, &query);
        return ("200 OK", "application/json; charset=utf-8", json.into_bytes());
    }
    ("404 Not Found", "text/plain; charset=utf-8", b"not found".to_vec())
}

/// Intent JSON: `{ "query", "intent" (mode key), "label", "reason", "confidence" }`.
/// Deterministic router (R12) — layer-1 rules, then thai2fit centroid fallback.
fn intent_json(engine: &Engine, query: &str) -> String {
    let g = engine.classify_intent_full(query);
    let conf = match g.confidence {
        wacha::intent::Confidence::Rule => "rule",
        wacha::intent::Confidence::Vector => "vector",
        wacha::intent::Confidence::Default => "default",
    };
    let mut s = String::from("{");
    s.push_str(&format!("\"query\":{},", json_str(query)));
    s.push_str(&format!("\"intent\":{},", json_str(g.intent.mode_key())));
    s.push_str(&format!("\"label\":{},", json_str(g.intent.thai_label())));
    s.push_str(&format!("\"reason\":{},", json_str(&g.reason)));
    s.push_str(&format!("\"confidence\":{}", json_str(conf)));
    s.push('}');
    s
}

/// Rhyme JSON: `{ "query", "rhymes":[word,...] }` (loose rhyme, ranked by freq).
fn rhyme_json(engine: &Engine, rhyme: &wacha::rhyme::RhymeIndex, query: &str) -> String {
    let mut words = rhyme.rhymes_of(query.trim());
    words.sort_by(|a, b| engine.frequency(b).cmp(&engine.frequency(a)).then(a.cmp(b)));
    words.truncate(20);
    let mut s = String::from("{");
    s.push_str(&format!("\"query\":{},", json_str(query)));
    s.push_str("\"rhymes\":[");
    for (i, w) in words.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&json_str(w));
    }
    s.push_str("]}");
    s
}

/// Register JSON: `{ "reg", "query", "words":[{word,freq}] }`. When `query` is set,
/// the register filter is composed with the reverse-dictionary candidate set.
fn register_json(engine: &Engine, rindex: &wacha::reverse::ReverseIndex, reg: &str, query: &str) -> String {
    let words: Vec<(String, u64)> = if query.trim().is_empty() {
        engine.register_search(reg, 20)
    } else {
        // compose: reverse-dictionary candidates filtered to the target register
        let hits = rindex.search(query, 60, |s| engine.segment_words(s));
        let mut v: Vec<(String, u64)> = hits
            .into_iter()
            .filter(|h| engine.word_has_register(&h.word, reg))
            .map(|h| (h.word.clone(), engine.frequency(&h.word)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v.dedup_by(|a, b| a.0 == b.0);
        v.truncate(20);
        v
    };
    let mut s = String::from("{");
    s.push_str(&format!("\"reg\":{},\"query\":{},", json_str(reg), json_str(query)));
    s.push_str("\"words\":[");
    for (i, (w, f)) in words.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&format!("{{\"word\":{},\"freq\":{}}}", json_str(w), f));
    }
    s.push_str("]}");
    s
}

/// Evolution-timeline JSON: `{ "query", "timeline":[{ "edition","definition","is_draft","draft_label" }] }`.
fn evolution_json(engine: &Engine, query: &str) -> String {
    let timeline = engine.evolution_timeline(query);
    let mut s = String::from("{");
    s.push_str(&format!("\"query\":{},", json_str(query)));
    s.push_str("\"timeline\":[");
    for (i, e) in timeline.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let label = if e.is_draft { wacha::evolution::DRAFT_2569_LABEL } else { "" };
        s.push_str(&format!(
            "{{\"edition\":{},\"definition\":{},\"is_draft\":{},\"draft_label\":{}}}",
            json_str(&e.edition),
            json_str(&e.definition),
            e.is_draft,
            json_str(label)
        ));
    }
    s.push_str("]}");
    s
}

/// Transliteration JSON: `{ "query", "source", "hits":[{ "english","thai","note" }] }`.
fn translit_json(engine: &Engine, query: &str) -> String {
    let hits = engine.translit_lookup(query);
    let mut s = String::from("{");
    s.push_str(&format!("\"query\":{},", json_str(query)));
    s.push_str(&format!("\"source\":{},", json_str(wacha::translit::TRANSLIT_SOURCE)));
    s.push_str("\"hits\":[");
    for (i, h) in hits.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"english\":{},\"thai\":{},\"note\":{}}}",
            json_str(&h.english),
            json_str(&h.thai),
            json_str(&h.note)
        ));
    }
    s.push_str("]}");
    s
}

/// Reverse-dictionary JSON: `{ "query", "hits":[{ "word", "score", "matched":[] }] }`.
fn reverse_json(engine: &Engine, rindex: &wacha::reverse::ReverseIndex, query: &str) -> String {
    let hits = rindex.search(query, 8, |s| engine.segment_words(s));
    let mut s = String::from("{");
    s.push_str(&format!("\"query\":{},", json_str(query)));
    s.push_str("\"hits\":[");
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut emitted = 0usize;
    for h in hits.iter() {
        if emitted > 0 {
            s.push(',');
        }
        emitted += 1;
        seen.insert(h.word.clone());
        s.push_str(&format!("{{\"word\":{},\"score\":{:.4},\"source\":\"bm25\",\"matched\":[", json_str(&h.word), h.score));
        for (j, m) in h.matched.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&json_str(m));
        }
        s.push_str("]}");
    }
    // R11 VEC — additional, clearly-labeled candidate source: semantic neighbours
    // (thai2fit cosine) of the query's own tokens. Composed WITH BM25, never
    // replacing it (same graded-confidence discipline as the relation tiers).
    if engine.vector_count() > 0 {
        let mut vec_cands: Vec<(String, f32, String)> = Vec::new();
        for tok in engine.segment_words(query) {
            for (w, sim) in engine.vector_neighbours(&tok, 5) {
                if !seen.contains(&w) && engine.lookup(&w, 0).entry.is_some() {
                    vec_cands.push((w, sim, tok.clone()));
                }
            }
        }
        vec_cands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0)));
        vec_cands.dedup_by(|a, b| a.0 == b.0);
        for (w, sim, via) in vec_cands.into_iter().take(5) {
            if seen.insert(w.clone()) {
                if emitted > 0 {
                    s.push(',');
                }
                emitted += 1;
                s.push_str(&format!(
                    "{{\"word\":{},\"score\":{:.4},\"source\":\"vector\",\"matched\":[{}]}}",
                    json_str(&w), sim, json_str(&format!("≈{via}"))
                ));
            }
        }
    }
    s.push_str("]}");
    s
}

/// Extract and URL-decode a `key=value` query parameter from a `?a=b&c=d` string.
fn extract_query_param(query_string: &str, key: &str) -> Option<String> {
    let qs = query_string.strip_prefix('?').unwrap_or(query_string);
    for pair in qs.split('&') {
        let mut it = pair.splitn(2, '=');
        if it.next() == Some(key) {
            return Some(url_decode(it.next().unwrap_or("")));
        }
    }
    None
}

/// Minimal application/x-www-form-urlencoded decoder (handles %XX and '+').
fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = hex_val(bytes[i + 1]);
                let lo = hex_val(bytes[i + 2]);
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push(h << 4 | l);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Serialize a `Lookup` to JSON by hand (keeps the crate dependency-light — no
/// serde_json). Shapes:
/// {
///   "query": "แมว",
///   "segmentation": [ { "text": "แมว", "in_vocab": true } ],
///   "entry": { "word": "แมว", "pos": "น.", "definition": "…" } | null,
///   "related": [ { "word": "เสือ", "score": 1.573, "path": ["…","…"] } ]
/// }
fn lookup_json(engine: &Engine, query: &str) -> String {
    if query.trim().is_empty() {
        return r#"{"query":"","segmentation":[],"entry":null,"related":[]}"#.to_string();
    }
    let r: Lookup = engine.lookup(query, 8);

    let mut s = String::from("{");
    s.push_str(&format!("\"query\":{},", json_str(query)));

    // segmentation
    s.push_str("\"segmentation\":[");
    for (i, t) in r.segmentation.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"text\":{},\"in_vocab\":{}}}",
            json_str(&t.text),
            t.in_vocab
        ));
    }
    s.push_str("],");

    // entry
    match &r.entry {
        Some(e) => {
            s.push_str(&format!(
                "\"entry\":{{\"word\":{},\"pos\":{},\"definition\":{},",
                json_str(&e.word),
                json_str(&e.pos),
                json_str(&e.definition)
            ));
            // classifiers (ลักษณนาม)
            s.push_str("\"classifiers\":[");
            for (i, c) in e.classifiers.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&json_str(c));
            }
            s.push_str("],");
            // optional สาขาวิชา / register
            match &e.subject {
                Some(sub) => s.push_str(&format!("\"subject\":{},", json_str(sub))),
                None => s.push_str("\"subject\":null,"),
            }
            match &e.register {
                Some(reg) => s.push_str(&format!("\"register\":{},", json_str(reg))),
                None => s.push_str("\"register\":null,"),
            }
            // source + licence badge
            s.push_str(&format!("\"source\":{},", json_str(&e.source)));
            s.push_str(&format!("\"license\":{},", json_str(&e.license)));
            // usage examples (L1) — same provenance/licence as the definition
            s.push_str("\"examples\":[");
            for (i, ex) in e.examples.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&json_str(ex));
            }
            s.push_str("]");
            // รากคำ (etymology): array of {lang, form}
            s.push_str(",\"etymology\":[");
            for (i, (lang, form)) in e.etymology.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&format!("{{\"lang\":{},\"form\":{}}}", json_str(lang), json_str(form)));
            }
            s.push_str("]");
            // ลูกคำ (sub_entries): array of headword strings
            s.push_str(",\"sub_entries\":[");
            for (i, sub) in e.sub_entries.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&json_str(sub));
            }
            s.push_str("]");
            s.push_str("},");
        }
        None => s.push_str("\"entry\":null,"),
    }

    // learner (offline-precomputed enrichment; may be null)
    match &r.learner {
        Some(l) => s.push_str(&format!(
            "\"learner\":{{\"simple\":{},\"example\":{},\"source\":{}}},",
            json_str(&l.simple),
            json_str(&l.example),
            json_str(&l.source)
        )),
        None => s.push_str("\"learner\":null,"),
    }

    // related
    s.push_str("\"related\":[");
    for (i, rw) in r.related.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("{{\"word\":{},", json_str(&rw.word)));
        s.push_str(&format!("\"score\":{:.4},", rw.score));
        s.push_str(&format!("\"source\":{},", json_str(rw.source.as_str())));
        s.push_str(&format!("\"confidence\":{},", json_str(rw.confidence.as_str())));
        s.push_str("\"path\":[");
        for (j, edge) in rw.path.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&json_str(edge));
        }
        s.push_str("]}");
    }
    s.push_str("],");
    // U2 — transliteration dimension of the same query (may be empty)
    s.push_str("\"translit\":[");
    for (i, h) in r.translit.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"english\":{},\"thai\":{},\"note\":{}}}",
            json_str(&h.english),
            json_str(&h.thai),
            json_str(&h.note)
        ));
    }
    s.push_str("],");
    // U2 — evolution timeline of the same query (may be empty)
    s.push_str("\"evolution\":[");
    for (i, e) in r.evolution.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let label = if e.is_draft { wacha::evolution::DRAFT_2569_LABEL } else { "" };
        s.push_str(&format!(
            "{{\"edition\":{},\"definition\":{},\"is_draft\":{},\"draft_label\":{}}}",
            json_str(&e.edition),
            json_str(&e.definition),
            e.is_draft,
            json_str(label)
        ));
    }
    s.push_str("]");
    // R11 SOUND: hard↔soft sound-symbolism profile (always present; UI hedges it)
    match &r.sound {
        Some(sp) => {
            let why = sp.why.iter().map(|w| json_str(w)).collect::<Vec<_>>().join(",");
            s.push_str(&format!(
                ",\"sound\":{{\"score\":{:.2},\"bucket\":{},\"label\":{},\"why\":[{}],\"hedge\":{}}}",
                sp.score, sp.bucket, json_str(sp.label), why, json_str(wacha::sound::HEDGE)
            ));
        }
        None => s.push_str(",\"sound\":null"),
    }
    s.push_str("}");
    s
}

/// JSON-encode a string (quotes + escapes). Thai UTF-8 passes through as-is
/// (valid in JSON).
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

const INDEX_HTML: &str = include_str!("../../web/index.html");
