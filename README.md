# วาจา (WACHA) — Dictionary Reimagined Hackathon (เปิดคลังคำ พลิกคลังคิด)

**วาจา (WACHA — Word Architecture, Cluster-aware Hybrid Analysis):** a hybrid Thai dictionary prototype —
katgpt-rs's tokenizer reads the dictionary, AXIOM's knowledge-graph engine explains word relationships —
for the ORST "เปิดคลังคำ พลิกคลังคิด" hackathon. วาจา is a real Thai word for "speech / word / utterance"
(as in สัตย์วาจา, "word of honor"); every letter of the backronym maps to a real, verified piece of the
system (see `wacha/README.md` for the full breakdown), not a marketing label.

Status: **Round 12 done (2026-09-15) — real official data ingested, job-menu front door + deterministic
intent detection shipped, deployed to the team's Tailscale network for the event.** `wacha/` now runs on
**76,649 words / 40,681 entries** (the real ORST RID 2554 excerpt, official transliteration list, 3
specialized-domain dictionaries, and a 3-edition word-evolution timeline — not just the original CC0
practice data), passes **126 tests** (121 lib + 4 poc + 1 alloc), and answers a query in **~14 ms** (p95).
The product concept has grown from "search a word" to **"วันนี้อยากให้ภาษาไทยทำอะไรให้คุณดี"** — one
free-text box, a rule-based intent router (no LLM), and 5 task-shaped modes (naming from real etymology,
register/rhyme-aware writing help, cross-discipline specialized terms, root/evolution exploration, official
transliteration). Pitch materials (`PITCH.md`, `PITCH_DECK.md`, `MENTOR_BRIEF.md`) are current as of Round
12. Full dated history: `PROGRESS.md`. Round-by-round task lists and verification reports live in
[`rounds/`](./rounds). This folder exists so any agent or session (including a fresh one with no memory of
how this plan was made) can pick the work up cold — that is the explicit purpose of this file and the
others below.

---

## Read these first, in order

0. **[BIBLE.md](./BIBLE.md)** — the complete, self-contained reference: problem background with full
   citations, solution rationale, architecture (with a Mermaid diagram), every mathematical/algorithmic
   principle used explained in detail, verified results, and an honest impact/novelty assessment. Read
   this first if you want the whole picture in one file; the rest below are the living/working documents.
1. **[AGENT_HANDOFF.md](./AGENT_HANDOFF.md)** — mission, the hybrid architecture and *why* each decision
   was made (with what evidence), guardrails (mistakes already ruled out — don't re-litigate these without
   new evidence), where every source file lives.
2. **[PLAN.md](./PLAN.md)** — the day-by-day phased plan (Day 0 prep → Day 1 build → Day 2 demo/submit).
3. **[PROGRESS.md](./PROGRESS.md)** — living log, dated entries, a status board at the top. **Update this
   every working session** — it's the single source of truth for what's actually been done vs. just
   planned. Read it before AGENT_HANDOFF.md/PLAN.md if you only have time for one file — it tells you
   whether either of those has drifted from reality.
4. **[NEXT_STEPS.md](./NEXT_STEPS.md)** — if you were handed this project to execute on, **this is your
   task list**: concrete, prioritized, self-contained instructions with acceptance criteria. One-time task
   list, not living — once done, fold anything durable into `AGENT_HANDOFF.md`/`PROGRESS.md`.
5. **[wacha/](./wacha)** — the **built product** (Rust library + CLI). This is the real
   vertical slice, not the POC. Run `cargo test` and
   `cargo run --release -- --data ../data lookup แมว` inside `wacha/` to see the full journey
   (segment → define → explainable related words) on the real 62k-word list. **Building the engine takes
   ~42s the first time** (see `AGENT_HANDOFF.md` §8) — that's expected, not a hang; don't kill it early.
6. **[poc/](./poc)** — the original feasibility proof-of-concept code (Rust), kept for provenance —
   superseded by `wacha/` for anything except historical reference.
7. **[PITCH.md](./PITCH.md)** — the demo/presentation script (Thai): 2–3 min pitch leading with the
   positioning, honest Q&A for hard questions, and a pre-verified demo word list with real output.
   **[PITCH_DECK.md](./PITCH_DECK.md)** — the AI-slide-generation brief (9 content pages + a 5-page
   Q&A-only appendix), each page with exact copy, numbers with citations, a visualization suggestion, a
   layout note, and a speaker note. **[MENTOR_BRIEF.md](./MENTOR_BRIEF.md)** — a non-technical soft-pitch
   covering all 8 mandatory slide topics, written for a mentor/judge who has never seen this project.
8. **[rounds/](./rounds)** — every past round's task list (`NEXT_STEPS_RX.md`) and verification report
   (`VERIFY_RX.md`), archived once done. The *current* round's plan (if one is in flight) lives at the
   repo root until it's finished, then moves here.
9. Background research (not this-project-scoped, but cited and reusable): [`../knowledge-base/topics/thai-dictionary-hackathon.md`](../knowledge-base/topics/thai-dictionary-hackathon.md).

## One-line orientation

- **Organizer / theme:** สำนักงานราชบัณฑิตยสภา (ORST) — build a Next-Generation Dictionary Platform
  prototype using AI/Open Data, 2 days/1 night, teams of 2-4.
- **Core bet (the hybrid):** don't pick one of AXIOM or katgpt-rs — use each for the part it's actually
  good at. `katgpt-rs`'s `Datrie`/`BpeTrainer` (real, working, standalone code) segments Thai text.
  AXIOM's `graph.rs` (vendored as a single dependency-free file, NOT the rest of AXIOM) takes
  dictionary-derived word-relationship triples and gives explainable, multi-hop "how are these words
  related" answers via Personalized PageRank + BFS — while deliberately never touching AXIOM's actual
  weaknesses (its English-only, noisy text-decomposition layer).
- **What's NOT in scope:** `katgpt-transformer` (confirmed non-functional for real checkpoints — random-init
  only), the rest of AXIOM beyond `graph.rs` (self-labeled "forensic archive"), and anything from the
  separate, unrelated [Green Mind AI 2026 / mango-a100](../neural-engines/mango-a100/) track.
- **Data source:** the CC0 62,107-word practice list (NECTEC LEXiTRON via PyThaiNLP) was the Day-0
  fallback; **the real event data arrived 2026-09-14** (`nextect.zip`, staged at `data/official/`,
  git-ignored) and is now ingested: RID 2554 (11,265 entries / 13,395 senses, ก–ซ only, as provided),
  official คำทับศัพท์ (2,256 pairs), 3 specialized-domain dictionaries (4,986 entries), and a 3-edition
  word-evolution timeline (2542→2554→2569, ก only). See `rounds/VERIFY_R10.md`.
- **Round 11-12 (2026-09-14/15):** the product reframed from "search a word" to a job-menu front door
  ("วันนี้อยากให้ภาษาไทยทำอะไรให้คุณดี") with a deterministic (no-LLM) intent router — type anything into
  one box, a rule table + a `thai2fit_wv` (PyThaiNLP, MIT) vector fallback guesses which of 5 modes you
  want, with a one-tap correction if it guesses wrong. Also shipped: an etymology-grounded naming
  assistant (a real, cited market gap — every existing Thai naming tool is numerology-based), a
  deterministic sound-symbolism score, register/rhyme search, and a PIE→English-cognate "bonus" layer
  cross-verified against Wiktionary (38/43 = 88.4% corroborated) before shipping. See `rounds/
  VERIFY_R11.md` / `VERIFY_R12.md`.
- **Feasibility POC → real build → win-readiness → real event data → job-menu + intent:** the POC
  (2026-09-04) proved both halves of the hybrid work on real Thai text; the product crate (`wacha/`) now
  passes **126 tests** and has been verified by actually running it (HTTP calls, CLI lookups, live
  spot-checks), not by trusting logs or a prior agent's summary — see `PROGRESS.md` for the full history
  of real bugs found this way, most recently a `{register}` tag-parsing bug (R12 WRITE) that had silently
  dropped 1,212 tags.
- **Persistent cross-session context:** Claude's memory system, `hackathon-dictionary-reimagined-2026`
  entry — kept in sync with this folder, but this folder is the canonical, detailed version.

## For a new agent picking this up

If you're starting fresh with no context: read files 1-3 above in order (10 minutes), then run the POC in
`poc/` to see the hybrid actually working. Don't re-derive any decision in `AGENT_HANDOFF.md`'s decisions
table without new evidence — each row states why, and re-litigating a settled call without new information
wastes hackathon time. If you make a new decision, change direction, or finish a task, **add a dated entry
to `PROGRESS.md`** before ending your session — that's what keeps the next agent (or the next context reset
of this same agent) in sync.
