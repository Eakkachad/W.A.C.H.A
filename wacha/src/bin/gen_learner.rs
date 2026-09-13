//! `gen-learner` — OFFLINE, RUN-ONCE generator for wacha's learner content.
//!
//! This tool is **not** part of the demo or the shipped runtime path. You run it
//! once, offline, on a machine with network + an API key, to (re)generate
//! `wacha/data/learner_content.json` — the static asset the engine then loads
//! with zero runtime LLM dependency (see `learner.rs` / AGENT_HANDOFF.md §1).
//!
//! It targets any **OpenAI-compatible** chat-completions endpoint, which is what
//! Typhoon 2 (SCB 10X, `https://api.opentyphoon.ai/v1`) and most SEA-LION hosts
//! expose. The actual HTTPS call is delegated to `curl` (universally present) so
//! this dev-only tool adds no HTTP-client dependency to the crate.
//!
//! Usage (offline, once):
//!   export WACHA_LLM_API_KEY=sk-...                       # required
//!   export WACHA_LLM_BASE_URL=https://api.opentyphoon.ai/v1   # optional (this is the default)
//!   export WACHA_LLM_MODEL=typhoon-v2-8b-instruct            # optional
//!   cargo run --bin gen-learner -- --out data/learner_content.json
//!
//! Flags:
//!   --out PATH     where to write the JSON asset (default: data/learner_content.json)
//!   --dry-run      print the prompts and exit WITHOUT calling the API (no key needed)
//!   --source LABEL provenance label to stamp on generated entries (default: typhoon-2)
//!
//! Honesty guarantee: entries are stamped with the `--source` label (default
//! `typhoon-2`). Do NOT set it to something the text didn't actually come from.

use std::collections::BTreeMap;
use std::env;
use std::process::Command;

use wacha::dictionary::seed_entries;

const DEFAULT_BASE_URL: &str = "https://api.opentyphoon.ai/v1";
const DEFAULT_MODEL: &str = "typhoon-v2-8b-instruct";

fn main() {
    let mut out_path = String::from("data/learner_content.json");
    let mut dry_run = false;
    let mut source = String::from("typhoon-2");

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--out" if i + 1 < args.len() => {
                out_path = args[i + 1].clone();
                i += 2;
            }
            "--source" if i + 1 < args.len() => {
                source = args[i + 1].clone();
                i += 2;
            }
            "--dry-run" => {
                dry_run = true;
                i += 1;
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => {
                eprintln!("unknown arg: {other}");
                print_help();
                std::process::exit(2);
            }
        }
    }

    let base_url = env::var("WACHA_LLM_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let model = env::var("WACHA_LLM_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let api_key = env::var("WACHA_LLM_API_KEY").ok();

    let words: Vec<(String, String)> = seed_entries()
        .into_iter()
        .map(|e| {
            let def = e.primary_definition().unwrap_or("").to_string();
            (e.headword, def)
        })
        .collect();

    eprintln!(
        "gen-learner: {} seed words | model={model} | endpoint={base_url} | source-label={source}{}",
        words.len(),
        if dry_run { " | DRY RUN" } else { "" }
    );

    if !dry_run && api_key.is_none() {
        eprintln!(
            "error: WACHA_LLM_API_KEY is not set. Set it, or use --dry-run to preview prompts.\n\
             (This tool is meant to be run offline/once; the demo never calls an LLM.)"
        );
        std::process::exit(1);
    }

    // BTreeMap → stable, sorted key order in the output JSON (deterministic diffs).
    let mut entries: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    for (word, formal_def) in &words {
        let prompt = build_prompt(word, formal_def);
        if dry_run {
            println!("\n=== {word} ===\n{prompt}");
            continue;
        }
        match call_llm(&base_url, &model, api_key.as_deref().unwrap(), &prompt) {
            Ok((simple, example)) => {
                eprintln!("  ✓ {word}");
                entries.insert(
                    word.clone(),
                    serde_json::json!({
                        "simple": simple,
                        "example": example,
                        "source": source,
                    }),
                );
            }
            Err(e) => {
                eprintln!("  ✗ {word}: {e} (skipping — will keep any existing entry)");
            }
        }
    }

    if dry_run {
        eprintln!("dry run complete — no API calls made, no file written.");
        return;
    }

    // Merge over the existing asset so a partial run doesn't destroy prior work.
    let merged = merge_with_existing(&out_path, entries, &source, &model, &base_url);
    match std::fs::write(&out_path, serde_json::to_string_pretty(&merged).unwrap()) {
        Ok(()) => eprintln!("wrote {out_path}"),
        Err(e) => {
            eprintln!("error writing {out_path}: {e}");
            std::process::exit(1);
        }
    }
}

/// Build the Thai instruction prompt for one word.
fn build_prompt(word: &str, formal_def: &str) -> String {
    format!(
        "คุณเป็นผู้ช่วยจัดทำสื่อการเรียนภาษาไทยสำหรับผู้เริ่มต้นและเด็ก\n\
         คำ: \"{word}\"\n\
         นิยามทางการ: \"{formal_def}\"\n\n\
         โปรดสร้าง JSON เพียงหนึ่งอ็อบเจกต์ รูปแบบ:\n\
         {{\"simple\": \"คำอธิบายง่าย ๆ ที่เด็กเข้าใจ ไม่เกิน 1-2 ประโยค\", \
         \"example\": \"ประโยคตัวอย่างที่ใช้คำนี้อย่างเป็นธรรมชาติ 1 ประโยค\"}}\n\
         ตอบเป็น JSON ล้วน ๆ ไม่ต้องมีข้อความอื่น"
    )
}

/// Call the OpenAI-compatible chat-completions endpoint via `curl`, returning
/// (simple, example) parsed from the model's JSON reply.
fn call_llm(base_url: &str, model: &str, api_key: &str, prompt: &str) -> Result<(String, String), String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": prompt }],
        "temperature": 0.4,
        "max_tokens": 300,
    });
    let body_str = serde_json::to_string(&body).map_err(|e| e.to_string())?;

    let output = Command::new("curl")
        .arg("-sS")
        .arg("--fail-with-body")
        .arg(&url)
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-H")
        .arg(format!("Authorization: Bearer {api_key}"))
        .arg("-d")
        .arg(&body_str)
        .output()
        .map_err(|e| format!("failed to spawn curl: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "curl failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let resp: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("bad JSON response: {e}"))?;
    let content = resp["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("response missing choices[0].message.content")?;

    parse_model_json(content)
}

/// Parse the model's reply (expected to be a JSON object with simple/example).
/// Tolerates the model wrapping the JSON in prose or ```json fences.
fn parse_model_json(content: &str) -> Result<(String, String), String> {
    let start = content.find('{').ok_or("no JSON object in model reply")?;
    let end = content.rfind('}').ok_or("no closing brace in model reply")?;
    let json_slice = &content[start..=end];
    let v: serde_json::Value =
        serde_json::from_str(json_slice).map_err(|e| format!("model JSON parse: {e}"))?;
    let simple = v["simple"].as_str().unwrap_or("").trim().to_string();
    let example = v["example"].as_str().unwrap_or("").trim().to_string();
    if simple.is_empty() {
        return Err("model reply had empty 'simple' field".into());
    }
    Ok((simple, example))
}

/// Merge freshly-generated entries over any existing asset, preserving the
/// `_meta` block (updated) and entries not regenerated this run.
fn merge_with_existing(
    out_path: &str,
    fresh: BTreeMap<String, serde_json::Value>,
    source: &str,
    model: &str,
    base_url: &str,
) -> serde_json::Value {
    let mut entries_obj = serde_json::Map::new();

    // Start from existing entries if the file already exists.
    if let Ok(existing_str) = std::fs::read_to_string(out_path) {
        if let Ok(existing) = serde_json::from_str::<serde_json::Value>(&existing_str) {
            if let Some(obj) = existing.get("entries").and_then(|e| e.as_object()) {
                for (k, v) in obj {
                    entries_obj.insert(k.clone(), v.clone());
                }
            }
        }
    }
    // Overlay fresh entries.
    for (k, v) in fresh {
        entries_obj.insert(k, v);
    }

    serde_json::json!({
        "_meta": {
            "description": "Learner-facing enrichment for wacha seed words: a plain-language explanation (คำอธิบายง่าย) and one example sentence per word. Loaded at runtime as a static asset — the demo NEVER calls an LLM.",
            "generated_by": format!("gen-learner (model={model}, endpoint={base_url}, source-label={source})"),
            "schema": {
                "simple": "plain-language explanation a Thai learner/child could understand",
                "example": "one natural example sentence using the word",
                "source": "provenance of THIS entry's text — 'human_seed', 'typhoon-2', or 'sea-lion'"
            }
        },
        "entries": serde_json::Value::Object(entries_obj)
    })
}

fn print_help() {
    eprintln!(
        "gen-learner — OFFLINE, run-once generator for wacha/data/learner_content.json\n\n\
         Env:\n\
         \tWACHA_LLM_API_KEY   (required unless --dry-run)\n\
         \tWACHA_LLM_BASE_URL  (default {DEFAULT_BASE_URL})\n\
         \tWACHA_LLM_MODEL     (default {DEFAULT_MODEL})\n\n\
         Flags:\n\
         \t--out PATH      output JSON asset (default data/learner_content.json)\n\
         \t--source LABEL  provenance label (default typhoon-2)\n\
         \t--dry-run       print prompts, make NO API calls, write nothing\n\
         \t-h, --help      this help\n\n\
         The demo/runtime never calls an LLM — this tool only regenerates the static asset."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_json() {
        let (s, e) = parse_model_json(r#"{"simple":"ง่าย","example":"ตัวอย่าง"}"#).unwrap();
        assert_eq!(s, "ง่าย");
        assert_eq!(e, "ตัวอย่าง");
    }

    #[test]
    fn parses_fenced_json_with_prose() {
        let reply = "นี่คือคำตอบ:\n```json\n{\"simple\": \"อธิบาย\", \"example\": \"ประโยค\"}\n```\nหวังว่าจะช่วยได้";
        let (s, e) = parse_model_json(reply).unwrap();
        assert_eq!(s, "อธิบาย");
        assert_eq!(e, "ประโยค");
    }

    #[test]
    fn rejects_reply_without_simple() {
        assert!(parse_model_json(r#"{"example":"x"}"#).is_err());
        assert!(parse_model_json("no json here").is_err());
    }

    #[test]
    fn prompt_includes_word_and_definition() {
        let p = build_prompt("แมว", "สัตว์เลี้ยง");
        assert!(p.contains("แมว"));
        assert!(p.contains("สัตว์เลี้ยง"));
        assert!(p.contains("simple") && p.contains("example"));
    }
}
