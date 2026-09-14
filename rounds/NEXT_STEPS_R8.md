# Round 8 — Close Every Remaining Weakness

**Written 2026-09-14 by the reviewing agent**, after verifying R7. Same unattended rules as
`R5B`/`R6`/`R7`: never silently substitute an approach, commit per task, every claimed number comes from
a script in the repo, **do not touch `katgpt-rs`**, and finishing fewer tasks cleanly beats leaving
several half-done.

**Documentation policy for this round:** the project owner has decided that **all documents get updated
in one pass after the system is finished** (2026-09-14). So do **not** spend this round rewriting
`README.md`/`PITCH.md`/`BIBLE.md` for staleness. The one exception is a statement that is actually
**false** rather than merely outdated — fix that immediately, as was done for the D1 write-up.

---

## 0. The weakness list this round exists to close

Everything below was found by independently verifying R5–R7. Items marked **(day-of)** cannot be closed
before the event and are handled by being stated honestly, not by pretending otherwise.

| # | Weakness | Closable now? |
|---|---|---|
| 1 | Cold build ~58 s → cannot ingest live on stage | **Yes** — S1b |
| 2 | Word-level F1 unmeasured; only char-level boundary F1 exists | **Yes** — D2 |
| 3 | ศัพท์บัญญัติ is 39 cached queries out of 89,410 terms | **Partly** — A1 |
| 4 | Tier audits are n=40, single-rater (±10–15 pp) | **Yes** — A2 |
| 5 | 10,154 senses carry usage examples; the learner objective is still 20 hand-written blurbs | **Yes** — L1 |
| 6 | No reverse lookup ("ค้นได้มากกว่าการเปิดหาความหมาย") | **Yes** — C |
| 7 | WASM init ~8 s — slow on a judge's phone | **Yes** — W3 |
| 8 | Possible ⚠-flag inconsistency (see V1) | **Yes** — V1 |
| 9 | Zero-alloc claims unproven | **Yes** — E1 |
| 10 | Relations are ~84% Wiktionary-derived | **(day-of)** — see §2 |
| 11 | Definition coverage 41% of the union vocabulary | **(day-of)** |

---

## 1. Verify first (cheap, and it may be a real bug)

### V1 — Does the ⚠ flag actually follow the measured band?

T2's own test documents the intent:

```
band C (single-source synset ≥3, 55%)  -> Unverified (warn)
band B (isolated pair, 80%)            -> Confirmed (no warn)
```

But `lookup เดิน` returns `ดำเนิน` — a Kaikki **word-level** pair, which `relations.rs:178` builds as a
2-member group, so `max_group_size = 2`, so it should be band B and **should not warn** — yet it carries
`⚠ ยังไม่ยืนยัน`.

Either the tier computation is not seeing what I think it sees, or the flag does not follow the band for
Kaikki-sourced pairs. **Determine which, and report it either way** — if it turns out to be correct
behaviour, write down why, because it is not obvious from the code.

**Acceptance:** for one band B pair and one band C pair, print the computed `src_set`, `max_group_size`,
resulting band, and whether the flag shows. Fix if wrong; document if right.

---

## 2. The strategic question: bring in the whole RID now?

**Recommendation: no. Do not scrape พจนานุกรมฉบับราชบัณฑิตยสถาน.** Three reasons, in order of weight:

1. **Reputational.** The RID has no bulk download; taking it means ~40,000+ POST requests to the
   organizer's own server in the days before their own competition. If anyone notices that traffic, we
   are the team that hammered ORST. No demo gain is worth that.
2. **Licensing.** The site's disclaimer (OCR-read 2026-09-13) states educational, non-commercial use with
   copyright held by ORST and the platform by NECTEC. We could not ship it inside the WASM artifact —
   our flagship — without redistributing it. So the scrape would not even serve the main demo.
3. **It is being handed to us anyway.** The organizers said teams receive the real dataset at the event.

**What actually proves the capability is speed plus a runbook, not possession of the data.** So prove it
this way instead:

- **S1b** brings cold build down far enough that ingestion is a stage-able action rather than a coffee break.
- **A3** produces a *scale headroom* measurement — ingest a synthetic corpus several times larger than the
  real RID and report the timing curve. Label it **synthetic** in the "analytical / not measured on real
  data" section, exactly as `BENCHMARKS.md` already separates measured from projected.
- **`COMPETITION_DAY.md`** is already written and timed against fixtures (R6 B2). Re-time it after S1b.

Then the on-stage line is true and checkable: *"ระบบเรารับข้อมูลจริงของท่านได้ในกี่วินาที — นี่คือขั้นตอน
และนี่คือเวลาที่เราจับได้จริง"* — without having taken anything we were not given.

**On weakness #10 (84% Wiktionary):** do not try to fix this by scraping ศัพท์บัญญัติ at scale either.
Shifting a 132,631-pair Wiktionary majority would need tens of thousands of requests — the same problem.
A1 expands the sample modestly for demo breadth, and the pitch states the mix honestly, noting that the
real dataset becomes the top-priority layer on the day (the `Importer` merge order already puts
`Rid > HumanSeed > CoinedWord > Kaikki > Lexitron`).

---

## 3. Tasks

### S1b — Fix the dense-alphabet trie and land the 6.9× *(highest value)*

The R6 spike (`wacha/src/symbol_trie.rs`) measured **6.9× cold build** but failed the differential test on
**23 of 62,107** words — a *"scale-triggered collision-relocation bug"*. It is compiled and unit-tested but
wired to nothing.

Debug it rather than redesign. The 23 failing words **are** the reproduction case — start there. The byte
version had a bug of exactly this class before (`PROGRESS.md` bug #2: `Datrie::insert` panic when the array
had to grow, found only on real data), so compare `resolve_collision` / `find_new_base` /
`reparent_children` against the byte implementation at the growth path specifically.

**Gates unchanged from R6:** ≥3× cold build **and** byte-identical segmentation across all vocabulary
words plus the `PITCH.md` sentences. Same stop rule — if it does not pass, it does not merge, and the
numbers go in the report.

**Knock-on wins to measure if it lands:** cold build, warm start, trie cache size, and the WASM artifact
(the `.seg` cache is the trie arrays — R7 projected 7.4 MB → ~3.7 MB and WASM gzip → ~2 MB; that projection
becomes a measurement).

### D2 — Measure word-level F1 (the open task from R7)

Carried from `BENCHMARKS.md` §4.2. Evaluate word-level F1 on the same wisesight1000 split under the
AttaCut protocol (per-sample mean ± std), and report it beside the boundary figure. Until this exists, the
standing instruction is that 0.8015 may not be placed next to the published 0.67/0.74/0.93 figures.

Optionally also implement maximal matching (newmm's algorithm) and report both — but report our own
numbers only, never newmm's, and note that our word list and setup differ.

**Acceptance:** both metrics in one table with the protocol stated; if WL-F1 is much lower than the
boundary figure, say so plainly — that is the honest, expected result.

### A1 — Expand the ศัพท์บัญญัติ sample for demo breadth (not for corpus share)

Currently **39 cached queries**. Expand to **at most ~500 terms**, chosen for *coverage of disciplines*
rather than volume — a handful from each of the 40 fields, plus terms likely to come up on stage. Same
hard limits as R6 B1: ≥500 ms between requests, single-threaded, stop on the first non-200, cache and
never re-fetch. Hard-code the 40-discipline list (the dropdown omits ธรณีวิทยา).

**Do not raise the ceiling.** The goal is that a judge from any discipline can find their own vocabulary
in the demo — not to change the corpus mix, which A1 cannot and should not do.

### A2 — Strengthen the corroboration audit

n=40 per tier with a single rater gives ±10–15 pp, which is why R7 could only claim that 55% vs 80%
separates and 92.5% vs 82.5% does not.

1. Raise to **n=100 per band** using the same pre-registered band definitions and a fresh seed; report the
   narrowed CIs.
2. If a second person can audit even 40 pairs, report **inter-rater agreement** (raw agreement is enough;
   Cohen's κ if you can). If not, state explicitly that all audits are single-rater.

**Do not revise the band definitions after seeing results.** If you want to, report both the
pre-registered and revised numbers and label which is which.

### L1 — Surface usage examples (the ORST learning objective)

**10,154 of 43,883 senses (23.1%) carry usage examples** from Kaikki and they are barely visible.
`lookup เดิน` shows a definition and related words but no example sentence.

The brief asks for a system that helps people learn to use Thai (*"ช่วยเรียนรู้การใช้ภาษาไทย"*), and this
is our weakest-served objective — currently 20 hand-written blurbs. Surface the real examples in CLI, web,
and WASM, with the same provenance/licence badge as definitions.

**Acceptance:** report how many of the 29,601 defined entries gain at least one example; paste three
lookups showing examples; confirm they appear in the WASM build too.

### C — Reverse dictionary (ค้นคำจากความหมาย)

Per `NEXT_STEPS_R6.md` Phase C, unchanged. Inverted index over segmented definitions (segmented with our
own segmenter), CSR posting lists, BM25 (k1=1.2, b=0.75 — our own ~40 lines; katgpt-rs has no BM25, do not
imply otherwise), returning matched tokens so the UI can show *why* each candidate matched.

Ship it in the WASM build too — an offline reverse dictionary on a judge's phone is something no
API-backed team can match.

**Acceptance:** ≥5 hand-checked queries with real output, including `สัตว์เลี้ยงสี่ขาเห่าได้` → `สุนัข`/`หมา`
in the top 5. Report index build time, index size, p95 reverse latency. **Report failures honestly rather
than curating the query set.**

### W3 — Cut WASM init time (~8 s is too slow for a phone demo)

1. Use `WebAssembly.instantiateStreaming` so compilation overlaps download.
2. Move the definitions blob out of the `.wasm` into a separate `fetch`ed asset if that lets the module
   instantiate before the data finishes loading — the service worker caches both for offline use.
3. Show real progress (download → compile → index ready), not a spinner.
4. If S1b lands, re-measure: the smaller trie should cut both size and init.

**Acceptance:** time-to-first-lookup on a cold cache and on a warm (service-worker) load, measured in a
real browser, before and after. Target < 3 s cold on a normal connection; state what you actually got.

### A3 — Scale headroom (proves day-of capability without taking anything)

Generate a synthetic corpus several times the size of the real RID (~40k headwords), run the full
ingestion path, and report the timing and memory curve at 1×, 2×, 5×, 10×. Put it in the **analytical /
synthetic** section of `BENCHMARKS.md`, clearly separated from real measurements.

Then re-time `COMPETITION_DAY.md` end-to-end against fixtures (R6 measured it; S1b should improve it) and
record the new number.

### E1 — `CountingAllocator`

Per `NEXT_STEPS_R6.md` E1. Copy the ~80-line pattern from
`katgpt-rs/crates/katgpt-dec/tests/common/counting_allocator.rs` (no dependencies) and report the **actual
allocation count** on the hot lookup path. Only claim "zero-alloc" where the counter reads zero.

---

## 4. Order

```
V1 → S1b → D2 → L1 → C → W3 → A1 → A2 → A3 → E1
```

- **V1 first** — it is minutes, and it may be a live correctness bug in the flag a judge would see.
- **S1b is the highest-value task** and everything about the day-of story depends on it.
- **L1 and C are the highest-value *product* work** — they close the two stated ORST objectives we serve
  worst (learning support, and search beyond headword lookup).
- **Droppable if the night runs short:** A3, E1, and A2's second rater.

**Deliverable:** `VERIFY_R8.md` in the established format (status table, raw script output, deviations,
stopped/skipped, known-stale claims), `BENCHMARKS.md` updated, dated `PROGRESS.md` entry. Documentation
staleness stays on the list for the single final pass — do not fix it here.
