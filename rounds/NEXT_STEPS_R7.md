# Round 7 — Make the Ranking Match the Evidence, and Finish the Flagship

**Written 2026-09-14 by the reviewing agent**, after verifying Round 6. Same unattended rules as
`NEXT_STEPS_R5B.md` / `R6`: never silently substitute an approach, commit per task, every claimed number
comes from a script in the repo, **do not touch `katgpt-rs`**, and finishing fewer phases cleanly beats
leaving several half-done.

**Round 6 is accepted.** The stop rules worked (S1 correctly refused to merge at 6.9× because the
differential test failed), and the report openly corrected the reviewer's own wrong P1 diagnosis. Round 7
fixes what that verification then exposed.

---

## 0. What verification found (do not re-derive — act on it)

### 0.1 The ranking contradicts our own audit, in the same commit

Measured precision by tier (`corroboration_audit_2026-09-14.md`, seed `0x4e362026`, n=40/tier):

| Tier | Definition | n | Precision |
|---|---|---|---|
| 2 | multi-source (≥2 distinct sources) | 40 | **92.5%** |
| 3 | ORST-attested | 40 | 82.5% |
| **0** | **isolated pair** | 40 | **80%** |
| **1** | **single-source corroborated (synset ≥3)** | 40 | **55%** ← worst |

But ranking orders by *tier number*. `relations.rs:903` asserts, as a test:

> `"corroborated WordNet synset member (tier 1) must outrank isolated Kaikki pair (tier 0)"`

**We promote the 55%-precision tier above the 80%-precision tier**, on the strength of our own data.

Worse, this is partly the reviewer's fault: R6's acceptance criterion was "`เรือน` must appear in the top
3 for `บ้าน`". `เรือน` sits in a 4-member WordNet synset — i.e. **tier 1, the 55% tier**. The criterion was
satisfied by a rule the evidence says is anti-correlated with quality. **Round 7 must not repeat that
mistake: no acceptance criterion in this document pins a specific word to a specific rank.**

### 0.2 The confidence flag is backwards

Our ⚠ `Unverified` marker means "isolated pair — no corroboration". The audit says isolated pairs are
**80%** correct while big single-source synsets are **55%**. So the flag currently warns about the
*better* class and stays silent on the worse one.

This is not embarrassing — it is the most interesting result of the round, and it is exactly the kind of
thing this project exists to catch. Fixing it is a strong pitch moment: *"เราวัดสัญญาณความเชื่อมั่นของ
ตัวเอง แล้วพบว่ามันกลับหัว — เราจึงแก้ตามข้อมูล ไม่ใช่ตามสัญชาตญาณ"*

### 0.3 The WASM flagship ships without definitions, with 22 MB of budget unused

Artifact is **3.17 MB gzip against a ~25 MB budget**, yet the build excludes Kaikki, so the UI says
*"ไม่มีนิยามในชุดข้อมูลย่อ (มีเฉพาะคำ seed)"*. R6 took the stop rule's "reduced dataset" path without
needing to. Handing a judge a phone where the dictionary has no definitions undoes R5's central work.

### 0.4 Start-up: the reviewer's P1 diagnosis was wrong; S2 is the real fix

`vocab_hash` was 9.6 ms, not 1.0 s. The ~1.08 s is the **RelationEngine build (global PageRank
recompute)**. S2 was marked "droppable" on the basis of that wrong diagnosis. **It is now the highest-value
performance task.**

### 0.5 S1's 6.9× is still winnable

The failure was a *"scale-triggered collision-relocation bug in the symbol-trie port"* — and the byte
version hit **the same class of bug before** (`PROGRESS.md`, bug #2: `Datrie::insert` panic when the array
had to grow, found only on real data). It is a known-fragile code path, not a dead end. The spike survives
at `wacha/src/symbol_trie.rs`, compiled and unit-tested.

---

# PHASE T — Make the ranking and the flags follow the evidence (mandatory)

## T1 — Re-base tier ordering on measured precision

**Do not order tiers by their construction number.** Order by what the audit measured, and treat
statistically indistinguishable tiers as one band. With n=40, 82.5% and 80% are inside each other's
confidence intervals, so the honest structure is **three bands**, not four:

```
band A (high)  : tier 2  — multi-source agreement            92.5%
band B (mid)   : tier 3 + tier 0 — ORST-attested, isolated    ~80-82%
band C (low)   : tier 1  — single-source synset ≥3            55%
```

**Ranking signal.** Combine, in this order: band → corpus frequency (`tnc_freq.txt`) → FolkRank PPR.
Corpus frequency is promoted from tiebreaker to a primary signal, and that is not a hack — real
lexicography orders senses and synonyms by frequency of use, and it is a signal a ราชบัณฑิตยสภา judge
will recognise immediately. Document the rule in `BIBLE.md` §6.4.

**Acceptance — measure ranking quality directly, do not pin any word to any rank.**
Hold out a fresh stratified sample (fixed seed, **different** from the tier-audit seed so this is not
scored on the data the tiers were fitted to), hand-audit it, and report **precision@5 of the ranked
related-word list, before and after**. The change ships only if precision@5 improves or holds. Report the
number either way.

Also re-run the existing guarantees: `cross_sense_pairs = 0`, `KEEP_recall = 40/40`, and paste
`lookup บ้าน / ครู / ครอบครัว / สุนัข / รถยนต์` before and after so the reviewer can see the shift.

## T2 — Fix the ⚠ confidence flag (it currently warns about the wrong class)

Re-base the user-facing marker on the measured bands from T1: warn on **band C**, not on "isolated pair".
Update the legend text in CLI and web, and update `BIBLE.md` §6.6 — which currently describes the
degree-based signal as the design — to state what was measured, that it was found to be inverted, and what
replaced it. Keep the original finding in the document; the reversal is the story.

**Acceptance:** a word whose top relations are band C shows the warning; an isolated-pair relation no
longer does. Paste both. `BIBLE.md` §6.6 rewritten.

## T3 — Statistical honesty on the corroboration numbers

1. **Report confidence intervals.** n=40 per tier: 92.5% is 37/40, 82.5% is 33/40 — roughly ±10 pp at 95%.
   State CIs next to every precision figure. The **55% vs 80%** gap is the finding that survives; the
   **92.5% vs 82.5%** gap does not clearly separate. Say so.
2. **Never quote 92.5% without its scope.** It covers **1,076 of 158,287 pairs (0.68%)**. The scope must
   appear in the same sentence, in `BENCHMARKS.md`, `BIBLE.md`, `PITCH.md` and the deck.
3. **State the corpus shift.** The graph is now **84% Wiktionary-derived** (132,631 of 158,287 pairs;
   WordNet 26,225; ศัพท์บัญญัติ 364; seed 145). That changes the product's character and belongs in
   `BIBLE.md` §7 and the pitch.

## T4 — A pitch regression test (stop discovering stale demos by accident)

Stale demo material has now bitten twice: `รถยนต์ → ยานยนต์` disappeared in R5B, and `บ้าน`'s ranking
degraded in R6 — both found only because a reviewer happened to run them.

Write `scripts/verify_pitch.sh`: for every demo word in `PITCH.md` §3, run the real lookup and assert the
scripted claim still holds (the word appears, the stated relation is present, the stated badge/flag is
what the script says). Fail loudly with a diff when a claim no longer matches.

**Acceptance:** the script passes, or it fails and you fix `PITCH.md` until it passes. Add it to
`verify_r5.sh` so future rounds cannot silently invalidate the pitch.

---

# PHASE W2 — Put the dictionary back into the flagship

## W2 — Ship Kaikki definitions inside the WASM build

There is ~22 MB of unused budget. 29,601 definitions as compressed text should land well inside it.

1. Build a compact definitions blob for the WASM target: one concatenated text region plus a `u32` offset
   table, keyed by entry id. Front-code the sorted headword list (Thai shares long prefixes). **No JSON at
   runtime.**
2. Embed or `fetch` it (a separate `fetch` is fine and keeps the `.wasm` smaller — the service worker
   caches it for offline use either way).
3. Report artifact size raw and gzipped, and time-to-first-lookup on a cold cache.
4. Update the UI text: it currently says definitions are absent. If the full set ships, that label must go.
   **If you ship a subset, the label stays and must state exactly what is included.**

**Stop rule:** if total gzip exceeds ~15 MB, ship definitions for the most frequent 15,000 words instead
of all 29,601 and label it precisely. Do not silently truncate.

**Acceptance:** loaded in a real browser, **DevTools offline mode enabled**, and `lookup` returns a real
definition for a non-seed word (e.g. `ปัญญาประดิษฐ์`) — describe exactly what you saw. Confirm the second
visit works with the network disabled from the start (service worker). Differential test against native on
the `PITCH.md` demo words must stay byte-identical.

---

# PHASE S — Performance, now correctly diagnosed

## S2 — Cache the global PageRank vector (the actual start-up fix)

Global PageRank is query-independent but recomputed on every engine build — this is the ~1.08 s. Serialize
it beside the trie cache, keyed by a content hash of the graph (entity + triple counts plus a hash of the
triple set). Invalidate exactly like the trie cache.

**Acceptance:** start-up breakdown before/after; target warm start **< 200 ms**; a test proving a graph
change invalidates the cached vector; `lookup` results unchanged (differential on the demo words).

## S1b — Second attempt at the dense-alphabet trie

Debug the collision-relocation bug rather than restarting the design. Start from the 23 failing words: they
are the reproduction case. Look first at `resolve_collision` / `find_new_base` / `reparent_children` in the
port — the byte version had a bug of exactly this class (array growth during relocation), so compare the
two implementations side by side at the growth path.

Keep every R6 gate: **≥3× cold build, and a byte-identical differential across all 72,175 vocabulary words
plus the `PITCH.md` sentences.** Same stop rule — if it does not pass, it does not merge, and the numbers
go in the report.

**Do D1 before this** (below): a measured segmentation F1 gives the differential test a quality backstop,
so if segmentation ever does change you can say by how much it changed, not just that it changed.

---

# PHASE R — Remaining value, in priority order

## D1 — Measured segmentation accuracy *(highest value of the remainder — NECTEC audience)*

Per `NEXT_STEPS_R5.md` Task 11. Evaluate on `pythainlp/wisesight1000` (CC0, ~74 kB, char-level
`is_beginning`), report boundary-F1 as per-sample mean±std following the published protocol.

Optionally switch greedy → maximal matching first and report both; **but do not quote newmm's 0.73 TNHC
figure as ours** — that is maximal matching's number under a different setup. Measure our own or report
nothing. Reference table (AttaCut paper, arXiv:1911.07056, Table 2) is reproduced in `NEXT_STEPS_R5.md`
Task 11 for context only.

NECTEC built LEXiTRON (our word list) and the RID platform, so expect someone in the room who knows these
benchmarks. Credit LEXiTRON/NECTEC explicitly in the pitch while you are here.

## C — Reverse dictionary (ค้นคำจากความหมาย)

Per `NEXT_STEPS_R6.md` Phase C, unchanged. Now additionally valuable because W2 puts the definitions in the
browser — reverse lookup would work offline on a judge's phone, which no API-backed team can match.

## E1 — `CountingAllocator` to make the zero-alloc claim verifiable

Per `NEXT_STEPS_R6.md` Phase E1. Only claim what the counter actually reads.

---

## Order, and what to drop

```
T1 → T2 → T3 → T4 → W2 → S2 → D1 → C → S1b → E1
```

- **T1–T4 are mandatory.** A ranking that contradicts our own published audit is the single worst thing
  we could take on stage, because our whole pitch is "we measure and we do not overclaim."
- **W2 is the highest-value demo work.** The flagship currently cannot look up a word.
- **S2 is cheap and now correctly targeted.**
- **D1 before S1b**, so segmentation quality is measured before it is touched.
- **Droppable:** S1b, E1, and C if the night runs short.

**Deliverable:** `VERIFY_R7.md` in the established format (status table, raw script output, deviations,
stopped/skipped, known-stale claims), `BENCHMARKS.md` updated, and a dated `PROGRESS.md` entry with the
status board refreshed.

**One standing instruction, given twice now and worth repeating:** if a task cannot be finished cleanly,
stop it, commit what is green, and write up why. Two rounds running, the stop rules produced better
outcomes than pushing through would have.
