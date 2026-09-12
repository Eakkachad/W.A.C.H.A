# วาจา (WACHA) — Dictionary Reimagined Hackathon (เปิดคลังคำ พลิกคลังคิด)

**วาจา (WACHA — Word Architecture, Cluster-aware Hybrid Analysis):** a hybrid Thai dictionary prototype —
katgpt-rs's tokenizer reads the dictionary, AXIOM's knowledge-graph engine explains word relationships —
for the ORST "เปิดคลังคำ พลิกคลังคิด" hackathon. วาจา is a real Thai word for "speech / word / utterance"
(as in สัตย์วาจา, "word of honor"); every letter of the backronym maps to a real, verified piece of the
system (see `wacha/README.md` for the full breakdown), not a marketing label.

Status: **win-readiness round (Round 3) in progress — core product is demo-complete.** `wacha/` (Datrie
segmenter, vendored graph engine, WordNet-expanded relationships with provenance/confidence tagging,
offline learner content, web UI with graph viz) passes 50 tests and runs end-to-end on the real 62k-word
CC0 list + ~29k WordNet-derived relations. Pitch materials (`PITCH.md`) and a rehearsed/adversarially-tested
demo are done; see `PROGRESS.md` for the full dated history. Event dates not yet confirmed. This folder
exists so any agent or session (including a fresh one with no memory of how this plan was made) can pick
the work up cold — that is the explicit purpose of this file and the others below.

---

## Read these first, in order

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
8. Background research (not this-project-scoped, but cited and reusable): [`../knowledge-base/topics/thai-dictionary-hackathon.md`](../knowledge-base/topics/thai-dictionary-hackathon.md).

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
- **Data source:** resolved (2026-09-04) — a real CC0 62,107-word Thai list (NECTEC LEXiTRON via
  PyThaiNLP), not a hypothetical risk anymore. See `PROGRESS.md`.
- **Remaining open items (2026-09-12, Round 3):** Tasks 7–10 are done (confidence tagging + measured
  84.2% WordNet precision, `PITCH.md`, rehearsed/adversarially-tested demo, and `wacha/API.md` — the
  `/api/lookup` contract + data-license "open data" story). Only Task 11.2 remains (optional — real
  Typhoon 2 access to upgrade learner content from `human_seed` provenance). See `NEXT_STEPS.md`'s
  "Round 3" and `PROGRESS.md` for the full detail.
- **Feasibility POC → real build → win-readiness:** the POC (2026-09-04) proved both halves of the hybrid
  work on real Thai text; a real product crate (`wacha/`) now exists with 50 passing tests and
  verified-by-actually-running output at every stage. See `PROGRESS.md` for the full history, including
  every real bug found by actually executing the code rather than trusting tests/logs or a prior agent's
  summary (a codepoint-vs-Thai-Character-Cluster OOV bug, a 42s build-time bug now fixed via a trie cache,
  and a real WordNet data-quality/confidence-tagging gap now closed with a measured 84.2% precision).
- **Persistent cross-session context:** Claude's memory system, `hackathon-dictionary-reimagined-2026`
  entry — kept in sync with this folder, but this folder is the canonical, detailed version.

## For a new agent picking this up

If you're starting fresh with no context: read files 1-3 above in order (10 minutes), then run the POC in
`poc/` to see the hybrid actually working. Don't re-derive any decision in `AGENT_HANDOFF.md`'s decisions
table without new evidence — each row states why, and re-litigating a settled call without new information
wastes hackathon time. If you make a new decision, change direction, or finish a task, **add a dated entry
to `PROGRESS.md`** before ending your session — that's what keeps the next agent (or the next context reset
of this same agent) in sync.
