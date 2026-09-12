# วาจา (WACHA) — API & Open Data

This documents the HTTP JSON contract of `wacha-web` and the license of every data asset วาจา ships, so a
third party (researcher, app developer, another ORST team) can realistically build on it. This is the
concrete answer to the hackathon brief's **"promote open data"** and **"build networks"** objectives:
open, CC0/permissively-licensed Thai lexical data behind a small stable JSON API.

Everything here is served by a dependency-free `std::net` server — no framework, no database, no cloud
service required to self-host.

---

## Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/` | The single-page web UI (HTML). |
| `GET` | `/api/lookup?q=<word>` | The lookup contract below. `q` is URL-encoded UTF-8 Thai. |
| `GET` | `/healthz` | Liveness check — returns `ok` (text/plain). |

Run it: `cargo run --release --bin wacha-web -- --data ../data [--host ADDR] [--port N]`
(default bind `127.0.0.1:8080`). All responses set `Access-Control-Allow-Origin: *`.

---

## `GET /api/lookup?q=<word>` — the contract

Returns a single JSON object. **Spot-checked against the live server on 2026-09-12** (`q=แมว`, related
truncated to 2 for brevity):

```json
{
  "query": "แมว",
  "segmentation": [
    { "text": "แมว", "in_vocab": true }
  ],
  "entry": {
    "word": "แมว",
    "pos": "น.",
    "definition": "สัตว์เลี้ยงลูกด้วยนมชนิดหนึ่ง เลี้ยงไว้ในบ้าน จับหนูเป็นอาหาร"
  },
  "learner": {
    "simple": "สัตว์สี่ขาตัวเล็ก มีขนนุ่ม ร้องเหมียว ๆ คนนิยมเลี้ยงไว้ในบ้าน",
    "example": "แมวของฉันชอบนอนกลางวันแล้วออกมาเล่นตอนกลางคืน",
    "source": "human_seed"
  },
  "related": [
    {
      "word": "แมวบ้าน",
      "score": 8.3572,
      "source": "wordnet",
      "confidence": "confirmed",
      "path": [
        "แมว --มีความหมายเหมือนกับ--> แมวบ้าน",
        "แมวบ้าน --มีความหมายเหมือนกับ--> แมว"
      ]
    },
    {
      "word": "เสือ",
      "score": 8.3085,
      "source": "seed",
      "confidence": "confirmed",
      "path": [
        "เสือ --ดูเพิ่มที่--> แมว",
        "แมว --ดูเพิ่มที่--> เสือ"
      ]
    }
  ]
}
```

### Fields

| Field | Type | Meaning |
|---|---|---|
| `query` | string | The (trimmed) query as received. Empty string if `q` is empty/absent. |
| `segmentation` | array | The query split into tokens by the Datrie longest-match segmenter. |
| `segmentation[].text` | string | One token (a dictionary word, or a whole Thai Character Cluster for OOV). |
| `segmentation[].in_vocab` | bool | `true` = a known dictionary word; `false` = out-of-vocabulary cluster. |
| `entry` | object \| null | The dictionary entry if `query` is exactly one known headword; else `null`. |
| `entry.word` / `pos` / `definition` | string | Headword, part of speech, formal definition. |
| `learner` | object \| null | Offline-precomputed learner enrichment for `query`, if any; else `null`. |
| `learner.simple` | string | Plain-language explanation (คำอธิบายง่าย). |
| `learner.example` | string | One example sentence. |
| `learner.source` | string | Provenance: `human_seed` (hand-authored) or `typhoon-2`/`sea-lion` (offline LLM). **Never generated at request time.** |
| `related` | array | Related words, ranked by relevance (may be empty). |
| `related[].word` | string | The related word. |
| `related[].score` | number | Relative-PPR relevance (higher = more related). **Only comparable within one response** — not across queries or data versions. |
| `related[].source` | string | `seed` = hand-verified relation; `wordnet` = auto-extracted from Thai WordNet (unaudited). |
| `related[].confidence` | string | `confirmed` (seed, or WordNet corroborated by ≥2 synsets) or `unverified` (isolated WordNet pair — not cross-corroborated; ~84.2% of WordNet pairs are genuine synonyms, see PROGRESS.md 2026-09-12). |
| `related[].path` | array of string | Human-readable relation edges explaining *why* the words connect (`A --relation--> B`). |

### Guarantees & edge behavior (verified 2026-09-12, Task 9)
- Always HTTP `200` with a well-formed object for `/api/lookup`, including empty `q`, non-Thai input,
  emoji, very long strings, invalid UTF-8 (decoded to `�`), and injection-looking strings (returned as
  escaped JSON string data — safe). Unknown paths return `404`.
- **Deterministic:** the same query returns the same result (no model inference at request time).
- **Fast:** microsecond-scale per query after a one-time engine build at startup (~43s cold, ~10ms from
  the on-disk cache — build once at startup, never per request).

---

## Data assets & licenses

Every derived data asset วาจา ships is open and reusable. Provenance is tracked in-code and in
`PROGRESS.md`; this table is the reuse-facing summary.

| Asset | What | Size | License | Source / provenance |
|---|---|---|---|---|
| `data/words_th.txt` | 62,107 Thai words (drives segmentation) | ~1.5 MB | **CC0-1.0** (public domain) | PyThaiNLP `words_th.txt`, from NECTEC LEXiTRON. No attribution required. |
| `data/tnc_freq.txt` | Word frequencies (ranking) | ~1.5 MB | **CC0-1.0** | PyThaiNLP `tnc_freq.txt` (Thai National Corpus). |
| `wacha/data/wordnet_synonyms.tsv` | 13,664 synonym groups (~29k words) | ~1 MB | **NICT permissive** (free use/copy/modify/distribute with copyright notice) | Derived from Thai WordNet `wordnet_th.db` (Thai Computational Linguistic Laboratory / NICT). We ship this ~1 MB derived TSV, not the 11 MB source DB. Only genuine synonym relations extracted; no is-a hierarchy fabricated. |
| `wacha/data/learner_content.json` | Learner คำอธิบายง่าย + examples (20 seed words) | ~8 KB | text is **hand-authored** (`source: human_seed`); regenerable offline via `gen-learner` (then `typhoon-2`) | Not AI-generated at runtime; provenance honestly labeled per entry. |

### Vendored code provenance (not data, but for completeness)
- `wacha/src/datrie.rs` — vendored from `katgpt-tokenizer` (MIT), with a panic-fix + serde support added here.
- `wacha/src/graph.rs` — vendored from AXIOM's `tle-axiom-gen` (`KnowledgeGraph`: triple-store + PPR + BFS).

---

## How a third party would reuse this
1. **Just the data:** take `words_th.txt` (CC0) as a Thai word list / segmentation dictionary, or
   `wordnet_synonyms.tsv` (NICT) as a Thai synonym network — no code needed.
2. **The API:** self-host `wacha-web` (single static binary + the `data/` dir) and call `/api/lookup` as a
   JSON microservice from any language. No GPU, no external service, works offline.
3. **The library:** depend on the `wacha` crate and use `Engine::load_from_dir(...).lookup(word, k)`
   directly.

If ORST publishes official RID data, it slots in at the `Entry`/`Relation` model (see
`wacha/src/dictionary.rs`) without architectural change.
