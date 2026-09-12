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
                thread::spawn(move || {
                    if let Err(e) = handle(stream, &engine) {
                        eprintln!("connection error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
}

fn handle(mut stream: TcpStream, engine: &Engine) -> std::io::Result<()> {
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
    let (status, content_type, body) = route(path, engine);
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

fn route(path: &str, engine: &Engine) -> (&'static str, &'static str, Vec<u8>) {
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
    ("404 Not Found", "text/plain; charset=utf-8", b"not found".to_vec())
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
        Some(e) => s.push_str(&format!(
            "\"entry\":{{\"word\":{},\"pos\":{},\"definition\":{}}},",
            json_str(&e.word),
            json_str(&e.pos),
            json_str(&e.definition)
        )),
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
        s.push_str("\"path\":[");
        for (j, edge) in rw.path.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&json_str(edge));
        }
        s.push_str("]}");
    }
    s.push_str("]}");
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
