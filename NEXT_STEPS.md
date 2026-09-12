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

## Task 4 (optional, lowest priority) — Wire Typhoon 2 for AI-simplified definitions

This is "direction 3" from `AGENT_HANDOFF.md` §7 — explicitly optional, cut first if time runs short.

**What to do:**
1. Get API/self-hosted access to Typhoon 2 (SCB 10X) or SEA-LION working end-to-end on one throwaway Thai
   prompt first, in isolation, before integrating.
2. Add a single, well-tested call path in `wacha` that takes a formal RID-style definition and asks
   the model for a plain-language/example-sentence version. Treat this as one API call wrapped in proper
   error handling (timeout, failure) — **per `AGENT_HANDOFF.md` §8's guardrail, this must never hang or
   crash the demo if the call is slow or fails.**
3. Test with at least 3 real definitions and paste real model output (not a mock) into `PROGRESS.md`.

**Acceptance criteria:** a real, working call to a real Thai LLM, with a graceful fallback path if the
call fails, demonstrated with 3 real examples pasted into `PROGRESS.md`.

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
