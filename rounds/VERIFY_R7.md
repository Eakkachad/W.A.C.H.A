# VERIFY_R7.md — Round 7 verification report

**Run:** unattended overnight, 2026-09-14. Rules from R5B/R6/R7: commit per task, every number from a
script, no silent substitution, do not touch `katgpt-rs`, do not commit the concurrent session's
`README.md`, finish fewer phases cleanly over many half-done.

**Headline:** all mandatory Phase-T tasks complete, plus the flagship W2 (definitions in WASM), S2 (the
correctly-diagnosed start-up fix), and D1 (measured segmentation F1). The round's defining move: **the
acceptance criterion was redesigned to measure ranking quality directly (precision@5 on a fresh held-out
sample) with no word pinned to a rank — and that measurement then rejected the plan's own freq-primary
proposal.** Optional C / S1b / E1 not started (honest scoping — see §4).

---

## 1. Status table

| Task | State | Commit | Note |
|---|---|---|---|
| **T1** re-base ranking on measured bands | ✅ done | `d59ad83` | band→PPR→freq shipped (p@5 79.3%); freq-primary measured (75.3%) & **rejected**. No word pinned to a rank. |
| **T2** un-invert the ⚠ flag | ✅ done | `58ea8a6` | warn band C (55%), not isolated pairs (80%). ข้อหา→มลทิน now unflagged; วงศ์ตระกูล now flagged. |
| **T3** statistical honesty | ✅ done | `9d0a690` | CIs (55 vs 80 separates; 92.5 vs 82.5 doesn't); 92.5% scoped to 0.68%; 84% Wiktionary corpus shift. |
| **T4** pitch regression test | ✅ done | `782ed7d` | `verify_pitch.sh` (in `verify_r5.sh §8`); caught R7's own stale demo claims, fixed PITCH §3. ALL PASS. |
| **W2** definitions in WASM | ✅ done | `b22ca3d` | 29,601 defs, 3.99 MB gzip (< both budgets, full set); non-seed lookup works; seg byte-identical. |
| **S2** cache global PageRank | ✅ done | `996b56a` | warm engine build 1.097 s → **69 ms** (PR load 65 µs); invalidation test; byte-identical lookups. |
| **D1** segmentation boundary-F1 | ✅ done | `7845867` | wisesight1000 (CC0): **0.8015 ± 0.1660** per-sample; our own number, not newmm's 0.73. |
| **C** reverse dictionary | ⏭️ not started | — | optional; §4. |
| **S1b** retry dense-alphabet trie | ⏭️ not started | — | optional; §4. |
| **E1** CountingAllocator | ⏭️ not started | — | optional; §4. |
| Deliverables | ✅ done | (this commit) | `VERIFY_R7.md` + `BENCHMARKS.md` + `PROGRESS.md`. |

Phase T (mandatory) precedes everything else.

---

## 2. Key measured numbers (from scripts — full tables in BENCHMARKS.md)

- **90 tests** pass (wacha) + 4 (poc). Graph: 57,061 entities / 73,025 triples / 29,353 sense nodes.
- `cross_sense_pairs = 0`; `KEEP_recall = 40/40`; `CUT_absence = 7/7`.
- **T1 precision@5** (held-out seed `0x52372026`, single-rater): band→PPR→freq **79.3%** (shipped, holds
  vs R6's 79.3%); band→freq→PPR **75.3%** (rejected). `WACHA_RANK={tier,freq}` reproduces both.
- **T3 CIs** (n=40 ⇒ ~±10–15 pp): 92.5%(37/40), 82.5%(33/40), 80%(32/40), 55%(22/40). 55% vs 80%
  separates; 92.5% vs 82.5% does not. 92.5% covers **1,074/158,045 = 0.68%**. Corpus is **~84% Wiktionary**
  (132,631/158,045; WordNet 26,225; ศัพท์บัญญัติ 120; seed 145).
- **S2 warm start:** engine build 1.097 s → **69 ms** (global PageRank recompute 1.027 s → cache load
  65 µs). Target < 200 ms met.
- **W2 WASM:** 14.80 MB raw / **3.99 MB gzip**; all 29,601 defs; ปัญญาประดิษฐ์/รถยนต์/ครอบครัว return real
  Kaikki defs; segmentation byte-identical to native on 10 PITCH words; init ~8 s.
- **D1 segmentation:** boundary-F1 **0.8015 ± 0.1660** per-sample; micro P/R/F1 0.685/0.911/0.782.
  **Character-level, therefore NOT comparable to the word-level F1 figures in the literature** (AttaCut
  Table 2); our word-level F1 is unmeasured and would be lower. Recall > precision means the segmenter
  **over-segments** (emits ~31.5% more boundaries than gold), not under — see `BENCHMARKS.md` §4.2, where
  an earlier backwards reading was corrected on 2026-09-14.
- `git -C ../katgpt-rs status --short` empty (untouched).

---

## 3. Deviations from the plan

1. **T1 frequency was NOT promoted to the primary ranking signal — the plan's proposal was measured and
   rejected.** The plan said "corpus frequency promoted to a primary signal … not a hack". Measured on the
   held-out precision@5 sample, band→freq→PPR scored **75.3%** vs band→PPR→freq **79.3%** — frequency
   primary pulls frequent-but-loose words (น้ำ/น้ำมัน for น้ำมันมนตร์; หัว/หัวใจ for หัวคันนา) into the top
   5. Per the T1 rule ("ships only if precision@5 improves or holds"), frequency was kept *below* PPR. The
   band re-basing (the actual fix for the measured tier inversion) still shipped. This is the plan's own
   overriding principle applied to the plan's own suggestion — documented, both numbers reported.
2. **W2/S2 real-browser and any browser step remain node-verified, not browser-verified.** An unattended
   CLI cannot drive a browser's DevTools offline toggle. W2 was verified via a `node`
   `WebAssembly.instantiate(bytes, {})` harness (empty imports ⇒ no host calls possible): non-seed
   definitions load and lookups work; segmentation is byte-identical to native. The literal "open in a
   browser, enable DevTools offline, look up ปัญญาประดิษฐ์" step is documented for a human. (Same honest
   limitation stated in R6.)
3. **S2's diagnosis confirmed the R6 reviewer's P1 note was wrong** (as the R7 plan itself flagged):
   vocab_hash was 3.8 ms; the ~1.08 s was the global-PageRank recompute. S2 fixes exactly that.

No approach was silently substituted; the one place the plan and the evidence disagreed (T1 frequency) is
called out explicitly with both measurements.

---

## 4. Stopped / skipped tasks

- **C (reverse dictionary), S1b (retry dense trie), E1 (allocator): NOT STARTED.** The plan lists these as
  droppable if the night runs short and orders them last. The mandatory Phase T plus the two
  highest-value optional items (W2 flagship, S2 start-up) plus D1 are complete and green. Rather than open
  three more phases and risk leaving them half-built at end of night (explicitly the outcome the plan does
  not want), they are left for a future round.
  - **C** is now well-primed: W2 put the 29,601 definitions in the browser, so a reverse index would work
    offline — the recommended next pickup.
  - **S1b** has its reproduction case ready (the 23 failing เปล/เปร words from R6's S1 spike) and D1 now
    gives it a segmentation-quality backstop.
- **No STOP rule fired** this round (W2 came in at 3.99 MB gzip, far under the 15 MB subset threshold, so
  the full definition set shipped; S2 hit < 200 ms).

---

## 5. Known-stale / watch-list claims

- **`84.2%` WordNet precision** (older figure) still appears in PITCH/BIBLE alongside the new R7
  corroboration numbers. It is a 2026-09-12 sample of the WordNet layer; the R7 tier table (92.5% / 55% /
  …) is the current, better-scoped precision story. Both are labelled; the 84.2% is not wrong, just older
  and narrower.
- **precision@5 and the tier audit are single-rater.** Stated as a limitation in the audit doc. A
  second rater would tighten the CIs but was out of scope unattended.
- **WASM `~8 s` init and the browser offline behaviour** are node measurements / documented-manual,
  not browser-measured (§3).
- **`wacha/README.md`** asset list still predates the WASM defs blob; `wacha/API.md` remains the licence
  source of truth.
- The concurrent session's **`README.md`** edit is still uncommitted and untouched.

---

## 6. Guardrail compliance

- `katgpt-rs`: untouched (status empty). `graph.rs` (vendored) untouched.
- Commit per task; each leaves `wacha-web --data ../data` serving (verified through S2).
- **No acceptance criterion pins a word to a rank** (the R7 mandate) — T1 is scored by precision@5 on a
  fresh held-out seed; the band test asserts a *class* ordering (band B > band C) on synthetic data, not a
  real word's position.
- No silent substitution: the T1 frequency deviation is documented with both measurements.
- No mass-scrape: D1 fetched one CC0 file (wisesight1000, 217 KB) once.
- `README.md` (concurrent session) not committed. Every number traces to a script (BENCHMARKS §Reproduce).
