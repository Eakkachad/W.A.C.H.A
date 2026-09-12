# วาจา (WACHA)

**W**ord **A**rchitecture, **C**luster-aware **H**ybrid **A**nalysis — a hybrid Thai dictionary engine for
the ORST **"เปิดคลังคำ พลิกคลังคิด"** hackathon. วาจา is a real Thai word for "speech / word / utterance"
(as in สัตย์วาจา, "word of honor"). Every letter of the backronym maps to something real and verified in
this codebase, not a marketing label — see the breakdown below.

Two of the user's own research assets, each used for exactly what it's good at:

- **Segmentation** — the vendored `Datrie` (double-array trie, `datrie.rs`) drives greedy longest-match
  Thai word segmentation, built directly from the dictionary's own word list. *The dictionary is the
  tokenizer.* Out-of-vocabulary text falls back to whole **Thai Character Clusters** (`tcc.rs`), never
  raw codepoints — so tone marks and leading vowels are never orphaned. Originally `katgpt-tokenizer`
  (from the user's own `katgpt-rs` repo); vendored in (2026-09-12, MIT-licensed) rather than kept as a
  live path dependency, since `katgpt-rs` isn't this project's repo to depend on for correctness-critical
  fixes — see `datrie.rs`'s module doc for exactly what was fixed and why.
- **Explainable relationships** — AXIOM's vendored `KnowledgeGraph` (`graph.rs`: triple-store +
  Personalized PageRank + BFS) over triples extracted from dictionary entries answers *"which words are
  related, and why."* Only structured relations the entries explicitly carry are used (synonym / antonym
  / is-a / see-also / category); AXIOM's English-only text-NLU layer is never touched.

## The backronym, mapped to real code

| Letter | Stands for | Where it lives |
|---|---|---|
| **W** | Word | `segmenter.rs` — dictionary-word-driven segmentation |
| **A** | Architecture | `Datrie` double-array trie (Aoe, 1989) — a literal node hierarchy |
| **C** | Cluster-aware | `tcc.rs` — the Thai Character Cluster OOV fix (a real bug found and fixed) |
| **H** | Hybrid | katgpt-rs's tokenizer + AXIOM's vendored `graph.rs`, combined |
| **A** | Analysis | `relations.rs`/`graph.rs` — Personalized PageRank + BFS explainable ranking |

## Run

```bash
# Interactive REPL on the built-in seed dictionary (no data files needed):
cargo run

# Full journey on the real 62,107-word CC0 list:
cargo run -- --data ../data lookup แมว
cargo run -- --data ../data segment "นักเรียนอ่านหนังสือที่โรงเรียน"
cargo run -- --data ../data stats

# Tests:
cargo test
```

`--data DIR` expects `words_th.txt` (one word per line) and optionally `tnc_freq.txt` (word⇥count) — the
CC0 PyThaiNLP corpora in `../data/`.

### Trie cache (first run is slow, the rest are instant)

Building the double-array trie from the full 62k-word list takes **~43 seconds** — a one-time cost caused
by Thai's narrow UTF-8 byte range triggering heavy trie-collision cascades (see `../PROGRESS.md`
2026-09-05). To avoid paying it on every run, the `--data` path **caches the built segmenter** to
`DIR/words_th.datrie.cache` (~7 MB, git-ignored) and reloads it in **~10 ms** on subsequent runs:

- First run: builds the trie (~43s), writes the cache.
- Later runs: loads the cache (~10ms, a ~4,400× speedup). Total process time ~0.02s.
- The cache is keyed on the word-list file's mtime — editing `words_th.txt` automatically invalidates it
  and forces a rebuild.

This makes even one-shot `lookup`/`segment` invocations fast, not just the long-lived REPL. Delete the
`.datrie.cache` file to force a clean rebuild.

## The user journey

`type a word → segment it → look up its definition → see related words, each with an explanation`

```
──────── ครู ────────
segmented: ครู
คำ: ครู  (น.)
นิยาม: ผู้สั่งสอนศิษย์; ผู้ถ่ายทอดความรู้
คำที่เกี่ยวข้อง (related words, ranked; with explanation):
  1. อาจารย์   (score 1.440)   ↳ ครู --มีความหมายเหมือนกับ--> อาจารย์
  2. โรงเรียน  (score 1.216)   ↳ ครู --ดูเพิ่มที่--> โรงเรียน
  3. นักเรียน  (score 1.216)   ↳ ครู --ตรงข้ามกับ--> นักเรียน
  ...
```

## Modules

| Module | Role |
|---|---|
| `tcc` | Thai Character Cluster grouping (the OOV bug fix) |
| `segmenter` | `Datrie` longest-match + TCC-aware OOV fallback |
| `dictionary` | `Entry` model, word-list + frequency loading, curated seed entries |
| `graph` | vendored AXIOM `KnowledgeGraph` (single file, verbatim) |
| `relations` | triple extraction + explainable `related(word, k)` query |
| `lib` (`Engine`) | facade stitching the full journey together |
| `bin/cli` | REPL + `lookup` / `segment` / `stats` subcommands |

## Data & licenses

- `words_th.txt` — 62,107 Thai words, **CC0-1.0** (PyThaiNLP, from NECTEC LEXiTRON).
- `tnc_freq.txt` — Thai National Corpus frequencies, **CC0-1.0** (PyThaiNLP).
- `graph.rs` — vendored from AXIOM (`neural-engines/AXIOM/crates/tle-axiom-gen/src/graph.rs`).
- `datrie.rs` — vendored from `katgpt-tokenizer` (`katgpt-rs/crates/katgpt-tokenizer/src/datrie.rs`),
  MIT-licensed, with two fixes made here (a real panic bug + serde support for the trie cache) — see the
  file's own module doc.

See `../AGENT_HANDOFF.md` for the full architecture rationale and guardrails.
