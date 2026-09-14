# Round 6 — Correctness Fixes, the Speed Weapon, and One New Capability

**Written 2026-09-14 by the reviewing agent.** Run unattended, same rules as `NEXT_STEPS_R5B.md`:
never silently substitute an approach, commit per task, every claimed number must come from a script in
the repo, and **do not touch `katgpt-rs`**.

---

## 0. Measured baseline (2026-09-14, verified by the reviewer — beat these, don't re-derive them)

| Metric | Now |
|---|---|
| Cold build (trie from 72,175 words) | **57.8 s** |
| Warm start | **1.419 s** — of which **1.034 s is the segmenter cache load** |
| ↳ Kaikki JSONL parse (82 MB) | 249 ms |
| ↳ ศัพท์บัญญัติ parse | 1 ms |
| p95 lookup latency (warm, via HTTP) | 13.6 ms |
| Trie cache size | 7,395,138 bytes |
| Graph | 57,061 entities / 74,215 triples / 29,759 sense nodes |
| Tests | 85 + 4 |

**Two hypotheses were tested and one was killed — do not chase it:**
- ❌ **`next_cluster_end` is O(n²)** — *refuted by measurement.* Segmenting pure-OOV strings takes
  1.34 s at 1,000 chars and 1.35 s at 16,000 chars. Flat. The time is all start-up. **Do not "optimize"
  this.**
- ✅ **Start-up is dominated by cache validation, not deserialization** — confirmed, see P1.

---

# PHASE P — Correctness first (blocking; nothing else ships without these)

## P1 — `vocab_hash` sorts 72k words on every start-up (1.0 s → target < 30 ms)

`segmenter.rs:43` documents `vocab_hash` as *"order-independent: words are sorted & hashed"*. Sorting
72,175 Thai strings on every process start is the single largest start-up cost — larger than parsing the
82 MB Kaikki file.

**Fix:** make the hash order-independent *without* sorting — combine per-word FNV-1a hashes with a
commutative operation (wrapping-add and/or XOR, mixed with the word count and total byte length to keep
collision resistance). O(n), no sort, no intermediate `Vec`.

**Acceptance:** start-up breakdown printed before/after; the cache-invalidation tests from R5 Task 5
still pass (change a word → rebuild; unchanged → cache hit). Target < 30 ms for the validation step.

## P2 — Ranking inversion: sparse Wiktionary pairs outrank corroborated WordNet synsets

**The bug.** `lookup บ้าน` returns 8 results, **all Wiktionary, all with the identical score 15.653**:
`บ้านเรือน, คาม, อาลัย, เคหะ, เหย้า, กว้าน, เคหสถาน, คห`. Meanwhile **`เรือน` — corpus frequency
3,469, the most common synonym, and a member of a genuine 4-word WordNet synset with `บ้าน`
(`wordnet_synonyms.tsv:5124`: `นิวาสถาน / บ้าน / บ้านช่อง / เรือน`) — does not appear at all**, beaten
by `คห` (frequency 39).

**Root cause.** `relations.rs:178` builds every Kaikki synonym pair as its own 2-member sense group. A
2-member group concentrates all PPR mass on one neighbour; a 4-member WordNet synset spreads it across
four. So **the less corroborated the evidence, the higher the score** — the exact inverse of what we
want, and a direct contradiction of our own confidence story, which flags isolated pairs with ⚠ as
*less* trustworthy while the ranking now puts them first.

The frequency tiebreaker works (ordering within the tie is 1030 → 341 → 325 → … → 76 → 39) but it
cannot help: `เรือน` is eliminated by PPR before the tiebreak is reached.

**Fix — implement and report both, then keep the better one:**
- **(a) Tiered ranking:** rank by source tier first — Seed > CoinedWord > WordNet > Kaikki — then by PPR
  within each tier. Simple, predictable, easy to explain on stage.
- **(b) Corroboration-weighted PPR:** damp a sense group's contribution by its provenance strength so a
  2-member unaudited pair cannot outscore a multi-member attested synset. Keeps one continuous ranking.

Whichever ships, **`เรือน` must appear in the top 3 for `บ้าน`**, and `ครู`/`ครอบครัว` must not regress.

**Acceptance:** `lookup บ้าน`, `ครู`, `ครอบครัว`, `รถยนต์`, `สุนัข` pasted before/after; a test named
`corroborated_outranks_isolated` asserting `เรือน` outranks `คห` for `บ้าน`; `cross_sense_pairs` still 0;
KEEP-recall still 40/40.

## P3 — Stale numbers and one over-claiming label

1. **`PITCH.md:69`** still says *"cold ~43 วิ ครั้งเดียว → warm ~10 มิลลิวินาที"*. Measured: cold 57.8 s,
   warm 1.419 s. `BIBLE.md` was updated in C2; this line was missed. Update it from `verify_r5.sh`
   output **after** P1 lands, so the number is the improved one.
2. **Kaikki sense-group label over-claims.** The UI prints `(ผ่านชุดความหมายเดียวกัน: บ้าน-คาม)` — the
   same wording used for real synsets `(ผ่านชุดความหมายเดียวกัน: 08078020-n)`. But Kaikki has **zero**
   sense-scoped synonyms (verified: 0 of 34,392 entries); we invented that grouping. The *implementation*
   is right (2-member groups are traversal-terminal, as specified), only the wording is wrong. Change the
   Kaikki case to something like `(คำพ้องระดับคำ — Wiktionary ไม่ได้ระบุว่าเป็นความหมายใด)`.
3. **Narrow gold set.** Note in `VERIFY_R5.md`/`BIBLE.md` that `KEEP_recall = 40/40` is measured on 47
   seed-adjacent pairs and therefore does **not** capture losses like `รถยนต์ → ยานยนต์`. State the
   scope next to the number.

---

# PHASE S — The speed weapon (high value; this is what makes the demo unbeatable)

**Why this matters beyond vanity.** On competition day we ingest the organizers' real dataset. If a cold
build drops from ~58 s to a few seconds, we can do what no other team can: **take the judges' data file
on stage and load it in front of them.** `COMPETITION_DAY.md` and the `Importer` trait already exist —
speed is the only missing piece. That turns an engineering fix into the closing demo beat.

## S1 — Dense Thai alphabet for the double-array trie (the root cause of 57.8 s)

**Diagnosis, now confirmed from the source.** `katgpt-rs/crates/katgpt-tokenizer/src/datrie.rs` (which
`wacha/src/datrie.rs` is vendored from) is **byte-keyed with no alphabet reduction**: `children_at` and
`reparent_children` both hard-code `for byte in 0..256`, so every collision resolution does a 256-wide
scan *and* allocates a `Vec::with_capacity(256)`. Thai in UTF-8 makes this pathological — every Thai
character starts with `0xE0` and the second byte is only `0xB8` or `0xB9`, so at depths 1–2 the trie has
a branching factor of ~2 and collisions cascade.

**The fix.** Remap characters to a dense symbol alphabet before insertion:

1. At construction, scan the vocabulary and assign every distinct `char` a dense symbol id
   (`u8`/`u16`) — Thai is ~87 characters, so with digits/Latin/punctuation the alphabet stays small.
   Store the mapping as a `HashMap<char, u16>` plus a reverse `Vec<char>`.
2. Reserve id `0` as **"unseen character"**, which must never match any transition. Two distinct unseen
   characters must not collide into the same matching symbol — otherwise you invent false dictionary hits.
3. Encode a query once into `(Vec<u16> symbols, Vec<u32> byte_offsets)`; run `longest_prefix` over
   symbols; map the match end back to a byte offset via `byte_offsets`. This keeps `Token` byte spans
   exactly as they are today, so TCC/OOV handling is untouched.
4. Narrow the `0..256` scans to `0..alphabet_len` and reuse a single scratch buffer instead of allocating
   a `Vec` per collision.

**Expected:** Thai text goes from 3 bytes/char to 1 symbol/char — trie depth ÷3, branching ~2 → ~90.
Cold build, cache size, and lookup should all improve substantially.

**⚠️ Spike before committing.** Build the remap behind a feature flag or a second constructor, measure
cold build on the real 72,175-word list, and only then replace the byte path. **If the measured cold
build is not at least 3× faster, stop, keep the byte path, and report the numbers** — do not ship a
half-migration.

**Correctness gate (non-negotiable):** for all 72,175 vocabulary words plus the `PITCH.md` demo
sentences, the new segmenter must produce **byte-identical output** to the current one. Write that as a
differential test. A faster segmenter that segments differently is a regression, not an optimization.

**Acceptance:** before/after table — cold build, warm start, cache size, p95 lookup; differential test
green; cache format version bumped so stale caches rebuild (R5 Task 5 machinery).

## S2 — Cache the global PageRank vector

Global PageRank is query-independent but recomputed at every engine build (part of the remaining ~1.1 s).
Serialize it next to the trie cache, keyed by a hash of the graph (entity + triple counts plus a content
hash), and load it instead of recomputing. Invalidate exactly like the trie cache.

**Acceptance:** warm-start breakdown before/after; a test proving a graph change invalidates the vector.

## S3 — CSR adjacency for the relation graph *(only if S1 and S2 are done and measured)*

If PageRank still dominates start-up, convert graph adjacency to CSR — `offsets: Box<[u32]>`,
`neighbors: Box<[u32]>`, two-pass build (count degrees → prefix-sum → scatter with a per-row cursor).
Reference implementation to study: `katgpt-rs/crates/katgpt-core/src/signed_coupling.rs:158-250`, which
also interleaves a parallel payload array so one pass touches each cache line once.

**⚠️ Read this before parallelizing anything.** `katgpt-rs/crates/katgpt-core/src/linalg/symmetric_eig/par.rs:14-26`
documents a **measured 13–62× slowdown** from naively applying rayon to many small operations; the fix
was to batch first, then make one parallel pass. `katgpt-core/src/karc/batched.rs:20-31` likewise rejects
rayon because ~5 µs dispatch overhead exceeded a 575 ns budget. **Measure before and after; if rayon does
not win, say so in the benchmark report and keep the serial path.** A documented negative result is a
perfectly good outcome.

---

# PHASE C — One new capability: reverse dictionary (ค้นคำจากความหมาย)

**Why this one.** We now have **29,601 definitions** — yesterday we had 20, so this was impossible then.
The ORST brief asks for *"ค้นได้มากกว่าการเปิดหาความหมาย"* and *"เป็นมากกว่าพจนานุกรม"*. This answers it
literally, stays 100% modelless and deterministic, and is explainable by construction.

**What it does.** The user types a description — `สัตว์เลี้ยงสี่ขาเห่าได้` — and the system returns
candidate words (`สุนัข`, `หมา`), showing **which definition words matched** as the explanation.

**Design.**
1. Segment every definition **with our own segmenter** (a nice architectural point for the pitch: the
   dictionary reads itself).
2. Build an inverted index `token → posting list of entry ids`, stored CSR-style (`offsets: Vec<u32>` +
   `ids: Vec<u32>`), not a `HashMap<String, Vec<u32>>`.
3. Score with BM25 (k1 = 1.2, b = 0.75 — state the parameters in `BIBLE.md`; ~40 lines, no dependency;
   **note that katgpt-rs has no BM25, so this is our own code — do not claim otherwise**).
4. Return the matched tokens per hit so the UI can highlight *why* each candidate matched.
5. Wire into CLI (`wacha reverse "<ความหมาย>"`) and `/api/reverse` with the same provenance/licence
   badges as `/api/lookup`.

**Acceptance:** at least 5 hand-checked queries with real output pasted, including
`สัตว์เลี้ยงสี่ขาเห่าได้` → `สุนัข`/`หมา` in the top 5. Report index build time, index size, and p95
reverse-query latency. If a query returns nothing useful, report it honestly rather than tuning the demo
set to hide it.

---

# PHASE E — Make the engineering claims verifiable

## E1 — `CountingAllocator` to prove zero-allocation claims

Copy the pattern from `katgpt-rs/crates/katgpt-dec/tests/common/counting_allocator.rs` (~80 lines,
`#[global_allocator]` + `AtomicUsize` counters + an `alloc_delta(f)` helper, no dependencies). Use it to
**prove** the hot lookup path's allocation behaviour, then state the measured number. Do not claim
"zero-alloc" anywhere the counter does not read zero — report the actual count.

## E2 — `BENCHMARKS.md` in the katgpt-rs house style

`katgpt-rs/.benchmarks/` uses a consistent, honest template worth adopting: header (date / config /
what is gated) → executive summary → criteria table (threshold / result / pass-fail) → before-after table
with ratios → documented warmup, iteration count and seed → **and a hard separation between *measured*
and *analytical* (computed, not measured) numbers**.

Produce `dict-hackathon/BENCHMARKS.md` in that shape covering: cold build, warm start, p95 lookup,
reverse-query latency, index sizes, and allocation counts — each with before/after and the command that
produced it. This is the artifact to hand a technical judge who asks "how do you know?".

---

## Order, and what to drop if the night runs short

**P1 → P2 → P3 → S1 → S2 → C → E1 → E2 → S3.**

- **P1–P3 are mandatory.** A demo that ranks `คห` above `เรือน`, or a pitch quoting timings that are 140×
  off, loses more than any optimization gains.
- **S1 is the highest-value engineering work** and the one with a real story attached.
- **Phase C is the highest-value *product* work.** If you must choose between C and S3, choose C.
- **S3 is explicitly optional.** Do not start it unless S1 and S2 are measured and committed.

**Stop rules:** S1 stops if the spike is under 3×. S3 stops if rayon does not beat serial. In both cases
keep the existing path, commit nothing, and write the measurements into the report — that is a result,
not a failure.

**Final deliverable:** update `VERIFY_R5.md` (or a new `VERIFY_R6.md`) with the same structure as before —
status table, raw script output, deviations, stopped/skipped tasks, known-stale claims — plus
`BENCHMARKS.md`, and the usual dated `PROGRESS.md` entry with its status board refreshed.

---

# ADDENDUM (2026-09-14) — Three phases that other teams structurally cannot match

Added after a scope discussion. **Read the honesty note at the bottom before planning your night** —
this addendum is deliberately larger than one night, and attempting all of it produces a half-broken
repo, which is worse than finishing less.

## Framing: what "unbeatable" actually means for THIS audience

The judges are ราชบัณฑิตยสภา lexicographers plus NECTEC technical staff. That shapes everything:

- **Nanosecond lookups win nothing here.** p95 is already 13.6 ms — imperceptible. Further micro-
  optimization of the query path is effort spent where no judge can see it. **Do not chase it, and do not
  claim it.**
- **"Massive scale" is not claimable.** 72,175 words is a small corpus. A scaling story would be
  transparently hollow to a NECTEC engineer. The defensible performance claims are **ingestion speed**
  and **footprint**, because both have a purpose (below).
- **NECTEC built the RID platform and LEXiTRON** (our word list comes from LEXiTRON; the RID site's own
  disclaimer credits NECTEC as platform author). So: credit LEXiTRON/NECTEC explicitly in the pitch, and
  expect at least one person in the room who knows the PyThaiNLP/AttaCut segmentation benchmarks by
  heart. **That raises the priority of D1** (`NEXT_STEPS_R5.md` Task 11 — a real measured boundary-F1):
  for this audience it is credibility table stakes, not an optional extra.

---

## PHASE W — Ship the whole thing as offline WASM (the flagship)

**Why this is the one.** Every competing team will demo a web app calling an LLM API. We compile the
entire engine to WebAssembly and run it **in the browser tab, with no backend, no GPU, and no network
after first load**. That is not a performance brag — it is the "modelless, deterministic, explainable"
thesis made physical:

- A judge can open it **on their own phone**, turn on airplane mode, and it still works.
- ORST could host it on static hosting (GitHub Pages) for **zero baht/month, forever** — a real answer
  for a government agency with no ops budget, and a direct answer to the brief's "ต่อยอด/สร้างเครือข่าย".
- Teams built on an LLM API **cannot do this at all.** It is structurally out of reach for them.

**How.**
1. `wasm-bindgen` cdylib crate wrapping `Engine` — expose `segment`, `lookup`, and `reverse` (Phase C).
   Working template to study: `katgpt-rs/crates/katgpt-moka-wasm/` is a minimal wasm-bindgen cdylib
   deliberately isolated from heavy workspace deps. Study the Cargo.toml shape; **do not depend on it.**
2. `wacha` must stay `no_std`-friendly at the edges: gate every `std::fs` / `std::net` path behind
   `#[cfg(not(target_arch = "wasm32"))]`. Data arrives as a `&[u8]` the JS side fetches.
3. Load the prebuilt artifact (Phase M) with `fetch` + streaming, and show a real progress indicator —
   honesty applies to loading states too.
4. Ship as a PWA with a service worker so a second visit is fully offline and installable.

**Acceptance:** built `.wasm` size (gzipped) reported; page loaded in a real browser with **DevTools
network throttled and then disabled**, with lookup still working — describe exactly what you saw;
time-to-first-lookup on a cold cache; confirmation that `lookup ครู` in WASM is **byte-identical** to the
native CLI (differential test across the `PITCH.md` demo words).

**Stop rule:** if the artifact cannot be brought under ~25 MB gzipped, ship WASM with a **reduced
dataset** (e.g. the 30k defined words only) and say so plainly in the UI and the pitch. A smaller honest
build beats a broken big one.

---

## PHASE N — Cross-source corroboration, measured (the novelty)

**This replaces P2's option (b) and is the principled version of the ranking fix — not extra scope.**

We now have **three independent sources**: Thai WordNet (NICT), Wiktionary (Kaikki), and ศัพท์บัญญัติ
(ORST). That enables something we could not do yesterday and most teams will never attempt:

> **Treat agreement between independent sources as a measurable confidence signal, and report how well it
> actually predicts correctness.**

The existing degree-based signal was honestly reported as weak (Confirmed 85.1% vs Unverified 81.8% —
barely separating). Cross-source agreement should separate far better, and **if it does not, that is
still a publishable-quality honest result** and must be reported as such.

**What to do.**
1. Tag every relation with the **set** of sources attesting it, not a single source.
2. Define tiers: `ORST-attested` > `multi-source agreement (≥2)` > `single-source` > `isolated pair`.
3. Use the tier as the corroboration weight in ranking (this is what fixes P2 properly: `เรือน`, attested
   by a 4-member WordNet synset, must outrank `คห`, an isolated Wiktionary pair).
4. **Measure it.** Stratified random sample, ~40 pairs per tier, hand-audited, with a **fixed random
   seed** so it is reproducible. Report precision per tier as a table.
5. Report the **overlap statistics** between the three sources — how many relations each contributes
   uniquely, how many are corroborated. That table alone is a genuinely interesting artifact about Thai
   lexical resources that, as far as we know, nobody has published.

**Acceptance:** the precision-by-tier table with n per tier and the seed; the source-overlap table; the
ranking change demonstrated on `บ้าน`. **Do not tune the tiers after seeing the audit results** — decide
the tier definitions first, then measure. If you want to revise them afterwards, report both the
pre-registered and the revised numbers and label which is which.

---

## PHASE M — Footprint engineering, with a purpose

**Only meaningful because Phase W needs it.** Do not frame this as "we use little RAM" — frame it as
"the entire Thai dictionary, relation graph and search index fit in a browser tab."

Current payload is far too large for the web: 7.4 MB trie cache + an 82 MB Kaikki JSONL.

1. **One prebuilt binary artifact** — trie + dictionary + graph + global PageRank + reverse index,
   built offline by a tool, loaded with a single `fetch`. No JSON parsing at runtime.
2. **Zero-copy load.** Lay the artifact out so it can be used directly from the loaded byte buffer
   (`bytemuck`-style casts over `&[u8]`), rather than deserializing into owned structures. Reference
   pattern: `katgpt-rs/src/kimi_k3/loader.rs:816-877` (mmap → `bytemuck::cast_slice`) and
   `crates/katgpt-transformer/src/mtp.rs:164-177` (`pod_collect_to_vec` bulk cast) — note the first is
   mmap-for-the-read, not end-to-end zero-copy, so read both before choosing.
3. **Compress the text.** Front-coding for the sorted word list (shared prefixes are huge in Thai);
   definitions in one blob with a `u32` offset table. Report bytes saved per technique.
4. **Report a real footprint table:** artifact size (raw + gzip), peak RSS native, peak JS heap in the
   browser. Measure, do not estimate.

**Stop rule:** stop when the artifact fits the Phase W budget. Compression beyond that point is effort
no judge will see.

---

## Honest scoping — read this before you start

**This addendum plus the original R6 is roughly 20+ hours of work. You have one night.** Attempting all
of it will leave several half-finished phases and a repo that is worse than if you had done less. That
outcome is explicitly not wanted.

**The spine, in priority order:**

```
P1 → P2/N → P3 → S1 → W   ← finish this and the project is in excellent shape
```

**Then, if time genuinely remains:** C (reverse dictionary) → M (only as far as W needs) → D1 (measured
segmentation F1 — high value for a NECTEC audience) → E1 → E2 → S2 → S3.

**Explicitly droppable:** S2, S3, E2, and all of M beyond what W requires.

**If you reach a point where a phase cannot be finished cleanly, stop that phase, commit what is green,
and write it up.** A repo where four phases are complete and three are listed as "not started, here's
why" is a strong result. A repo with seven phases half-done is not.
