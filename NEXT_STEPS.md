# Next Steps — Instructions for the Executing Agent

**You are picking up an in-progress hackathon project.** Read in this order before touching anything:
[`README.md`](./README.md) → [`PROGRESS.md`](./PROGRESS.md) (the living log — read this fully, it's the
source of truth for what's actually true right now) → [`AGENT_HANDOFF.md`](./AGENT_HANDOFF.md) (decisions
and why) → [`PLAN.md`](./PLAN.md) (the original day-shaped schedule — note that reality has run ahead of
it: most of "Day 1 build" already happened during what was nominally Day 0 prep, so treat `PLAN.md`'s
day labels as loosely aspirational, not literal, and prioritize by the task list below instead).

**Do not re-derive or re-litigate any decision in `AGENT_HANDOFF.md` §3's table** (hybrid architecture,
AXIOM `graph.rs`-only, no `katgpt-transformer`, etc.) without new evidence — those are settled. This
document is about what to build next, not what to reconsider.

**Mandatory habit while you work:** update [`PROGRESS.md`](./PROGRESS.md) — its status board and a new
dated log entry — as you complete each task below, or if you change your mind about how to do one. This
is how the person supervising this work (and any other agent) verifies what actually happened without
re-reading everything. Do not just claim something works — the project's own convention (see
`PROGRESS.md`'s 2026-09-05 entry) is to actually run the code and report real output/numbers, because the
last two "done" claims each hid a real bug that only surfaced when someone actually executed the thing
instead of trusting the test suite or a previous log entry. **Follow that convention**: for every task
below, the acceptance criteria include actually running something and pasting/describing real output, not
just "implemented and tests pass."

---

## Task 1 (do first — data-quality/credibility risk) — Audit the 20 seed dictionary entries

**✅ DONE (2026-09-12, verified) — see `PROGRESS.md`.** 3 mislabels found and fixed (ครู/นักเรียน,
อ่าน/เขียน, สุข/ความสุข), new `Relation::RelatedTo` + 2 regression tests added. Task body below kept for
historical context, not something left to do.

**Why this is first:** the whole pitch rests on the relationship graph being *correct*, not just
technically working. One mislabeled relation was already found and not yet fixed:
`ครู --ตรงข้ามกับ--> นักเรียน` ("teacher opposite-of student") is tagged as an antonym relation, but
teacher/student is a complementary role pair, not a true lexical antonym (unlike `ใหญ่ ตรงข้ามกับ เล็ก`,
a real antonym). If a judge from ราชบัณฑิตยสภา (actual lexicographers) sees a mislabeled relation, it
undermines the credibility of the whole "we built this on real dictionary relationships" story.

**What to do:**
1. Open `dict-hackathon/wacha/src/dictionary.rs` and find the seed entries (search for
   `seed_entries` — 20 curated Thai words with relations).
2. Review every relation triple for correctness against real Thai lexical semantics. Specifically check:
   - Every `ตรงข้ามกับ` (antonym) relation is a true antonym, not a related-but-not-opposite pair. Fix or
     relabel `ครู`/`นักเรียน` (e.g. to a `เกี่ยวข้องกับ`/"related role" relation, or just remove it if no
     clean relation type fits).
   - Every `มีความหมายเหมือนกับ` (synonym) relation is a genuine synonym.
   - Every `เป็นชนิดของ` (is-a) and `อยู่ในหมวด` (category) relation is taxonomically correct.
3. If you're not confident in a judgment call, it's fine to leave a relation out rather than guess — fewer
   correct relations beats more relations with one wrong one a judge can catch.
4. Re-run `cargo test` in `dict-hackathon/wacha/` (must still pass — the existing tests reference
   some seed relations by name, e.g. `relations::tests::cat_relates_to_animal_with_explanation`; if you
   remove/change a relation a test depends on, update that test too, don't just delete it to make it pass).
5. Re-run the CLI (`cargo run --release --bin wacha -- --data ../data lookup ครู`, and 2-3 other
   seed words) and paste the actual output into your `PROGRESS.md` entry — confirm the fix is visible in
   real output, not just in the source file.

**Acceptance criteria:** every seed relation is one you'd be comfortable defending to an actual Thai
lexicographer; `cargo test` passes; CLI output for at least 3 words is pasted into `PROGRESS.md` showing
the corrected relations.

---

## Task 2 — Decide and implement a fix for the 42-second cold-start build

**✅ DONE (2026-09-12, verified) — see `PROGRESS.md`.** Went with the recommended serialize-to-disk
option: cold 43.25-43.35s → warm ~10-12ms (~3,500-4,400×). Also: the `katgpt-tokenizer` fixes this task
mentioned turned out to be uncommitted in a repo the user doesn't own, so `Datrie` was vendored directly
into `wacha/src/datrie.rs` instead of staying a path dependency — `wacha` now has zero external
dependencies beyond crates.io (`serde`, `postcard`). Task body below kept for historical context.

**Context:** `Engine::build` takes 42.4 seconds on the real 62,106-word list (confirmed 2026-09-05, see
`PROGRESS.md` and `AGENT_HANDOFF.md` §8). Root cause is in `katgpt-tokenizer`'s `Datrie` collision
resolution, not in `wacha` itself — Thai's narrow UTF-8 byte range causes massive collision cascades
at real-dictionary scale. This is currently mitigated only by the guardrail "never restart the process
during a demo," which works but is fragile (a crash mid-demo would be costly).

**What to do, in this order of preference:**

1. **First, try the low-risk fix: serialize the built segmenter to disk.** After `Segmenter::from_words`
   builds successfully once, save enough state to reconstruct it quickly on the next run (e.g. `serde` +
   `bincode`, or even just caching the sorted `Vec<(Vec<u8>, usize)>` entries isn't enough by itself since
   the 42s is in the trie *construction*, not the sort — you likely need to serialize the built
   `DatrieVocab`'s internal `base`/`check`/`value` arrays directly, or add a save/load path in
   `katgpt-tokenizer` itself if the fields aren't accessible from outside the crate). Check whether
   `Datrie`/`DatrieVocab` derives `Serialize`/`Deserialize` already (`katgpt-tokenizer`'s `Cargo.toml`
   already depends on `serde`) — if not, this may require a small addition to
   `katgpt-rs/crates/katgpt-tokenizer/src/datrie.rs` (adding `#[derive(Serialize, Deserialize)]` where
   possible). **If you touch `katgpt-tokenizer`, that's a shared crate used by the separate, unrelated
   Green Mind AI 2026 track (`neural-engines/mango-a100/`) — keep the change additive (new
   method/derive), never change existing method signatures or behavior**, and note the change in
   `dict-hackathon/PROGRESS.md` the same way the 2026-09-04 `grow_to` fix was noted, since it affects
   other consumers of that crate.
2. **If serialization proves complex or too time-consuming, fall back to documenting and hardening the
   existing mitigation instead of forcing a fix:** make sure the actual demo entry point (CLI REPL mode,
   or the web UI from Task 3 if you build it) builds the engine exactly once at startup and never rebuilds
   per interaction. Add a startup log line printing build time so it's visible/expected rather than
   surprising during a live demo, and add a one-line README note in `wacha/README.md` warning
   against one-shot invocations on the real data.
3. **Do not attempt to fix the underlying Aoe double-array collision algorithm itself** (i.e. don't
   redesign `resolve_collision`/`find_new_base` in `katgpt-tokenizer`) — that's a deeper algorithmic
   research problem, out of scope for hackathon time, and risks breaking a crate another active track
   depends on. Serialization (option 1) or architectural mitigation (option 2) are the right-sized fixes.

**Acceptance criteria:** either (a) a second run of the CLI/engine on the real word list completes in
well under 42 seconds via a cache/serialized load path, with the actual before/after timings pasted into
`PROGRESS.md`, or (b) if you chose the fallback, the demo entry point is confirmed (by actually running
it, not just reading the code) to build once and reuse the engine across multiple interactions without
rebuilding, with that confirmation described in `PROGRESS.md`.

---

## Task 3 (optional, do only if Tasks 1-2 are done and there's time left) — Minimal web UI

**✅ DONE (2026-09-12, verified by real curl requests) — see `PROGRESS.md`.** Built `wacha-web`: a
dependency-free `std::net` HTTP server + embedded single-page UI (`wacha/web/index.html`) over the shared
`Engine::load_from_dir` loader (engine built once at startup, confirmed via the startup log). Task body
below kept for historical context.

Build a thin web frontend over the `Engine` API (`wacha`'s `lib.rs`) so the demo doesn't rely on a
terminal CLI in front of judges. This was flagged as a nice-to-have in `PROGRESS.md`'s 2026-09-04 entry,
never started.

**What to do:**
1. Pick the lightest-weight approach that gets a working demo fast — e.g. a small Rust web server (axum
   or actix-web) wrapping the existing `Engine`, or even a static HTML page hitting a tiny local JSON
   API — don't over-engineer this for a hackathon demo.
2. The UI needs at minimum: a search box, the segmented display, the definition, and the ranked
   related-words list with its explanation path (mirror what the CLI's `print_lookup` already shows).
3. **Critical: build the `Engine` exactly once at server startup, not per-request** (see Task 2) — verify
   this by hitting the server with several different words in a row and confirming each response is fast
   after the initial startup delay.
4. Actually open it in a browser (or at minimum `curl` the endpoints) and confirm real output before
   calling this done — screenshot or paste real output into `PROGRESS.md`.

**Acceptance criteria:** a running local server, demoed with at least 3 real word lookups, with either a
screenshot or pasted terminal output (curl) in `PROGRESS.md`, and confirmation the engine builds once, not
per-request.

---

## Task 4 (now P1 in `AGENT_HANDOFF.md` §7.5 — RESCOPED 2026-09-12, do this one first among what's left)

**✅ DONE (2026-09-12, verified) — see `PROGRESS.md`.** Static asset `wacha/data/learner_content.json`
(20 words) + embedded loader `wacha/src/learner.rs` + `gen-learner` offline generator tool
(Typhoon-2/OpenAI-compatible). Wired into CLI, `wacha-web` JSON, and the web card. **Runtime is
LLM-free.** Provenance is honest: the shipped text is currently `human_seed` (hand-authored — no Typhoon 2
key/access from the build environment), and `gen-learner` regenerates it as `typhoon-2` when run offline
with a key. Task body below kept for historical context.

**Rescoped:** no longer "wire a live Typhoon 2 API call." The organizer's brief (see `AGENT_HANDOFF.md`
§7.5) explicitly wants a feature that helps people learn how to use Thai — this task now targets that
directly, but as an **offline-precomputed static asset**, not a live model dependency, to protect the
project's "modelless at runtime" positioning (`AGENT_HANDOFF.md` §1) and to remove all live-demo failure
risk.

**What to do:**
1. Get API/self-hosted access to Typhoon 2 (SCB 10X) or SEA-LION working end-to-end on one throwaway Thai
   prompt — this happens once, offline, not as part of the shipped binary/demo.
2. For each of the 20 seed words (`wacha/src/dictionary.rs`'s `seed_entries`), generate a plain-language
   definition and/or one example sentence using the model, **offline, once**.
3. Hardcode or embed the generated text as a static data field on each `Entry` (or a small companion JSON
   file loaded at startup) — no network call, no API key, no runtime dependency on the model existing.
   Wire it into the CLI (`lookup`) and `wacha-web`'s `/api/lookup` response as a `simple_explanation`-style
   field.
4. Paste the real generated text for at least 3 words into `PROGRESS.md`, and note in the same entry that
   it was generated offline/once (so a future reader doesn't mistake this for a live integration).

**Acceptance criteria:** every seed word has AI-generated learner-facing text visible through both the CLI
and the web UI, with zero runtime dependency on Typhoon 2/SEA-LION being reachable — the demo works fully
offline. See `AGENT_HANDOFF.md` §7.5 (P1) for the full rationale.

---

## Task 5 (P2 in `AGENT_HANDOFF.md` §7.5) — Expand relation coverage via Thai WordNet

**✅ DONE (2026-09-12, verified on 5 non-seed words) — see `PROGRESS.md`.** Extracted synonym groups from
`wordnet_th.db` (NICT) → `wacha/data/wordnet_synonyms.tsv` (13,664 groups, ~1MB, shipped instead of the
11MB DB) → `wacha/src/wordnet.rs` loader → fed as `Synonym` edges into the graph at build time
(`from_dictionary_with_wordnet`, seed relations authoritative). Graph grew 24→29,281 entities /
65→52,545 triples; ~29k words now return explainable synonyms. Only synonyms extracted (the DB has no
is-a links — not fabricated). Runtime stays deterministic/LLM-free. Task body below kept for context.

**Why:** only the 20 hand-curated seed words have real relations today. A judge searching any of the
other ~62,000 words in the list gets segmentation but no explainable relationships. `wordnet_th.db`
(SQLite, permissive NICT license) was already found and license-checked on 2026-09-04 but never wired in
— this is the single highest-leverage remaining improvement for both data depth and demo robustness.

**What to do:**
1. Inspect `wordnet_th.db`'s schema (find it via the PyThaiNLP corpus download path noted in
   `PROGRESS.md`'s 2026-09-04 entry, or re-fetch it).
2. Extract synonym/hypernym/hyponym relations and map them onto the existing `Relation` enum
   (`wacha/src/dictionary.rs`) — reuse `SeeAlso`/`Category`/`RelatedTo` etc. where they fit; only add a new
   variant if genuinely nothing existing fits (match the care taken in the Task 1 audit — a wrong relation
   label is worse than a missing one).
3. Feed the extracted triples into `graph.rs` alongside the seed entries' triples at `Engine` build time.
4. Verify with real queries on words *outside* the original 20-word seed set — pick 5 words at random from
   `words_th.txt` that weren't in the seed list, run `lookup`, and paste real output into `PROGRESS.md`
   showing they now return explainable related words too.
5. Re-check the cold/warm build-time numbers after this change (more triples = more graph-build work,
   though this is normally cheap compared to the trie) and re-verify the trie cache still round-trips
   correctly.

**Acceptance criteria:** words outside the original 20-word seed set return real, explainable related-word
results; `cargo test` still passes; real query output for 5 new (non-seed) words pasted into `PROGRESS.md`.

## Task 6 (P3 in `AGENT_HANDOFF.md` §7.5, optional polish) — Visualize the relationship graph in `wacha-web`

**✅ DONE (2026-09-12, verified in a real browser) — see `PROGRESS.md`.** Added a client-side SVG radial
graph to `wacha/web/index.html` (`buildGraphSvg`): query word centered, related words as clickable nodes
(distance/size by score), edges labeled with the relation from the explanation path; click/Enter a node to
re-query. Text list kept as the accessible fallback. No backend change. Task body below kept for context.

**Why:** the current related-words panel is a ranked text list with path strings. A small interactive
graph (nodes + edges, click to re-center on a related word) communicates the "explainable AI reasoning"
story far more viscerally to judges, and directly serves the brief's own "เห็นภาพ" (visualize) language.

**What to do:**
1. Client-side only — `/api/lookup`'s JSON already carries every related word's score and explanation
   path; no backend change should be needed.
2. Keep it simple (an SVG or `<canvas>` force-directed-ish layout, or even a simpler radial layout with the
   query word at the center) — this is a hackathon demo enhancement, not a general graph-visualization
   library.
3. Verify by actually loading it in a browser and clicking through a few words, not just reading the code.

**Acceptance criteria:** a real, working, clickable graph visualization verified in an actual browser
session, described (or screenshotted) in `PROGRESS.md`.

---

# Round 3 (2026-09-12, later) — win-readiness: correctness risk, pitch, demo, real-world story

Tasks 1-6 above are all done and verified (see `PROGRESS.md`) — the core product (correct seed relations,
fast start, CLI + web with an explainable relationship graph, ~29k-word WordNet-expanded coverage, offline
learner content) is demo-complete. **This round is not about building more product** — it's about what
actually determines whether this wins: closing a real credibility risk found during verification, and the
non-code work (pitch, rehearsal, the "real-world use" story) that hasn't been touched at all yet.

**Do not re-litigate anything in `AGENT_HANDOFF.md` §1/§3/§7.5** (the modelless/deterministic positioning,
the hybrid architecture, the WordNet-expansion decision) — those are settled. This round works within that,
it doesn't reconsider it.

## Task 7 (P0 — do this first, highest priority of everything below) — Confidence-tag WordNet relations + quantify their real error rate

**✅ DONE (2026-09-12, verified live + real measurement) — see `PROGRESS.md`.** Degree-based
`RelationConfidence {Confirmed, Unverified}` (isolated both-degree-1 WordNet pairs → Unverified; seed
always Confirmed) live in CLI + web JSON + web UI (⚠ marker + legend). Verified: `ข้อหา/มลทิน`=Unverified,
`สุนัข/หมา`=Confirmed. Real precision: random 120-pair sample (seed 20260912) manually judged = **84.2%**
genuine synonyms (Confirmed 85.1% vs Unverified 81.8% — signal separates only weakly; the honest defense
is provenance + confidence labeling, not a claim that the signal is a strong classifier). Task body below
kept for context.

**Why this is first:** verification on 2026-09-12 found a real, judge-visible correctness problem: the
mass-imported Thai WordNet synonym data (`wordnet_synonyms.tsv`, ~13,664 groups) contains at least one
confirmed bad pair — `ข้อหา` (accusation/charge) linked as a "synonym" of `มลทิน` (moral stain/blemish),
which are not synonyms in ordinary Thai. This traces to Thai WordNet's own source-data imprecision (a
cross-lingual mapping artifact from Princeton WordNet), not a bug in `wacha`'s code — the graph mechanically
reproduces whatever the source data says, with zero judgment layer (see `PROGRESS.md`'s discussion of how
the relationship graph "connects" words: pure graph math over facts, no semantic evaluation of any kind).
If a judge free-types a word and hits a pair like this, it undermines the entire "explainable and correct"
positioning that this project is built around. This is fixable without touching the core architecture.

**What to do:**
1. **Add a confidence signal using data the graph already has — no new data acquisition needed.** For each
   WordNet-derived relation, compute the degree (in `graph.rs`, `adjacency_of(entity_id).len()`) of both
   endpoints. A pair where **both** endpoints have degree 1 (i.e., their only connection in the whole graph
   is to each other, with no corroborating evidence from any other synset or seed relation) is
   structurally uncorroborated and should be flagged as lower-confidence. This was verified during
   analysis: `ข้อหา`/`มลทิน` is exactly this shape (an isolated 2-node pair), while a good pair like
   `สุนัข`/`หมา` is not (both appear in multiple WordNet groups). Confirm this predicts well on a handful of
   further known-good and known-bad pairs before wiring it in broadly.
2. **Surface the confidence signal, don't just compute it silently.** Add it to the `RelatedWord`/relation
   data returned by `relations.rs` (a `confidence: Confirmed | Unverified` -style field, or similar — match
   the existing code's style, don't over-engineer this into a numeric score). Show it in both the CLI
   output and `wacha-web`'s JSON/UI — a small, honest marker (e.g., a subtle label or icon on low-confidence
   relations is enough; this is not meant to be alarming, just honest) is sufficient. Seed-sourced relations
   (the 20 hand-curated words) should always read as fully confirmed — this signal is specifically for the
   WordNet-derived tier.
3. **Quantify the real error rate with an actual random sample, not just the one pair already found.**
   Randomly sample ~100-150 pairs from `wordnet_synonyms.tsv`, manually judge each as a genuine synonym or
   not (use real Thai-language judgment, not a heuristic), and compute the resulting precision estimate.
   Record the exact sample, the judgment for each, and the resulting percentage in `PROGRESS.md` — this
   number is for the pitch (Task 8), so it must be a real, defensible measurement, not an estimate.
4. Re-run `cargo test` (must still pass) and verify live: query a known-bad-shape pair (or a newly found
   one from the sample) and confirm it's marked low-confidence; query a well-corroborated pair (e.g.
   `สุนัข`/`หมา` or `ครู`/`อาจารย์`) and confirm it is **not** flagged.

**Acceptance criteria:** the degree-based confidence signal is live in both CLI and web output, verified
against at least one known-bad and one known-good real pair; a real random-sample error-rate measurement
(with the actual sample and judgments) is recorded in `PROGRESS.md`.

## Task 8 — Pitch materials

**Why:** the positioning in `AGENT_HANDOFF.md` §1 (modelless, deterministic, explainable — "not an AI
chatbot wearing a dictionary costume") is strong but currently only exists as internal documentation.
Nobody has written down what to actually say to judges.

**What to do:**
1. Write a short (2-3 minute spoken) pitch script in Thai, as a new file `dict-hackathon/PITCH.md`. Open
   with the positioning statement from `AGENT_HANDOFF.md` §1 — that's the differentiator, lead with it, not
   bury it. Cover: the problem (per the organizer's brief and `AGENT_HANDOFF.md` §7.5's scorecard), the
   hybrid architecture in one sentence each, and the honest scope (offline-precomputed AI, WordNet-derived
   relations at a measured confidence level from Task 7 — don't hide the limitation, state it as evidence
   of rigor).
2. Prepare honest, specific answers (not deflections) to the hardest likely questions: "why not just use
   an LLM for everything," "how do you know the WordNet data is reliable" (this is where Task 7's real
   number goes), "what would it take to actually deploy this." Write these as a Q&A section in the same
   file.
3. Pick 4-6 specific words to search live during the demo — a mix that shows the hand-curated seed
   richness (e.g. `ครู`), a good WordNet-expanded example (e.g. `รถยนต์` or `สุนัข`), and one word chosen to
   preempt "let me try my own word" by demonstrating the confidence tagging from Task 7 honestly. List
   these with their actual expected output in `PITCH.md` so whoever presents doesn't have to guess live.

**Acceptance criteria:** `PITCH.md` exists with a complete script, a Q&A section with real (not
hand-wavy) answers, and a specific demo word list with real, pre-verified output for each.

## Task 9 — Demo rehearsal and robustness

**What to do:**
1. Start `wacha-web` fresh (cold, from a deleted cache, timed) at least once to confirm the ~43s number is
   still accurate, then confirm the warm/cached path is what will actually be used at demo time — the
   server must be started well before presenting, never live.
2. Run a real battery of adversarial queries against the running server and record actual results: empty
   query, a very long string, non-Thai input (English, numbers, emoji), a single stray byte/invalid UTF-8
   if easy to construct, and a word from the Task 7 sample known to be low-confidence. None of these should
   crash the server or hang — confirm this by actually doing it, not by code review.
3. Produce a fallback: a screen recording (or at minimum a sequence of screenshots) of one full successful
   demo run, in case live network/hardware fails on the day. Note where this is saved.

**Acceptance criteria:** a documented, real adversarial-query test session with actual results (not
predictions) in `PROGRESS.md`; a fallback recording/screenshot set exists and its location is noted.

## Task 10 — Package the open-data/real-world story

**Why:** answers the organizer's brief's "build networks" and "promote open data" objectives concretely,
and gives a real answer to "would this ever actually get used" beyond the demo.

**What to do:**
1. Write a short, developer-facing section (in `wacha/README.md` or a new `wacha/API.md`) documenting the
   `/api/lookup` JSON shape as a stable-enough public contract, and explicitly stating the license of every
   derived data asset shipped (`words_th.txt`, `wordnet_synonyms.tsv`, `learner_content.json` — all
   CC0/NICT-permissive per existing provenance notes) so a third party could realistically reuse them.
2. Reference this section directly in `PITCH.md`'s answer to "how would this get used after the hackathon."

**Acceptance criteria:** the documentation exists, is accurate (spot-check the JSON shape against a real
`/api/lookup` response), and `PITCH.md` references it.

## Task 11 (optional, lowest priority — only if time remains after 7-10)

1. **✅ DONE — the `--host`/Tailscale change is committed (`bec54b8`, 2026-09-12).** Finished properly:
   `--host ADDR` flag (default `127.0.0.1`), security warning on non-localhost bind, verified live binding
   to the Tailscale IP `100.76.70.14`. Not dangling. (This item predated that commit.)
2. If real Typhoon 2 (or SEA-LION) API access becomes available, run `gen-learner` for real and confirm the
   asset's `source` field updates from `human_seed` to `typhoon-2` — verify at least 3 words' new output
   before considering this done, same as every other task in this project.

---

## When you're done (or stopping partway)

1. Update `PROGRESS.md`'s status board for every task above (done / not started / in progress with what's
   left).
2. Add a dated log entry describing what you did, what you found, and any new decisions — following the
   style of the existing 2026-09-04 and 2026-09-05 entries (be specific: real numbers, real output, real
   bugs found — not summaries like "implemented successfully").
3. If you found a new bug or made a new architectural decision not covered by this file or
   `AGENT_HANDOFF.md`, add it to `AGENT_HANDOFF.md` §3 (decisions) or §8 (guardrails) as appropriate — this
   file (`NEXT_STEPS.md`) is a one-time task list, not a living document; don't let it silently go stale,
   update the living docs instead.
4. Leave the repo in a state where `cargo test` passes in both `poc/` and `wacha/` — don't hand back
   a broken build.
