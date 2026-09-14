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
| **R3** licence gate | ✅ done (GATED) | `481ccf8` | RID **would** reach the public WASM (built from full engine). Added `load_from_dir_opts` public-only path; blobs now embed **0 RID / 0 ศัพท์บัญญัติ** records. See §R3. |
| **T** transliteration assistant | ✅ done | `50f382f` | ORST official คำทับศัพท์ (2,256 pairs), both directions. 5 round-trips verified. See §T. |
| **E** specialized-domain terms | ✅ done | `f1acb35` | 3 dictionaries → 4,986 Thai terms / 6,252 senses; cross-discipline links (96 shared English headwords). See §E. |
| **W** word-evolution timeline | ✅ done | `62c447d` | ก only, 2542→2554→2569; 693 headwords in all 3 editions, 686 differ; verbatim 2569 draft label. See §W. |
| **U2** unified word profile | ✅ done | `1018ac0` | One `lookup` returns def+related+etymology+ลูกคำ+translit+evolution. See §U2. |
| **P** pitch pass | ✅ done | `cb677f8` | Data-usage line, evolution→Innovation headline, one-search framing, fixed false RID/coverage claims; verify_pitch re-run PASS. See §P. |
| **X** dialect navigator | ⏹️ CUT | — | Cut per plan ("cut first if short on time") — lowest leverage, legacy .doc parse. See §X. |
| Deliverables | ✅ done | `<this>` | This file + `PROGRESS.md` entry + Status board. |

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

---

## U1 — Etymology + ลูกคำ surfaced (the cheapest real win)

`Entry` already carried `etymology` (รากคำ) and `sub_entries` (ลูกคำ) — parsed by RidImporter/Kaikki —
but `EntryView` (the display projection) dropped both, so no CLI/web surface ever showed them. Added both
fields to `EntryView`, populated from the matched `Entry`, and rendered them in `print_lookup` (CLI),
`lookup_json` (web JSON) and `render()` (web — ลูกคำ as clickable cross-navigation chips). New test
`etymology_and_sub_entries_reach_entry_view` guards the *display layer*, not just parsing.

**Verified:** `wacha lookup กรรม/บิดา/แมว` print a รากคำ line from real Kaikki data; `/api/lookup?q=กรรม`
returns non-empty `etymology`. 94 tests pass.

---

## R — Ingest the real RID ๒๕๕๔ excerpt (closes "uses zero assigned data")

**R1:** `reshape_rid_2554.py` turns `DICT_2554 (ก_ซ).xlsx` (13,395 rows, git-ignored raw) into
`data/rid/dict_2554.txt` — the block grammar `RidImporter` already parses. **11,266 entries** (687 with
`SUBENTRIES`, grouped by the `แม่คำ_0 ; ลูกคำ_1` parent-reference column). Homographs from `headword_sense`
(๑/๒/…); `สัน.` normalized to `สัน` (the one Pos marker that disagreed with the source); scientific-name
HTML italics stripped.

**R2:** `RidImporter` auto-loads `data/rid/` at highest merge priority. Dictionary grew **29,772 → 36,305
entries**; RID contributes **11,265 entries / 13,394 senses**. All 5 `COMPETITION_DAY` spot-check words
(ก, กก, กระดาษ, ก็, เขียน) return `RID ๒๕๕๔ (ORST)`.

**Three real-data quality bugs the rank guard + ingestion surfaced (all fixed, not tuned around):**
1. **Homograph shadowing.** `Dictionary::get` preferred the no-homograph key, so 644 common RID words
   (กรรม, กรม, กรด, …) listed ONLY as homographs were shadowed by a plain Kaikki entry. Now `get` prefers
   the highest-priority *source* across homographs (RID=5). Regression test added.
2. **Fragile load.** One malformed block aborted the whole 11k-entry load. Now it skips individual bad
   blocks (explicit error count, not `total − entries` which mis-counted `Ok(None)`), hard-failing only
   above 5%. 1/11,266 skipped.
3. **Greedy `ดู` cross-references.** The importer grabbed whole prose tails (`ยาม).`) and parenthetical
   `(ดู X)` supplementary pointers as first-class SeeAlso relations, displacing audited synonyms and
   dropping the rank guard to **48.7%**. Now it takes only the first clean Thai token, and parenthetical
   refs stay in `see_also` (display) without becoming ranked relations. **Rank guard restored to 51.3%.**

The 48.7%→51.3% episode is exactly what the R9 guard exists to catch — a silent ranking regression from
new data, caught and root-caused before commit rather than shipped.

---

## R3 — see the licence-gate section above (§R3).

---

## T — Transliteration assistant (ORST official คำทับศัพท์)

`reshape_translit.py` → `data/translit.tsv` (**2,256** English↔Thai pairs from
`คำทับศัพท์ที่ใช้บ่อย.xlsx`). New `wacha/src/translit.rs`: a bidirectional `BTreeMap` index; direction
auto-detected by ASCII letters. `Engine.translit` loaded from `data/translit.tsv` — an **open** ORST
reference (ประมวลคำทับศัพท์ พ.ศ. 2563), so present in BOTH the full and public builds (unlike RID). CLI
`wacha translit <term>`; web `/api/translit?q=` + a คำทับศัพท์ toggle.

**Verified:** 5 round-trips both directions (computer↔คอมพิวเตอร์, internet↔อินเทอร์เน็ต, digital↔ดิจิทัล,
calculus↔แคลคูลัส, graphic↔กราฟิก) via unit test `five_round_trips_both_directions` AND live CLI + HTTP.

---

## E — Specialized-domain cross-discipline terms

3 ORST dictionaries (ศัพท์จิตวิทยา / ปรัชญา / แพทยศาสตร์, 7,241 source rows). `reshape_specialized.py` →
`data/specialized_terms.tsv`; a thin `SpecializedImporter` builds the same `Entry` shape as
`CoinedWordImporter`, gated behind `include_orst_licensed` (excluded from public WASM).

**HONEST numbers (the real extraction, not the aspirational 7,241):** **6,252 (english, thai) rows →
4,986 distinct Thai coined terms / 6,252 senses.** The ปรัชญา file has only 348 non-empty coinage cells of
its 1,600 rows — the rest are genuinely blank in the source, so they are not counted. Dictionary grew
**36,305 → 40,681**.

**Cross-discipline link:** the dedicated column `ศัพท์อื่นที่เป็นคำเดียวกันในต่างสาขา` is EMPTY in all
three files (measured 0/7,241), so it cannot be used. Instead the genuine link is derived from the data:
**96 English headwords are coined in more than one discipline**, and those Thai terms get a cross-discipline
`RelatedTo` edge. Verified: ความสามารถ→"ability (จิตวิทยา)", อภิธรรม→"abhidharma (ปรัชญา)", and
อปรกติ (จิตวิทยา) → ผิดปรกติ (แพทยศาสตร์) surfaced as a related word (same English "abnormal").

---

## W — Word Evolution Timeline (ก only) — the Innovation beat

The feature an LLM-wrapper cannot fake (it needs the actual text of three editions to diff).
`reshape_evolution.py` keys the ก-slice of DICT_2542 (3,136 ก-headwords), the Phase-R DICT_2554 block file,
and the DICT_2569 draft into `data/evolution_ko.tsv`. New `wacha/src/evolution.rs`: chronological timeline
per headword, gated behind `include_orst_licensed`. CLI `wacha evolution <hw>` + `/api/evolution?q=` + a
วิวัฒนาการคำ web panel.

**MANDATORY integrity:** every 2569 row carries the ORST caveat VERBATIM —
"ร่าง อยู่ระหว่างดำเนินการ ยังไม่เป็นข้อมูลทางการ" (a test pins the exact string; CLI prints ⚠, web shows a
`.draft` line).

**Measured:** **3,204 distinct ก-headwords** across editions; **693 present in all three**; **686 differ**
in wording between 2542 and 2569. Honest per-edition usable-def counts: 2542=2,770 / 2554=3,017 / 2569=784
(2569 is an incomplete draft). Verified on ก, ก กา, ก ข ไม่กระดิกหู — three visibly-different editions.

---

## U2 — Unified Word Profile (mentor #1: one search, many dimensions)

No new data or algorithms — only what one lookup returns and how one page renders. `Lookup` extended with
`translit` + `evolution` (both empty for most words, zero cost when absent); `Engine::lookup` populates them
from the same single call. CLI `print_lookup` adds "คำทับศัพท์ที่เกี่ยวข้อง" + "พจนานุกรม ๓ ยุค" sections;
web `render()` adds two cards under the definition card. **Still ONE search box, no mode switcher.**

**Verified acceptance — one `/api/lookup?q=กระดาษ` call returns:** definition (RID ๒๕๕๔) + etymology (รากคำ)
+ 12 ลูกคำ + evolution across all 3 editions (2569 draft label present) — four dimensions from a single
query. A loanword query (คอมพิวเตอร์) additionally lights the transliteration section.

---

## P — Pitch pass (and the R6/R7 silent-break guard, honored)

Applied to the real deck files (`PITCH_DECK.md` / `PITCH.md` / `BIBLE.md`):
- **Data-usage line** on slide 2 (pre-empts "did you use our data?"): states the ingested assigned sets —
  RID ๒๕๕๔ (13,395), คำทับศัพท์ (2,256), ศัพท์เฉพาะทาง ๓ สาขา, + วิวัฒนาการ ๓ ยุค — merged with open CC0.
- **One-search-many-dimensions** framing on slide 2 (Phase U2), grounded in real product sections.
- **Word Evolution Timeline promoted** to the Innovation headline (slide 9) with real numbers (693/686) and
  the "LLM-wrapper can't diff 3 editions" point — Innovation is the 20% (2nd-heaviest) criterion.
- **Fixed now-FALSE statements** (doc-staleness policy: fix only false ones): the deck said RID was a future
  "ถ้า ORST เปิด" placeholder and coverage was 62,107/~29,000 — RID is ingested and coverage is 76,649
  words / 40,681 entries; public deploy reframed as licence-gated (R3), not "not yet available".
- **Multi-audience grounded in product data dimensions** per mentor #2, WITHOUT deleting `BIBLE` §2.2's
  honesty note (both true at once: a capability argument, not fabricated user research).

**Item 5 (the guard):** re-ran AFTER all edits — `verify_pitch.sh` **ALL PASS**, `verify_r5.sh` §8 pitch +
§9 WASM smoke **ALL PASS**, §10 rank guard **51.3% OK**, 103+4+1 tests. No silent R6/R7-style break.

---

## X — Regional dialect navigator: CUT (per plan)

`NEXT_STEPS_R10` marks X "cut first if short on time." With a hard submission deadline tonight and the
must-do floor (0/U1/R/T) plus all should-do phases (E/W/U2/P) already landed cleanly, X is the lowest
leverage remaining: it needs legacy `.doc`/`.docx` parsing (mixed local-script glyphs + IPA) and serves
only the two 15%-weight criteria, not the 45% (Problem Fit + Innovation) that R/W serve. **Deliberately not
attempted** — no half-working code, no fabricated dialect entries. Recorded here rather than silently
skipped.

---

## Final certification (all measured this session)

- `cargo test`: **103 lib + 4 poc + 1 alloc** pass, 0 failed, 0 warnings.
- `verify_pitch.sh`: ALL PASS. `verify_r5.sh`: §8 pitch + §9 WASM smoke + §10 rank guard (51.3%) all pass.
- Audit invariants: cross_sense=0, KEEP 40/40, CUT 7/7.
- Data: 76,649 words / 40,681 entries / 59,542 graph entities / 73,020 triples. RID 11,265/13,394;
  translit 2,256; specialized 4,986/6,252; evolution 3,204 ก-headwords.
- Public WASM licence gate: regenerated public `defs.blob` = 29,540 records, **0 RID / 0 ศัพท์บัญญัติ**.
- `katgpt-rs` untouched throughout; `README.md` / `.gitignore` not touched; `data/official/` raw never
  committed (only derived .txt/.tsv projections).
- One commit per phase: `9696058` U1 · `a6797bd` R · `481ccf8` R3 · `50f382f` T · `f1acb35` E · `62c447d`
  W · `1018ac0` U2 · `cb677f8` P · (this) deliverables.
