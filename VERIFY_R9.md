# VERIFY_R9.md — Round 9 verification report

**Run:** unattended, 2026-09-14. Rules from R5B–R9: commit per task, every number from a script, no silent
substitution (STOP + document if a task can't be done cleanly), do not touch `katgpt-rs`, finish fewer
tasks cleanly over many half-done. Doc-staleness policy: leave it for the final pass; fix only statements
that are actually *false*.

---

## 1. Status table

| Task | State | Commit | Note |
|---|---|---|---|
| **G1** ranking-regression guard | ✅ done | `517ce0c` | `wacha rankguard` p@5 over KEEP gold set; verify_r5.sh §10, threshold 50%. Failure path demonstrated. |
| **G2** deadline on `symbol_trie.rs` | ⏳ policy | — | Enforced at Q3: if Q3 fails its gates, delete `symbol_trie.rs` this round. |
| **D1** WASM licence audit | ✅ done (STOP) | (this) | **ORST ศัพท์บัญญัติ content IS embedded** → no public deploy. See §D1 below. |
| **D2** phone deploy + test | ⚠️ constrained | — | D1 forces **tailnet-only**; a real public-network phone test can't be driven unattended. See §D2. |
| **Q1** maximal matching | — | — | pending |
| **Q2** reverse dictionary v2 | — | — | pending |
| **Q3** S1b final attempt | — | — | pending (droppable) |
| **X1** Wikidata Lexemes | — | — | pending (droppable) |

---

## D1 — Licence audit of the WASM artifact (the answer)

**Question:** is ORST ศัพท์บัญญัติ content (no redistribution licence stated on the source site — only
"educational, non-commercial") compiled into `wacha_wasm.wasm`, making a public deploy a redistribution
of it?

**Method (all reproducible):**
- Traced the three generators that produce the embedded blobs:
  - `examples/gen_defs_blob.rs` → `assets/defs.blob`: built from **`Engine::load_from_dir`** — the
    **full** engine, which runs `CoinedWordImporter` over `data/coined_word_cache/`. **Includes ศัพท์บัญญัติ.**
  - `examples/gen_reverse_index.rs` → `assets/reverse.idx`: also from **`Engine::load_from_dir`** (full
    engine). Indexes every definition, **including the ศัพท์บัญญัติ ones.**
  - `examples/gen_pagerank_blob.rs` → `assets/pagerank.blob`: built from **seed entries only**
    (`build_from_segmenter`, no `coined_word_cache`). **No ศัพท์บัญญัติ.**
- Counted directly in the shipped blob:
  ```
  defs.blob: 29,601 entries, 113 with src_code=2 (CoinedWord/ศัพท์บัญญัติ)
  ```
- Confirmed a sample is ORST-labeled: `lookup ขั้นตอนวิธี` →
  `นิยาม: algorithm (ศัพท์บัญญัติ · คอมพิวเตอร์…)` · `ที่มานิยาม: ศัพท์บัญญัติ (ORST) · ORST (educational, non-commercial)`.

**Per-source verdict for a PUBLIC deploy (redistribution):**

| embedded source | in artifact | licence | publicly redistributable? |
|---|---|---|---|
| `words_th.txt` (LEXiTRON) | yes (WORDS_TH + seg cache) | CC0-1.0 | **Yes** |
| Thai WordNet | yes (relations) | NICT permissive | Yes, **with the NICT copyright notice** |
| Kaikki / Wiktionary defs | yes (defs.blob, reverse.idx) | CC BY-SA + GFDL | Yes, **with attribution + share-alike stated** |
| **ศัพท์บัญญัติ (ORST)** | **YES — 113 defs in defs.blob + reverse.idx + relation graph** | **"educational, non-commercial" only; no redistribution licence** | **NO** |

**Verdict: STOP — do not deploy the current artifact publicly.** The ศัพท์บัญญัติ layer is embedded and
its stated terms ("educational, non-commercial") are not a redistribution licence. Options (not executed
this round; the `Importer` seam makes the first a clean build-time choice):
1. **Public variant with the ORST layer excluded** — regenerate `defs.blob`/`reverse.idx` from an engine
   built *without* `coined_word_cache`, drop the CoinedWord relations, and ship that. The Wiktionary layer
   then needs a visible CC BY-SA attribution + the NICT WordNet notice on the page.
2. **Keep the full-data build private/unlisted (tailnet only)** and demo from a device on the tailnet.

This round takes option 2 for D2 (below) — it needs no data changes and keeps the demo intact. Building the
ORST-excluded public variant is a clean follow-up, not attempted here to avoid shipping a half-done deploy.
