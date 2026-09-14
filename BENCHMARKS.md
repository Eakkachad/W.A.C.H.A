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

### 3.3 S1/S1b — dense-alphabet trie spike (measured, NEVER merged; DELETED R9 per G2)

A dense-`char`-symbol double-array trie was prototyped to cut the byte-trie's ~58 s cold build (Thai's
3-byte UTF-8 makes the byte trie's collision scans pathological). It was attempted **three times** and
stopped every time on its non-negotiable byte-identical differential gate, then **deleted in R9** per the
G2 deadline rule. The measurements are kept here because they are real and they justify the effort:

| attempt | cold build (byte → symbol) | speedup | trie arrays | differential mismatches |
|---|---|---|---|---|
| S1 (R6) | 58.3 s → 8.5 s | 6.9× | 16.7 → 8.4 MB | 23 / 62,107 words |
| S1b (R8) | 57.1 s → 8.2 s | 6.96× | 16.78 → 8.39 MB | 8,990 (deterministic, sorted alphabet) |
| S1b (R9 Q3) | 59.1 s → 8.4 s | **7.02×** | 16.78 → 8.39 MB | **26** (current vocab) |

**Every attempt hit ~7× build speedup and ~2× smaller arrays — and every attempt FAILED the
byte-identical differential**, so it was never merged (a faster segmenter that segments *differently* is a
regression, not an optimisation). Two real defects were fixed en route in R8 (non-deterministic alphabet →
sorted; in-place relocation overlap → snapshot). The **root cause** is a double-array **base-region
invariant violation** that only the dense contiguous symbol ids expose: under heavy relocation a node's
new slot / a sibling's base window can overlap another node's occupied child region, silently dropping a
transition (so a longer word matches only its shorter prefix). The proper fix is a base-allocation rework
(free-list / disjoint-window, à la darts-clone) — genuinely more than an unattended-night change.

**R9 decision (G2):** three stops is enough. `symbol_trie.rs` (439 lines, wired to nothing) and its two
`examples/s1b_*` harnesses were **deleted** in R9 Q3 to stop dead code from rotting. The capability it
targeted — cutting the ~59 s day-of ingestion to ~9 s — remains a real, quantified opportunity for anyone
who wants to do the base-allocation rework; this table is the starting point. The production byte-keyed
`datrie.rs` is unchanged and correct.

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

### 4.2 Segmentation F1 on wisesight1000 (R7 D1 + R8 D2 + R9 Q1 both modes, measured — our own numbers)

Evaluated on `pythainlp/wisesight1000` (CC0, 993 human-tokenised social-media samples), both under the
char-level `is_beginning` boundary protocol and the AttaCut word-level (exact-span) protocol
(`data/eval/wisesight1000.label`, run via `cargo run --release --example seg_wl_f1` — measures BOTH modes).

**Two segmentation modes (R9 Q1).** Greedy longest-match (original) vs newmm-style **maximal matching**
(DP minimising total tokens). Both use the same trie + TCC OOV fallback; they differ only in how known
words chain. Measured, our own numbers:

| mode | word-level F1 (per-sample) | word-level micro F1 | boundary F1 (per-sample) | boundary micro F1 |
|---|---|---|---|---|
| greedy longest-match | 0.6611 ± 0.2120 | 0.6343 | 0.8015 ± 0.1660 | 0.7822 |
| **maximal matching (SHIPPED)** | **0.6809 ± 0.2054** | **0.6508** | **0.8195 ± 0.1548** | **0.7957** |

**Maximal matching wins on every metric, so it is the shipped default** (`Segmenter::segment`); greedy
stays available via `segment_with(_, SegMode::Greedy)`. **But the win is modest (+~0.02 word-level), not
the gap-closer hypothesised** — it does **not** reach newmm's published **0.74** word-level on
Wisesight-1000 (we sit at 0.6508 micro / 0.6809 per-sample). The residual gap is not just greedy-vs-DP;
it is our word list (LEXiTRON, not newmm's) and the TCC fallback on social-media OOV. `verify_pitch.sh`
passes under the new default and the WASM artifact was rebuilt + smoke-tested.

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

**Word-level F1, measured (R8 D2 + R9 Q1, `cargo run --release --example seg_wl_f1`).** Under the AttaCut
protocol (a predicted word is a true positive only if its exact `(start,end)` span matches a gold word),
the **shipped maximal-matching** segmenter scores **per-sample 0.6809 ± 0.2054 / micro F1 0.6508** (greedy:
0.6611 / 0.6343) on the 993 wisesight1000 samples — **~0.14 lower than its boundary figure**, exactly as
expected: a single wrong internal boundary breaks two words. Precision < recall (micro P 0.576 < R 0.748),
the over-segmentation signature. **On the like-for-like word-level metric neither mode beats PyThaiNLP's
0.74 on Wisesight-1000 — maximal matching sits at 0.6508 micro, below it.** We report only our own numbers;
our word list is NECTEC LEXiTRON and our setup differs, so this is a self-measurement, not a ranking claim.
This is the honest, expected outcome for a dictionary-driven, learned-disambiguation-free segmenter.

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

### 4.4 Reverse dictionary (R8 C build + R9 Q2 v2 enrich/damp, measured)

Own BM25 (k1=1.2, b=0.75) over segmented definitions, CSR postings, matched-token explanations; CLI
`reverse`, web `/api/reverse`, and offline in WASM. **Q2 v2** enriched each document with the entry's
usage examples + related-word headwords, and added a coverage damping factor (`score × coverage^1.0`).

| metric | R8 C (v1) | R9 Q2 (v2) |
|---|---|---|
| indexed docs | 29,584 | 29,763 |
| distinct terms | 18,873 | **28,829** (examples + related words added) |
| index size (in-mem) | ~2.30 MB | ~2.81 MB |
| search p95 | 0.341 ms | sub-ms (unchanged order) |

**Honest quality (10 hand-checked queries, un-curated — full table in `rounds/VERIFY_R9.md` §Q2):** 3/10 clear
hits (ความรู้สึกเสียใจอย่างมาก→น้ำตาตกใน; น้ำที่ตกลงมาจากฟ้า→ฝน; เครื่องดนตรีที่มีสาย→พิณ/ไวโอลิน), several
partials, and the **2 known structural failures persist** (สัตว์เลี้ยงสี่ขาเห่าได้→สุนัข,
เครื่องมือสำหรับเขียนหนังสือ→ปากกา) — the targets' terse ORST-style glosses simply do not contain the
query's descriptive words, so BM25 over glosses cannot reach them. v2's enrichment helped multi-word
conceptual queries and its damping compressed single-rare-term noise (R3's top score fell 11 → 2.7), but it
is not a fix for the structural cases. Reported as-is, not curated.

### 4.5 Loose rhyme index (R11 WRITE-2, measured)

The rhyme finder keys each headword by its final rime — (final-consonant class per มาตราตัวสะกด, vowel
nucleus, long/short) — extracted from the RID pronunciation respelling (falling back to the headword). Two
words sharing a key loosely rhyme. **Deliberately bounded**: this is loose rhyme, not classical เอก/โท
meter matching (a scope trap explicitly avoided this round).

| metric | value |
|---|---|
| indexed words | 38,550 |
| distinct rhyme keys | 234 |
| **build time (full corpus, at web startup)** | **35 ms** (measured; 24 ms in a warm native run) |
| lookup (`rhymes_of`) | O(1) map hit + freq sort of the bucket; sub-ms |

Verified real rhymes (not key collisions): `บ้าน` → การ, งาน, ด้าน, ท่าน, อาหาร, ผ่าน, รัฐบาล (all แม่กน,
long า); `กด` → กฎ, กรด (แม่กด, short อ). CLI `rhyme`, web `/api/rhyme`.

### 4.6 Register filter (R11 WRITE-1, measured)

`Register` (แบบ/โบ/ปาก/ราชา/เลิก) was parsed per-sense but the R10 reshape emitted it in `{braces}` while the
importer only read `(parens)` — so RID register tags were silently dropped until R11 fixed the parser.
1,212 RID senses carry `{โบ}`. `register ราชา` → ผม, คุณ, ทราบ, ดิฉัน, เท้า, สตรี, โค (genuine ราชาศัพท์);
composable with the reverse-dictionary candidate set when a topic query is given. CLI `register`, web
`/api/register`. Self-contained scan over the dictionary — no separate index, sub-ms.

### 4.7 Pretrained semantic vectors — thai2fit_wv (R11 VEC, measured)

Integrated PyThaiNLP's **MIT-licensed** `thai2fit_wv` (51,358 words × 300-dim, word2vec, trained on Thai
Wikipedia — **nothing trained by us**). Licence confirmed before download (thai2fit repo LICENSE = MIT);
used in the local/judge build, gated out of any public deploy per the same caution as RID/ศัพท์บัญญัติ.

| metric | value |
|---|---|
| thai2fit vocab | 51,358 × 300-dim |
| **overlap with wacha vocab (measured)** | **28,589 words = 45.9% of wacha, 55.7% of thai2fit** |
| overlap blob size | ~35 MB (gitignored, regenerable via `scripts/gen_thai2fit_overlap.py`) |
| NN search (flat array + cosine, linear scan) | **~3.9 ms** avg over 100 lookups (28,589 words) — no ANN index needed |

**NN sanity (5 known pairs, measured):** related pairs are far closer than unrelated — หมา~แมว 0.52,
ครู~อาจารย์ 0.45, แม่~พ่อ 0.65, กิน~ดื่ม 0.29, รถ~เรือ 0.36 vs หมา~คอมพิวเตอร์ 0.07, ครู~ก้อนหิน 0.05. Agrees
with the hand-verified relation graph (no contradiction → safe to ship). **Reverse recall (measured):** for
`ความกล้าหาญ`, BM25 returns 6 definitional hits (วีรบุรุษ/วีรกรรม/…); the vector source *adds* 4 semantically-
adjacent candidates (ความบริสุทธิ์ 0.74, ความจงรักภักดี 0.73, ความยิ่งใหญ่ 0.72, ความซื่อสัตย์ 0.71), clearly
labeled `source:"vector"` and composed with (never replacing) BM25.

### 4.8 Intent classification (R12 INTENT, measured)

Deterministic router: layer-1 keyword rules, then a thai2fit centroid fallback only when no rule fires.
No trained classifier.

| layer | metric | value |
|---|---|---|
| Layer-1 keyword rules | regression fixture (22 real-shaped queries) | **22/22 correct** |
| Layer-2 vector fallback | held-out no-keyword queries (10) @ threshold 0.20 | fired 9/10, **6/10 correct** (~60–67% long tail) |
| latency | rule pass | sub-ms (literal `str::contains` over ~40 keywords) |
| latency | fallback | one query-centroid + 5 seed-centroid cosines over 300-dim (sub-ms; centroids are ≤5 word lookups each) |

Threshold 0.20 chosen from a 0.10–0.30 sweep (stable 0.10–0.25). The fallback only fires when layer-1 finds
nothing, and a wrong guess degrades to a routed mode with a one-tap correction line (INTENT-3) — never worse
than the pre-R12 General default. Honest number reported: the fallback is a long-tail aid, not a 90% claim.

**⚠️ v2 also caused a regression, found on review (2026-09-14).** `ที่เก็บเงินของรัฐ` returned **`คลัง`
at rank 1 in v1**; in v2 `คลัง` **falls out of the top 5 entirely** (now: หัวเบี้ย, ค่าธรรมเนียม, ส่วนลด,
เงินตรา, ภาษี). The coverage damping that fixed the single-rare-term noise penalises entries with **terse**
glosses — and `คลัง`'s gloss ("ที่เก็บ…") is exactly that shape, so it loses to longer definitions that
cover more query terms. This is a real trade-off of v2, not a tuning accident: the same mechanism that
improved the conceptual queries demoted the short-gloss ones. **Both directions must be stated whenever
v2 is described.** Two further review queries also fail (`คนที่รักษาคนป่วย` → แวดล้อม/รักษา/คุ้มครอง, not
แพทย์; `ยานพาหนะที่บินได้` → การบิน/บิน/ผู้โดยสาร, not เครื่องบิน), consistent with the same structural
cause: the system finds entries whose glosses *mention* the query words, not entries the query *describes*.

**Open task:** decide whether the damping exponent should be lowered so short-gloss entries are not
punished this hard, and re-measure both the conceptual queries and the short-gloss ones before changing it.
A length-normalisation term (BM25's `b`) is the conventional lever here and is currently at the default.

**Open task (ranking guard resolution):** `verify_r5.sh` §10's rank guard runs on a gold set of **n=39**,
so one pair is 2.6 pp and the whole spread between the shipped ordering and the two known-worse ones is
**two pairs** (20 / 19 / 18 of 39). No threshold can both separate those and avoid false alarms on a single
legitimate reshuffle, so the floor is set at 40% as a smoke alarm and the printed number is what should be
watched. Enlarging the audited gold set is the real fix.

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
