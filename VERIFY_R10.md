# VERIFY_R10.md — Round 10 verification report

**Run:** unattended, 2026-09-14, the night before the 2026-09-15 Google Form deadline. Trigger: the
organizer's real data bundle (`nextect.zip`) arrived and is staged git-ignored at `data/official/`.
Rules from R5B–R9, followed exactly: commit per phase, every number from something actually run, no
silent substitution (STOP + document if a phase can't be done cleanly), never touch `katgpt-rs`,
`cargo test` green after every phase. Phase order 0 → U1 → R1 → R2 → R3 → T → E → W → U2 → P → X;
must-do floor 0/U1/R/T. Doc-staleness policy: leave for the final pass; fix only *false* statements.

**Headline:** the product now uses the real ORST assigned data — this closes the single biggest
scoring exposure ("uses zero assigned data" under criterion 2). RID 2554 (11,265 entries / 13,394
senses) is ingested and served locally; the public WASM build is licence-gated to exclude it.

---

## 1. Status table

| Phase | State | Commit | Note |
|---|---|---|---|
| **0** corrections (weights, zero-data) | ✅ internalized (no code) | — | Weights **25/15/20/15/15/10** confirmed from `page5_criteria.png` via catalog §3.1 note. Problem Fit 25% + Innovation 20% = 45%. |
| **U1** etymology + ลูกคำ to display | ✅ done | `9696058` | `EntryView` dropped both; now surfaced in CLI/JSON/UI + clickable ลูกคำ chips. `wacha lookup กรรม` prints รากคำ. +1 test. |
| **R1** reshape DICT_2554 xlsx | ✅ done | `a6797bd` | `reshape_rid_2554.py` → `data/rid/dict_2554.txt`: 11,266 entries (687 w/ SUBENTRIES) from 13,395 rows. |
| **R2** ingest RID | ✅ done | `a6797bd` | Dict 29,772 → 36,305 entries; RID 11,265/13,394. All 5 spot-check words return `RID ๒๕๕๔ (ORST)`. +3 real-data fixes. |
| **R3** licence gate | ✅ done (GATED) | `<this>` | RID **would** reach the public WASM (built from full engine). Added `load_from_dir_opts` public-only path; blobs now embed **0 RID / 0 ศัพท์บัญญัติ** records. See §R3. |
| **T** transliteration assistant | ⬜ pending | — | — |
| **E** specialized-domain terms | ⬜ pending | — | — |
| **W** word-evolution timeline | ⬜ pending | — | — |
| **U2** unified word profile | ⬜ pending | — | — |
| **P** pitch pass | ⬜ pending | — | — |
| **X** dialect navigator | ⏸️ cut-first | — | Lowest leverage; only if time. |
| Deliverables | 🔄 in progress | — | This file + `PROGRESS.md` entry + Status board. |

---

## R3 — Does the real RID excerpt reach the public WASM build? (the answer)

**Question a ราชบัณฑิตยสภา judge could ask live:** the RID 2554 excerpt was handed to teams as an
*unpublished* (ก–ซ only, "for prototype development") excerpt under an ORST-educational framing — is it
compiled into the publicly-deployable `wacha_wasm.wasm`?

**Method (all reproducible):**
- Traced the generators that produce the embedded WASM blobs (same seam as R9 D1):
  - `examples/gen_defs_blob.rs` → `assets/defs.blob` and
  - `examples/gen_reverse_index.rs` → `assets/reverse.idx`
  both built from `Engine::load_from_dir("../data")` — the **full** engine, which since R2 loads
  `data/rid/` at highest priority. **So before this phase, a rebuilt WASM WOULD embed RID definitions**
  (and would still embed ศัพท์บัณฑิตยสภา per R9 D1).
- RID's blob source-code is `3` (`ศัพท์บัญญัติ`=2); before the gate a rebuilt `defs.blob` carried
  thousands of src=3 records.

**Verdict:** the public artifact **must not** carry RID 2554 or ศัพท์บัญญัติ — both are ORST-educational,
non-commercial, with no public-redistribution licence (RID is additionally an unpublished draft excerpt).
This is *stricter* than R9's ศัพท์บัญญัติ finding.

**Fix (the Importer seam, build-time choice):**
- Added `Engine::load_from_dir_opts(dir, include_orst_licensed, log)`. `load_from_dir` = the full
  local/judge build (`include_orst_licensed=true`). The two WASM generators now call it with `false`,
  which skips both `data/rid/` and `data/coined_word_cache/`.
- The public build then rests only on open-licensed sources: PyThaiNLP `words_th.txt` (CC0), Thai
  WordNet (NICT, with notice), Kaikki/Wiktionary (CC BY-SA + attribution).

**Measured proof (regenerated the public `defs.blob` and counted its records):**
```
defined headwords: 29,540
total records = 29,540   RID(src=3) = 0   ศัพท์บัญญัติ(src=2) = 0
```
The local build is unaffected — `wacha lookup กก` still returns `RID ๒๕๕๔ (ORST)`. So: **RID powers the
local/judge demo (the full data story); the public WASM is clean.** Combined with R9 D1, the public deploy
remains tailnet/closed-demo only until an explicit ORST redistribution licence is obtained (one of the
live judge questions in `NEXT_STEPS_R10.md`).

| embedded source | in PUBLIC WASM now | licence | publicly redistributable? |
|---|---|---|---|
| `words_th.txt` (PyThaiNLP) | yes | CC0-1.0 | **Yes** |
| Thai WordNet | yes (relations) | NICT permissive | Yes, with notice |
| Kaikki / Wiktionary | yes (defs, reverse) | CC BY-SA + GFDL | Yes, with attribution + share-alike |
| **RID 2554 (ORST)** | **NO (gated out)** | ORST educational, unpublished draft | **NO** — local/judge build only |
| **ศัพท์บัญญัติ (ORST)** | **NO (gated out)** | ORST educational, non-commercial | **NO** — local/judge build only |
