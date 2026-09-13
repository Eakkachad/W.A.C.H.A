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


---

## 5. Analytical (computed, not measured)

- **S1 projected full-graph gain if the relocation bug were fixed:** the spike's measured 6.9× cold-build
  and 2× array-size reduction would carry directly into the WASM artifact (the .seg cache is the trie
  arrays), projecting the 7.4 MB embedded cache → ~3.7 MB and the WASM gzip → ~2 MB. **This is a
  projection from the spike's array-size measurement, not an end-to-end measured build** — it is not
  claimed as achieved, and depends on first fixing the differential-test failure.

No other analytical numbers appear in this document; everything in §1–§4 was produced by a command.
