# VERIFY_R8.md — Round 8 verification report

**Run:** unattended overnight, 2026-09-14. Rules from R5B/R6/R7/R8: commit per task, every number from a
script, no silent substitution (STOP + document if a task can't be done cleanly), do not touch
`katgpt-rs`, finish fewer tasks cleanly over many half-done. **Documentation policy this round:** the owner
updates all docs in one final pass, so staleness in `README/PITCH/BIBLE` was left alone — the exception is
statements that are actually **false** (none found this round beyond code bugs, which were fixed).

**Headline:** the whole weakness list from R8 §0 was addressed. The two highest-impact outcomes:
1. **W3 found and fixed a real bug that broke the WASM demo in every strict wasm runtime** —
   `std::time::Instant::now()` is called during engine build and panics on `wasm32-unknown-unknown`
   ("time not implemented"). The flagship offline demo trapped on init. Fixed + init cut **8000 ms → 45 ms**.
2. **S1b was STOPPED again on its correctness gate** (6.96× build speed, but the byte-identical differential
   still fails) — the root cause was isolated but the fix is a base-allocation rework, out of scope.

---

## 1. Status table

| Task | State | Commit | Note |
|---|---|---|---|
| **V1** ⚠-flag audit | ✅ done (right) | `ea256b4` | Flag is CORRECT. `เดิน→ดำเนิน` warns because it shares a size-**46** single-source Wiktionary synset → band C. Plan's "2-member" premise was wrong. Added `wacha probe`. |
| **S1b** dense trie retry | ⏹️ **STOPPED** | `555ca93` | Speed gate PASSES (**6.96×**, 57.1s→8.2s); differential gate FAILS (**8990** mismatches). NOT merged, byte path kept. Fixed 2 real defects; root cause isolated (base-region overlap w/ dense ids). |
| **D2** word-level F1 | ✅ done | `1a70b40` | WL-F1 **0.6611 ± 0.2120** / micro **0.634**, vs boundary 0.8015. Lower as expected; does **not** beat newmm's 0.74 WL. |
| **L1** usage examples | ✅ done | `cd8cc6a` | **5,505/29,601** defined entries gain ≥1 example; CLI + web + WASM, with provenance badge. |
| **C** reverse dictionary | ✅ done | `bf9a3bd` | Own BM25 over segmented defs; CLI + `/api/reverse` + WASM offline. Build 186 ms, p95 **0.341 ms**. Acceptance query result reported honestly (see §3). |
| **W3** WASM init time | ✅ done | `2ded674` | **8000 ms → 45 ms** (prebuilt PageRank blob) **+ fixed the Instant panic** that trapped WASM on init. |
| **A1** ศัพท์บัญญัติ breadth | ✅ done | `aa9725c` | 39 → **174** curated queries across ~40 disciplines ⇒ **402 entries / 834 senses**. Hard limits honored, no non-200. |
| **A2** corroboration n=100/band | ✅ done | `3003bdd` | Fresh seed. Band A **96%**, B **90%**, C **42%** (n=100). Finding holds, firmer. Single rater. |
| **A3** scale headroom | ⏭️ droppable | — | see §4. |
| **E1** CountingAllocator | ⏭️ droppable | — | see §4. |
| Deliverables | ✅ done | (this commit) | `VERIFY_R8.md` + `BENCHMARKS.md` (updated per-task) + `PROGRESS.md`. |

---

## 2. Key measured numbers (from scripts)

- **94 tests** pass (wacha) + 4 (poc). Graph: **57,202 entities / 74,018 triples** / 29,790 sense nodes
  (grew from A1's ศัพท์บัญญัติ expansion). Segmenter 72,301 words; def coverage 41.2%.
- `cross_sense_pairs = 0`; `KEEP_recall = 40/40`; `CUT_absence = 7/7` (invariants intact).
- **V1** (`wacha probe`): `เดิน→ดำเนิน` src=wiktionary, max_group_size=**46**, tier 1, band C, **warns=true** ✓.
  Contrast: `ข้อหา→มลทิน` size 2 → band B, no warn; `วงศ์ตระกูล→วงศ์วานว่านเครือ` size 4 → band C, warns.
- **S1b** (`examples/s1b_time.rs`, `s1b_diff.rs`): byte build **57.1 s** → symbol **8.2 s = 6.96×**; arrays
  16.78 MB → 8.39 MB; **differential mismatches = 8990** (deterministic) ⇒ gate FAILS ⇒ not merged.
- **D2** (`examples/seg_wl_f1.rs`): word-level F1 **0.6611 ± 0.2120** per-sample, micro P/R/F1
  0.558/0.734/0.634 on 993 wisesight1000 samples (AttaCut protocol). Boundary F1 (D1) = 0.8015.
- **L1**: 5,505/29,601 entries carry ≥1 usage example. `รัก`→[พ่อแม่รักลูก, รักชาติ, รักชื่อเสียง];
  `น้ำ`→[น้ำตา, น้ำปลา, น้ำพริก]; `กิน`→[กินหมาก]. Present in WASM (defs.blob v2).
- **C** (`wacha reverse`, `/api/reverse`, WASM `wacha_reverse`): 29,584 docs / 18,873 terms / ~2.30 MB
  in-mem (1.86 MB serialised); build **186 ms**; search **p50 0.040 / p95 0.341 / p99 0.353 ms**.
- **W3** (node, wasm32): instantiate **3 ms** + `wacha_init` **45 ms** = **48 ms** (target < 3 s). แมว related
  words load (proves PR vector loaded, not fallback); รัก examples + reverse work. WASM **17.26 MB raw /
  4.88 MB gzip**. Native warm PageRank cache still loads in **<1 ms** (S2 intact).
- **A1**: ศัพท์บัญญัติ 402 entries / 834 senses; CoinedWord in 460 pairs. `wacha field economy` shows
  discipline disambiguation (เศรษฐศาสตร์: เศรษฐกิจ/ระบบเศรษฐกิจ/การประหยัด vs ภาษาศาสตร์: ความประหยัด).
- **A2** (`wacha auditbands`, seed `0x8a2d2026`): band A 96/100, B 90/100, C 42/100; CIs ~⅓ narrower than
  R7's n=40. Band C is the clearly-worst, low-trust band → validates T2 (warn band C only).

---

## 3. Honest results (un-curated)

- **C — the acceptance query FAILS.** `สัตว์เลี้ยงสี่ขาเห่าได้` does **not** return `สุนัข`/`หมา` in the top 5;
  it returns `จตุร / จัตวา / ๔ / การเห่า` (high-idf single-token matches on `สี่` and `เห่า`). Root cause:
  the ORST-style definition of `สุนัข` (`สัตว์เลี้ยงลูกด้วยนม…เลี้ยงไว้เฝ้าบ้าน`) contains neither `สี่ขา` nor
  `เห่า`, and greedy longest-match makes `สัตว์เลี้ยงลูกด้วยนม` a single token that the query's `สัตว์เลี้ยง`
  cannot match. The **semantically-aligned** query `สัตว์เลี้ยงเฝ้าบ้าน` **does** return `สุนัข` #1. This is a
  real limitation of terse-definition + greedy-tokenizer reverse lookup, reported rather than curated away.
- **A2 band C is 42%, slightly *below* R7's 55% point estimate** (CIs overlap in [40, 52]) — reported as-is.
- **D2 word-level F1 (0.634) does not beat PyThaiNLP's 0.74** on Wisesight-1000 — stated plainly.

---

## 4. Stopped / skipped

- **S1b — STOPPED on the correctness gate** (same discipline as R6). 6.96× build speedup is real, but the
  byte-identical differential fails (8990 mismatches). Two genuine defects were fixed en route
  (non-deterministic alphabet; in-place relocation overlap); the remaining root cause is a double-array
  base-region invariant violation that dense contiguous symbol ids expose, needing a base-allocation rework
  (free-list / disjoint-window) — more than one unattended night. Byte production path untouched; spike
  stays unwired; numbers reproducible via `examples/s1b_{time,diff}.rs`.
- **A3 (scale headroom) — not started.** Droppable per plan §4. The day-of story is still supported by
  `COMPETITION_DAY.md` (measured against fixtures in R6) and the merge-order design; A3 would add a
  synthetic 1×/2×/5×/10× timing curve. Left for a future round.
- **E1 (CountingAllocator) — not started.** Droppable per plan §4. No zero-alloc claim is made anywhere that
  isn't already backed by code inspection; E1 would let us *assert* it with a counter.
- **A2 second rater — dropped** (allowed). All audits are single-rater, stated in the audit doc.

---

## 5. Guardrails honored

- `git -C ../katgpt-rs status --short` = **clean** throughout (verified after every task).
- `README.md` **not touched** by this session (a concurrent session owns it and has committed R8 edits).
- One commit per task; **every commit leaves `wacha-web --data ../data` demoable** (verified each time).
- Generated blobs (`defs.blob`, `reverse.idx`, `pagerank.blob`, `words_th.seg`, `coined_word_cache/`) are
  **gitignored**; regenerated by `examples/gen_*.rs` + `scripts/fetch_coined_word.sh`.
- Every number here came from a committed script/CLI, pasted from real output.

---

## 6. R8 commits

`ea256b4` V1 · `555ca93` S1b (stopped) · `1a70b40` D2 · `cd8cc6a` L1 · `bf9a3bd` C · `2ded674` W3 ·
`aa9725c` A1 · `3003bdd` A2 · (this commit) deliverables.
