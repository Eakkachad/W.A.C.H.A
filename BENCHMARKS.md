# BENCHMARKS.md — วาจา (WACHA), Round 6

**Date:** 2026-09-14 · **Machine:** macOS (Apple Silicon), single-threaded unless noted ·
**Build:** `cargo build --release` (native), `wasm32-unknown-unknown` release (opt-level=s, lto,
panic=abort, strip) for the WASM artifact.
**What is gated:** all timings below are *warm* (post-build) except where "cold" is stated. Latency is
measured through the HTTP server (an in-process engine), not the CLI (which reloads per call).

**Reproduce:** `cd wacha && bash scripts/verify_r5.sh` prints most of these; the WASM row comes from
`cd wacha-wasm && bash web/build.sh`; the corroboration precision table from
`wacha --data ../data corroboration` (seed `0x4e362026`) hand-audited in
`wacha/data/corroboration_audit_2026-09-14.md`.

> **Measured vs analytical.** Every number in §1–§4 is *measured* (a command produced it). §5 contains one
> *analytical* (computed, not measured) projection and is labelled as such.

---

## 1. Executive summary

| Metric | Result | Note |
|---|---|---|
| Tests | **89 pass** (wacha) + 4 (poc) | `cargo test` |
| Cold engine build | **~58 s** | trie build from 72,175-word vocab dominates |
| Warm start | **~1.09 s** | of which vocab_hash 3.8 ms + cache deser 5.0 ms + engine build ~1.08 s |
| p95 lookup latency (warm) | **13.9 ms** | 40 words via HTTP |
| `บ้าน` top result | **เรือน** (was: absent) | P2/N corroboration ranking fix |
| Cross-source tier-2 precision | **92.5%** | the corroboration headline (§3) |
| Offline WASM artifact | **3.17 MB gzip** (10.83 MB raw) | under the 25 MB budget |
| WASM segmentation vs native | **byte-identical** | 10 PITCH demo words |

---

## 2. Criteria table (Round 6 acceptance thresholds)

| Criterion | Threshold | Result | Pass? |
|---|---|---|---|
| P1 vocab_hash validation step | < 30 ms | **3.8 ms** | ✅ |
| P2 `เรือน` in top 3 for `บ้าน` | top 3 | **#1** | ✅ |
| P2 cross_sense_pairs | 0 | **0** | ✅ |
| P2 KEEP-recall preserved | 40/40 | **40/40** | ✅ |
| S1 cold-build speedup | ≥ 3× | **6.9×** | ✅ (speed) |
| S1 differential (byte-identical) | 0 mismatches | **23/62,107 mismatch** | ❌ → **STOPPED, not merged** |
| W artifact size | ≤ ~25 MB gzip | **3.17 MB** | ✅ |
| W segmentation vs native | byte-identical | **identical** (10 words) | ✅ |

S1 is the one criterion that failed its correctness gate; per the stop rule it was not integrated (the
byte-path segmenter remains production). Speed passed; correctness did not — so it does not ship.

---

## 3. Before / after tables

### 3.1 P1 — `vocab_hash` (order-independent set hash), full merged vocab ~96k words / 72k distinct

| | median time | ratio |
|---|---|---|
| OLD (sort 72k strings + FNV) | 9.6 ms | 1.0× |
| **NEW (O(n) commutative FNV, no sort)** | **3.7 ms** | **2.6×** |

Warmup: 1 untimed call each. Iterations: median of 5. **Honest finding:** vocab_hash was *not* the ~1.0 s
start-up bottleneck the R6 baseline attributed to it — the ~1.0 s is the RelationEngine build (global
PageRank recompute), which S2 (not done) would cache. P1 is a real but small win.

### 3.2 P2/N — ranking of `บ้าน` related words (corroboration tiers)

| | #1 result | `เรือน` rank | `คห` (isolated Kaikki pair) |
|---|---|---|---|
| BEFORE | บ้านเรือน (Wiktionary, score 15.653) | **absent** | top 8 |
| **AFTER** | **เรือน** (WordNet synset, tier 1) | **#1** | dropped out of top 8 |

### 3.3 S1 — dense-alphabet trie spike (measured, NOT merged)

| | cold build | trie arrays | alphabet |
|---|---|---|---|
| byte-keyed (production) | 58.3 s | 16.7 MB | 256 (bytes) |
| symbol-keyed (spike) | **8.5 s** | **8.4 MB** | 136 (chars) |
| ratio | **6.9×** | 2.0× | — |

**Differential test: FAILED** — 23 of 62,107 vocab words segment to a shorter end offset than the byte
trie (pattern: เปล/เปร clusters), a scale-triggered collision-relocation bug in the port. Not merged.

### 3.4 W — offline WASM

| | value |
|---|---|
| artifact raw | 10.83 MB |
| **artifact gzip** | **3.17 MB** (budget ≤ ~25 MB) |
| load + one-time engine build (node) | ~8 s |
| per-lookup after load | < 1 ms |
| segmentation vs native CLI (10 PITCH words) | **byte-identical** |
| dataset | reduced: 62,106 words + seed + WordNet; **no Kaikki** (labelled in UI) |

---

## 4. Cross-source corroboration precision (Phase N, measured) — with CIs and scope (R7 T3)

Stratified random sample, 40 pairs/tier, fixed seed `0x4e362026`, single-rater hand-audit
(`wacha/data/corroboration_audit_2026-09-14.md`). Tier definitions pre-registered in
`relations::corroboration_tier` before the audit. **n=40 per tier ⇒ 95% CI ≈ ±10–15 pp** (Wilson).

| Tier | n | precision (count) | 95% CI (approx) |
|---|---|---|---|
| 2 multi-source (≥2 distinct) | 40 | **92.5%** (37/40) | ~80–98% |
| 3 ORST-attested (post CoinedWord same-discipline fix) | 40 | ~82.5% (33/40) | ~68–91% |
| 0 isolated pair | 40 | ~80% (32/40) | ~65–90% |
| 1 single-source corroborated (synset ≥3) | 40 | **~55%** (22/40) | ~39–70% |

**What separates and what doesn't (honest reading of the CIs):**
- **55% vs 80% separates** — the CIs (≤70% vs ≥65%) barely touch and the 25-pp gap is the real,
  actionable finding. It drove the T1 band re-basing and the T2 flag reversal.
- **92.5% vs 82.5% does NOT clearly separate** — the CIs overlap heavily. So tier 3 and tier 0 are
  merged into one band (B), and 92.5% is reported as "highest, but within sampling error of ~82%".

**Scope — always quoted with the number:** the 92.5% multi-source figure covers **1,074 of 158,045
distinct related pairs (0.68%)**. It is a statement about the rare agreement set, not the whole graph.

**Corpus shift (T3):** the relation graph is now **~84% Wiktionary-derived** — Wiktionary participates in
**132,631 of 158,045 pairs**; WordNet 26,225; ศัพท์บัญญัติ 120; seed 145. Only **0.68%** are multi-source:
Thai WordNet and Wiktionary encode largely *disjoint* synonym knowledge, so combining them adds coverage
more than redundancy, and the rare agreements (band A) are disproportionately trustworthy.

### 4.0b Corroboration audit re-run at n=100 **per band**, fresh seed (R8 A2)

To narrow the R7 CIs, the audit was re-run at **n=100 per measured-precision band** (A/B/C, not raw
tiers) with a **fresh seed `0x8a2d2026`** (≠ the R7 tier-fit `0x4e362026`), same pre-registered band
definitions, single rater (`wacha auditbands`; verdicts in
`wacha/data/corroboration_audit_bands_2026-09-14.md`). Bands available: A 1,017 · B 11,411 · C 145,819.

| band | n | precision (count) | 95% CI (Wald, n=100) |
|---|---|---|---|
| A multi-source | 100 | **96%** (96/100) | ±3.8 → [92, 100] |
| B ORST + isolated pair | 100 | **90%** (90/100) | ±5.9 → [84, 96] |
| C single-source synset ≥3 | 100 | **42%** (42/100) | ±9.7 → [32, 52] |

**The finding holds and is now firmer.** At n=100 the CIs are ~⅓ narrower than at n=40. Band C is the
worst by a wide, clearly-separated margin (42% vs 90–96%) — if anything slightly *below* the R7 55%
estimate (the two CIs overlap in [40, 52]), confirming it is a coin-flip-quality band and validating the
T2 decision to warn (⚠) band C only. Bands A vs B (96% vs 90%) still overlap at the edges, so we do **not**
claim a firm A>B ordering. Band C's misses are dominated by big single-source Wiktionary "synonyms" lists
that lump words sharing a head morpheme (น้ำ…/หัว…/ข้าว…); its hits are largely correct royal/poetic
register synonyms (นฤปะ/อธิป, ยุพเรศ/ยุพิน, กุญชร/คชาชาติ) — the band genuinely mixes both. **Single rater;
no inter-rater agreement (second rater dropped).**

### 4.1 Ranking quality — precision@5 on a held-out sample (R7 T1)

Fresh stratified sample, **seed `0x52372026` (different from the tier-fit seed** so this is not scored on
data the bands were fitted to), 30 query words with ≥5 related, single-rater hand-audit of the top-5:

| ranking | precision@5 | verdict |
|---|---|---|
| R6 tier-number (before) | 119/150 = **79.3%** | baseline |
| **band → PPR → freq (R7 shipped)** | 119/150 = **79.3%** | holds — SHIPPED |
| band → freq → PPR (freq-primary, plan's proposal) | 113/150 = **75.3%** | measured worse — REJECTED |

Frequency-primary was the plan's proposal; it was **measured and rejected** because it pulls
frequent-but-loose words (น้ำ/น้ำมัน for น้ำมันมนตร์, หัว/หัวใจ for หัวคันนา) into the top 5. Reproduce
both via `WACHA_RANK={tier,freq} wacha --data ../data patk 30`.

### 4.2 Segmentation F1 on wisesight1000 (R7 D1 boundary + R8 D2 word-level, measured — our own numbers)

Evaluated our **greedy longest-match** segmenter on `pythainlp/wisesight1000` (CC0, 993 human-tokenised
social-media samples), char-level `is_beginning` boundary protocol
(`data/eval/wisesight1000.label`, run via `cargo run --release --example seg_f1`):

| metric | value |
|---|---|
| per-sample boundary-F1 (mean ± std) | **0.8015 ± 0.1660** |
| micro precision / recall / F1 (boundary) | 0.685 / 0.911 / **0.782** |
| per-sample **word-level** F1 (mean ± std) — R8 D2 | **0.6611 ± 0.2120** |
| micro precision / recall / F1 (**word-level**) — R8 D2 | 0.558 / 0.734 / **0.634** |

This is **our own measured number**, not a citation.

**What the precision/recall split actually means (corrected 2026-09-14).** Recall 0.911 > precision 0.685
means we emit **more** boundaries than the gold annotation — about 31.5% of the boundaries we predict are
not in the gold — so this segmenter **over-segments**, i.e. it splits *more* finely than the human
annotator, and it misses few real boundaries. The most likely driver is the OOV path: wisesight1000 is
social-media text (slang, typos, Latin script), and unknown spans fall back to Thai Character Clusters,
which are short and therefore introduce extra boundaries. *(An earlier version of this paragraph described
this as an "over-merge signature" that "splits less than a human" — that was backwards, and is corrected
here rather than quietly deleted.)*

**⚠️ The boundary figure is NOT comparable to the word-level F1 figures in the Thai segmentation
literature.** The 0.8015 is **character-level boundary F1**; the widely-cited table (AttaCut,
arXiv:1911.07056, Table 2 — PyThaiNLP 0.67 / DeepCut 0.93 on BEST-2010, PyThaiNLP 0.74 on Wisesight-1000)
reports **word-level (WL) F1**, a strictly harder metric: one wrong boundary invalidates the whole word.
The AttaCut authors make this point themselves — *"measuring only the character-level metrics would
overestimate the tokenization performance of word tokenizers"* (§4.2) — which is precisely why they
added WL.

**Word-level F1, measured (R8 D2, `cargo run --release --example seg_wl_f1`).** Under the AttaCut protocol
(a predicted word is a true positive only if its exact `(start,end)` span matches a gold word), our
segmenter scores **per-sample 0.6611 ± 0.2120 / micro F1 0.634** on the same 993 wisesight1000 samples —
**0.14–0.15 lower than the boundary figure**, exactly as expected: a single wrong internal boundary breaks
two words. The precision/recall split holds (micro P 0.558 < R 0.734), confirming the over-segmentation
signature. **On the like-for-like word-level metric, our greedy longest-match does NOT beat PyThaiNLP's
0.74 on Wisesight-1000 — we sit below it (0.634 micro).** We report only our own numbers; our word list is
NECTEC LEXiTRON and our setup differs, so this is a self-measurement, not a ranking claim. This is the
honest, expected outcome for a dictionary-driven greedy segmenter with no learned disambiguation.

**We also do NOT quote newmm's 0.73 TNHC figure as ours** — different algorithm, different corpus, different
metric. Our word list is NECTEC LEXiTRON (credited in the pitch); the benchmark is PyThaiNLP's.

### 4.3 Hot-path allocation counts (R8 E1, measured with a CountingAllocator)

A global `CountingAllocator` (pattern from `katgpt-rs`'s test infra, copied — not imported — into
`wacha/tests/alloc_hotpath.rs`; `katgpt-rs` untouched) counts allocations on the hot query paths of a
seed-only engine. We claim only what the counter reads:

| operation | allocations | note |
|---|---|---|
| `segment("แมว")` (1 in-vocab word) | **2** | output `Vec` + the token's owned `String` |
| `segment(16-char, 10 tokens)` | **94** | ~9/token: each `Token` owns a `String` (from_utf8_lossy) + TCC fallback |
| `segment("x")` (1-char OOV) | **5** | trie miss → single TCC cluster |
| `related_ranked("แมว", 5)` | **31,084** | **alloc-heavy** — see below |

**Honest finding:** the **segmenter path is cheap and linear** in tokens (≈2 allocs/word, no hidden
blow-up), but the **relation path is allocation-heavy** — ~31k allocations per query — because `related()`
runs a **per-query personalized PageRank over the whole graph** (the FolkRank π_q − π subtraction). We
therefore make **no zero-alloc claim** for relation lookup; the counter says otherwise. This is a concrete
optimization target (cache/prune the per-query PPR working set) rather than something to paper over. Run:
`cargo test --release --test alloc_hotpath -- --nocapture`.

---

## 5. Analytical (computed, not measured)
- **S1 projected full-graph gain if the relocation bug were fixed:** the spike's measured 6.9× cold-build
  and 2× array-size reduction would carry directly into the WASM artifact (the .seg cache is the trie
  arrays), projecting the 7.4 MB embedded cache → ~3.7 MB and the WASM gzip → ~2 MB. **This is a
  projection from the spike's array-size measurement, not an end-to-end measured build** — it is not
  claimed as achieved, and depends on first fixing the differential-test failure.

### 5.1 SYNTHETIC scale-headroom curve (R8 A3 — synthetic corpus, NOT real RID)

To probe day-of ingestion capability without possessing the RID, a **synthetic** Thai-shaped word list was
generated at multiples of the ~40k real-RID scale and the dominant cost (the byte-trie segmenter build)
measured (`cargo run --release --example a3_scale`). **This is synthetic data, clearly separated from the
measured §1–§4 numbers.**

| × real RID | synthetic words | segmenter build | trie arrays | ms / 1k words |
|---|---|---|---|---|
| 1× | 40,000 | 25.4 s | 8.39 MB | 636 |
| 2× | 80,000 | 96.9 s | 16.78 MB | 1,212 |
| 5× | 200,000 | 452 s (7.5 min) | 33.55 MB | 2,261 |
| 10× | 400,000 | 2,493 s (42 min) | 134.22 MB | 6,233 |

**Honest finding: the byte-trie build is super-linear (≈quadratic) in vocabulary size** — per-1k-word cost
rises 636 → 6,233 ms across the 10× range. Trie *memory* scales linearly (~0.34 MB/1k words). So the
current ingestion path handles the real ~40k RID comfortably (~25 s cold, then cached to ms), but a
much larger drop (5–10×) would be a multi-minute cold build. **This is exactly the cost S1b's
dense-alphabet trie targets (~7× faster build) — which is why S1b matters and why it is not abandoned,
only stopped on its correctness gate.** The day-of story stands for the real RID scale; beyond ~2× it
needs S1b. Numbers are synthetic; the real RID may differ in word-length distribution.

No other analytical numbers appear in this document; everything in §1–§4 was produced by a command.
