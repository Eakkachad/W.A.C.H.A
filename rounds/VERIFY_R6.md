# VERIFY_R6.md — Round 6 verification report

**Run:** unattended overnight, 2026-09-14. Same rules as R5B (commit per task, every number from a script,
no silent substitution, do not touch `katgpt-rs`, do not commit the concurrent session's `README.md`).

**Headline:** the full spine **P1 → P2/N → P3 → S1 → W** is complete. S1 hit its stop rule (fast enough
but failed the byte-identical gate) and was correctly NOT merged. The flagship (W, offline WASM) works and
is byte-identical to native for segmentation. Optional phases (C/D1/E1) were **not started** — see §4.

---

## 1. Status table

| Task | State | Commit | Note |
|---|---|---|---|
| **P1** vocab_hash O(n), no sort | ✅ done | `aa20ab2` | 3.7 ms vs 9.6 ms (2.6×); <30 ms target. Honest finding: hash was not the ~1 s bottleneck (that's the PageRank build). |
| **P2/N** corroboration-tier ranking | ✅ done | `deb984a` | `บ้าน`→`เรือน` now #1; test `corroborated_outranks_isolated`; cross_sense 0; KEEP 40/40. |
| **N** precision-by-tier + overlap (measured) | ✅ done | `3157a3e` | tier-2 = 92.5%; CoinedWord cross-discipline bug found & fixed (52.5%→82.5%); source overlap 0.68%. CLI `corroboration`, seed `0x4e362026`. |
| **P3** stale numbers / Kaikki label / gold-set scope | ✅ done | `4deed80` | honest per-source explanation label; PITCH timing 58s/1.4s; BIBLE KEEP-recall scope note. |
| **S1** dense-alphabet trie | ⏸️ **STOPPED (documented)** | `d3de78a` | 6.9× cold build BUT differential test failed (23/62,107) → not merged; byte path kept; spike retained + unit-tested. |
| **W** offline WASM flagship | ✅ done | `3b5d78d` | 3.17 MB gzip; seg byte-identical to native; ~8 s load; PWA. Reduced dataset (no Kaikki), labelled. |
| **C** reverse dictionary | ⏭️ not started | — | optional; night budget spent on the spine + honest measurement. §4. |
| **D1** segmentation F1 | ⏭️ not started | — | optional; §4. |
| **E1** allocator counting | ⏭️ not started | — | optional; §4. |
| Deliverables | ✅ done | (this commit) | `VERIFY_R6.md` + `BENCHMARKS.md` + `PROGRESS.md`. |

Phase-P commits precede S1/W, so the mandatory correctness fixes gated the rest as required.

---

## 2. Key measured numbers (all from scripts — see BENCHMARKS.md for the full tables)

- Tests: **89 pass** (wacha) + 4 (poc). Graph: 57,061 entities / 73,025 triples / 29,353 sense nodes;
  72,175 searchable words / 29,601 defined entries.
- `cross_sense_pairs = 0`; `KEEP_recall = 40/40`; `CUT_absence = 7/7` (CLI `audit`).
- Warm start: vocab_hash **3.8 ms** + cache deser **5.0 ms** + engine build **~1.08 s** = ~1.09 s.
- p95 lookup latency (warm, HTTP): **13.9 ms**.
- Corroboration precision by tier (seed `0x4e362026`, hand-audited): T2 **92.5%**, T3 **~82.5%**
  (post-fix), T0 **~80%**, T1 **~55%**. Source overlap: 0.68% multi-source.
- WASM: **3.17 MB gzip** / 10.83 MB raw; segmentation byte-identical to native on 10 PITCH demo words.
- `git -C ../katgpt-rs status --short` empty (untouched).

---

## 3. Deviations from the plan

1. **W uses a raw `extern "C"` ABI, not wasm-bindgen.** The plan named wasm-bindgen; the unattended build
   box has no `wasm-bindgen` CLI and `cargo install`-ing it risked eating the night. The raw ABI
   (alloc/free/init/segment/lookup over linear memory) is self-contained, needs no extra tooling, and is
   verifiable with plain `node`. A wasm-bindgen wrapper is a drop-in future nicety. Documented in the
   crate header.
2. **W real-browser airplane-mode test is documented as manual, not executed.** An unattended CLI cannot
   toggle a browser's network state. Instead the module was verified self-contained via a `node`
   `WebAssembly.instantiate(bytes, {})` harness (empty imports = no host calls possible) and its
   segmentation proven byte-identical to native. The literal "open in a browser, enable airplane mode,
   lookup still works" step remains for a human — but the artifact provably makes no host calls.
3. **W ships the reduced dataset (no Kaikki), as the stop rule directs** — but note the reduction is
   "no Kaikki definitions", i.e. segmenter + seed + WordNet, rather than "30k defined words". Embedding
   the 29k Kaikki definitions is the Phase-M footprint work (front-coded blob), which was not done, so the
   simplest honest reduction was to omit that layer. The UI states this plainly.
4. **P2's "option (a) vs (b), keep the better" was resolved by doing Phase N's principled version**
   (corroboration tiers), exactly as the addendum says N replaces P2(b). Option (a) (pure source-tier)
   was not separately implemented/measured; the tier ranking IS the source-tier idea with a measured
   precision table behind it.
5. **CoinedWord same-discipline fix made mid-Phase-N.** The audit exposed a real data-model bug (cross-
   discipline synonym links). Fixing it is a correctness change, not tier-tuning; both pre-fix (52.5%)
   and post-fix (82.5%) tier-3 numbers are reported, and the tier *definitions* were not changed.

No approach was silently substituted; every deviation is recorded here.

---

## 4. Stopped / skipped tasks

- **S1 (dense-alphabet trie): STOPPED per its own rule.** Measured 6.9× cold-build speedup (well over the
  3× bar) but the non-negotiable differential test failed (23/62,107 vocab words segment differently — a
  scale-triggered collision-relocation bug in the symbol-trie port). Per "a faster segmenter that segments
  differently is a regression", it was NOT integrated; the byte path stays production. The spike
  (`wacha/src/symbol_trie.rs`) is kept, compiled, and small-scale unit-tested, wired to nothing, so the
  6.9× result and the remaining bug are on record. This is a documented result, not a failure.
- **C (reverse dictionary), D1 (segmentation F1), E1 (allocator): NOT STARTED.** The honest-scoping note
  in the plan is explicit that the addendum + original R6 is >20 h of work for one night, and that
  finishing the spine cleanly beats leaving several phases half-done. The spine (P1–P3, S1-with-stop, W)
  is complete and green; rather than open C/D1/E1 and risk leaving them half-built at the end of the
  night, they are left for a future round. D1 in particular is called out as high-value for a NECTEC
  audience and is the recommended next pickup.

---

## 5. Known-stale / watch-list claims

- **`84.2%` WordNet precision** (older figure in PITCH/BIBLE) was not re-measured this round. Phase N's
  new tier table (T2 92.5%, etc.) is the current, better-scoped precision story; the 84.2% still describes
  a 2026-09-12 random sample of the WordNet layer and should be read as such.
- **PITCH.md / PITCH_DECK.md** were updated for timings (P3) but their demo-word *related lists* were last
  fully re-verified in R5B C2; the P2/N ranking change reorders some related words (e.g. `บ้าน` now leads
  with `เรือน`). The pitch's `สนาม`/`ครู` beats are unaffected, but a pre-demo re-read of the `บ้าน` beat
  is advised. (Not fixed this round to avoid scope creep; flagged here.)
- **`wacha/README.md`** asset list still predates CoinedWord/RID and the WASM build; `wacha/API.md` remains
  the authoritative licence source.
- **WASM `~8 s` load** is a node measurement; a real browser (WASM streaming compile + different JIT) may
  differ. Reported as measured-via-node, not measured-in-browser.
- The concurrent session's **`README.md`** edit is still uncommitted and was deliberately left untouched.

---

## 6. Guardrail compliance

- `katgpt-rs`: untouched (status empty).
- Commit per task; each commit leaves `wacha-web --data ../data` serving (verified after P1/P2/P3/S1/W).
- No silent substitution: S1's stop was honored (not shipped); the WASM ABI deviation is documented (§3).
- No mass-scrape (no network fetches this round beyond the already-cached data).
- `README.md` (concurrent session) not committed.
- Every documented number traces to a script/command (BENCHMARKS.md §Reproduce).
