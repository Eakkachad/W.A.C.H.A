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
