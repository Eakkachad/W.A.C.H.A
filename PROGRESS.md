# Progress Log

Living log. Append a dated entry each working session — don't rewrite history, add to it. Keep the
**Status board** at the top current; it's the fast-scan summary for anyone (including a fresh agent)
picking this up. If you change a decision that `AGENT_HANDOFF.md` or `PLAN.md` documents, update those
files too and note it here — this log is the record of *that it changed*, those files are the record of
*what's currently true*.

---

## Status board

| Phase | State | Last touched |
|---|---|---|
| Research — open data, prior art, Thai NLP tooling, AXIOM/katgpt-rs feasibility | done | 2026-09-03 |
| Architecture decision — katgpt-rs vs. AXIOM vs. hybrid | done (hybrid chosen) | 2026-09-04 |
| Feasibility POC — Datrie segmenter + vendored graph engine on real Thai text | done, passed | 2026-09-04 |
| Research — application-form evidence (problem/users/outcomes/deployment path) | done | 2026-09-04 |
| Day 0 — resolve Thai word-list/data source (`AGENT_HANDOFF.md` §6) | **done** (CC0 `words_th.txt`, 62,107 words) | 2026-09-04 |
| Day 0 — fix OOV-fallback bug found in POC (TCC-boundary-aware, not raw codepoints) | **done** (`wacha/src/tcc.rs`) | 2026-09-04 |
| Day 0 — verify Typhoon 2 access | not started (direction 3, optional) | — |
| Day 1 — build (hybrid vertical slice) | **done** (`wacha/` crate, CLI, 29 tests) | 2026-09-04 |
| Verification pass — actually ran the built binary, not just the tests | done — found a real 42s cold-start bug (see log) | 2026-09-05 |
| Day 0/1 — fix or plan around the 42s Datrie build-time cost | not started — see `NEXT_STEPS.md` Task 2 | — |
| Data-quality audit of 20 seed relations (found: ครู/นักเรียน mislabeled as antonyms) | not started — see `NEXT_STEPS.md` Task 1 | 2026-09-05 |
| Web UI (optional) | not started — see `NEXT_STEPS.md` Task 3 | — |
| Typhoon 2 / direction 3 (optional) | not started — see `NEXT_STEPS.md` Task 4 | — |
| Day 2 — demo/submit | not started | — |

**A real product crate now exists** (`wacha/`) in addition to the `poc/` feasibility harness. The
core hybrid vertical slice — Datrie segmenter (TCC-aware OOV fix) + vendored graph engine with
explainable relationship queries over dictionary-derived triples — builds clean, passes 29 tests, and
runs end-to-end on the real 62,107-word list via a CLI. Directions 1+2 from `AGENT_HANDOFF.md` §7 are
implemented; direction 3 (Typhoon 2 AI-simplified definitions) remains the optional add-on, not started.

---

## Log

### 2026-09-03 — Research passes 1-2, initial AXIOM-vs-katgpt-rs decision

- Ran a deep-research workflow (108 agents, 25 sources, 3-vote verification) scoping the hackathon: open
  data availability for the Royal Institute Dictionary, global next-gen dictionary prior art
  (wiktextract/Lexonomy/Etytree/DBnary), Thai NLP open-source tooling (PyThaiNLP/AttaCut/BGE-M3), and
  AXIOM/katgpt-rs feasibility. Saved to
  [`../knowledge-base/topics/thai-dictionary-hackathon.md`](../knowledge-base/topics/thai-dictionary-hackathon.md).
  Headline finding: ORST publishes no general-purpose open API/dataset for the RID lexicon (only a narrow
  personal-names CSV) — this became the standing open risk in `AGENT_HANDOFF.md` §6.
- Compared AXIOM vs. `katgpt-rs` as the tech backbone. Read `katgpt-rs`'s actual maturity directly
  (`examples/kimi_k3_4b_hello_world.rs`): confirmed its transformer/inference path only runs on
  **random-init weights** — no real safetensors loader exists. Decided: `katgpt-tokenizer`'s
  `Datrie`/`BpeTrainer` for Thai segmentation (real, working code); an external Thai LLM (Typhoon 2) for
  any generative feature, never `katgpt-transformer`.
- Initially parked AXIOM entirely — its own repo self-labels it a "forensic archive" (per
  `../knowledge-base/local-inventory/neural-engines-workspace.md`), English-only, with a documented 52-pt
  find-vs-select accuracy gap, and no external literature support for a Thai-lexical-graph adaptation.
- Created `dict-hackathon/AGENT_HANDOFF.md` (v1.0) and `PLAN.md` to carry this forward.

### 2026-09-04 — Hybrid architecture decision + feasibility POC

- User pushed back on parking AXIOM entirely and asked about a hybrid. Read AXIOM's actual source
  (`neural-engines/AXIOM/crates/tle-axiom-gen/src/graph.rs`) directly: found it's a **666-line,
  dependency-free (`std` only)** triple-store + Personalized PageRank + BFS-subgraph module, populated via
  a direct `add_triple(subject, relation, object)` call — completely decoupled from AXIOM's actual
  weaknesses (its English-only `decompose_sentence`/`extract_query_entities` text-NLU layer). This
  unlocked a genuinely low-risk hybrid: katgpt-rs segments Thai text, vendored `graph.rs` (not the
  `tle-axiom-gen` crate, which would drag in VSA/decomposition deps) gives explainable relationship
  queries over dictionary-derived triples.
- Updated `AGENT_HANDOFF.md` to v2.0 and `PLAN.md` to reflect the hybrid as the chosen architecture
  (explainable word-relationship graph promoted from "stretch item" to primary differentiator, §7).
- Ran a second deep-research workflow (99 agents, 4 search angles, 3-vote verification) specifically for
  hackathon-application-form evidence (problem proof, affected users, outcome metrics, deployment path).
  Appended findings to
  [`../knowledge-base/topics/thai-dictionary-hackathon.md`](../knowledge-base/topics/thai-dictionary-hackathon.md#application-form-evidence-pass-2026-09-04-run-wf-f44a3e10-d6c).
  Headline: strong evidence Thai is an under-resourced NLP language and RID has no open-data path;
  **thin-to-absent evidence for who specifically is affected (form Q5) and for any Thailand
  government-to-community adoption precedent (form Q8)** — drafted honest answers acknowledging both gaps
  rather than inventing numbers or precedent.
- Built and ran a real feasibility POC (`poc/`, Rust): a Datrie-based greedy longest-match Thai segmenter
  using `katgpt-tokenizer` (path dependency, built clean standalone), and the vendored `graph.rs` seeded
  with 14 hand-built Thai word-relationship triples. **Result: both halves work on real Thai text.**
  Segmenter got 3/4 test sentences exactly right; the graph's `personalized_pagerank` + `bfs_subgraph`
  produced correct, explainable output (e.g. seeded on "แมว" (cat), correctly surfaced "สัตว์" (animal),
  "ปลา" (fish), "บ้าน" (house) as most related, with a readable relation path).
  **Real bug found, not speculative:** the out-of-vocabulary fallback (for words not in the tiny test word
  list) split unknown text into raw Unicode codepoints, shattering Thai Character Clusters (e.g. a
  consonant + tone mark got separated) — confirms in code exactly why PyThaiNLP's own segmenter enforces
  TCC boundaries. **Fix not yet applied** — logged as a Day-0 task on the status board above, not a
  blocker to the overall approach.
- Created this file and `README.md` so a new or resumed agent can orient without re-reading the whole
  conversation history.

**Next action:** resolve the Thai word-list/data-source question (`AGENT_HANDOFF.md` §6) — it blocks
scaling the POC to real data and is the single biggest remaining risk to the whole plan.

### 2026-09-04 (later) — Day-0 blockers cleared + hybrid product vertical slice built

Went from "planning complete, no product code" to a real, tested, demoable hybrid engine in one session.

**Data-source question resolved (`AGENT_HANDOFF.md` §6) — with verified evidence, not a re-cite.**
- Checked PyThaiNLP directly: not installed locally, but its corpus is on GitHub. Fetched the corpus
  license file (`pythainlp/corpus/corpus_license.md`) and confirmed:
  - **`words_th.txt` — 62,107 Thai words, CC0-1.0 (public domain, no attribution required).** Derived
    from NECTEC LEXiTRON (which §6 flagged as an unchecked lead — now checked and it's the provenance of
    this list). Downloaded to `data/words_th.txt` (1.5 MB).
  - **`tnc_freq.txt` — Thai National Corpus word frequencies, CC0-1.0.** Downloaded to `data/tnc_freq.txt`
    (106,122 lines) for result ranking / PPR seeding.
  - **Thai WordNet (`wordnet_th.db`)** exists under a permissive NICT license (free use/copy/modify/
    distribute with copyright notice) — a real synonym/hypernym source available later if we want to
    scale triple coverage beyond the curated seed. Not wired in yet (it's a SQLite DB; the seed entries
    cover the demo).
  This is a legally-clean, 62k-word substitute — exactly the Day-0 exit criterion. If ORST hands out an
  official RID dataset at the event it supersedes this (the `Entry`/relation model is the integration
  point).

**OOV-fallback bug fixed (TCC-boundary-aware).** New module `wacha/src/tcc.rs` implements
rule-based Thai Character Cluster grouping (leading vowels bind to the following consonant; tone marks /
above-below vowels / SARA AM bind to the preceding base). The segmenter's OOV path now emits whole
clusters instead of raw codepoints. Regression test asserts the exact POC bug case ("เด็กน้อย" →
previously shattered into เ|ด|็|ก|น|้|อ|ย) no longer orphans any tone mark or leading vowel.

**Product crate built: `wacha/`** (library + CLI, separate from `poc/`). Modules:
- `tcc.rs` — TCC clustering (the bug fix).
- `segmenter.rs` — `Datrie` longest-match with TCC-aware OOV fallback; tags tokens in-vocab vs OOV.
- `dictionary.rs` — `Entry` model (word/pos/definition/relations), word-list + frequency loading, and a
  curated seed of 20 real Thai entries with real synonym/antonym/is-a/see-also/category relations.
- `graph.rs` — vendored verbatim from AXIOM (single file only, per the guardrails).
- `relations.rs` — extracts triples from entries into the graph; `related(word, k)` returns ranked
  related words each with a human-readable **explanation path** (direct edge or 2-hop bridge). Symmetric
  relations made bidirectional; disconnected floor-score nodes filtered out; explanation edges deduped.
- `lib.rs` — `Engine` facade + `Lookup` type stitching the full journey (segment → define → related).
- `bin/cli.rs` — REPL + `lookup`/`segment`/`stats` subcommands; `--data DIR` loads the real word list.

**Verified end-to-end.** `wacha` builds with no warnings, 29 tests pass. CLI on the real 62,107-word
list: segmentation correct, the OOV bug case stays intact, and `lookup แมว` / `lookup ครู` return ranked
related words each with a relation-path explanation (e.g. `ครู --มีความหมายเหมือนกับ--> อาจารย์`,
`ครู --ตรงข้ามกับ--> นักเรียน`). This is directions 1+2 from `AGENT_HANDOFF.md` §7 working together.

**Latent dependency bug found and fixed (not speculative).** `katgpt-tokenizer`'s `Datrie::insert`
asserted `new_child < check.len()` after `resolve_collision` with the message "grow should have happened
in resolve" — but `find_new_base` can legitimately return a base whose slot is past the array end
(out-of-range slots are treated as free), and nothing grew the arrays. The 23-word POC set never tripped
it; the seed word set did. Fix: replaced the assert with `grow_to(new_child + 1)`, mirroring the normal
insert path. All 16 `katgpt-tokenizer` tests still pass. This is a shared-crate change outside
`dict-hackathon/` — flagged here because it affects other `katgpt-rs` consumers (it only makes a
previously-panicking build path succeed, so it's strictly a fix, not a behavior change for working cases).

**What's left:** direction 3 (Typhoon 2 AI-simplified definitions) is still optional/unstarted; a
visual (web) UI on top of the CLI would strengthen the demo; scaling triple coverage via Thai WordNet is
a post-slice enhancement. None of these block the core submission.

**Next action:** optionally build a minimal web UI over the `Engine` API for the live demo, or wire
Typhoon 2 for direction 3 — but the core hybrid (directions 1+2) is now a working, tested vertical slice.

### 2026-09-05 — Verification pass: found a real 42-second cold-start performance bug

Re-verified the previous session's claims by actually running the built binary (not just trusting the
log) — per this project's own "trust but verify" convention. `cargo test` confirmed 29/29 passing as
claimed. The CLI's live output was also as good as claimed (`lookup ครู` correctly returns
`ครู --มีความหมายเหมือนกับ--> อาจารย์` etc.) — **but** the earlier log's "runs end-to-end on the real
62,107-word list" did not mention how long that takes, and it turned out to matter:

- **Root cause (isolated with a throwaway `examples/repro.rs`, not yet deleted — safe to remove or keep
  as a benchmark):** `Engine::build` → `Segmenter::from_words` → `DatrieVocab::build` takes **42.4
  seconds** on the real 62,106-word list. `cargo test` never caught this because every unit test uses a
  17-23-word toy vocabulary.
- **Why:** Aoe's double-array trie collision-resolution (`resolve_collision`/`find_new_base` in
  `katgpt-tokenizer/src/datrie.rs`) assumes a reasonably diverse branching factor at each trie level. Thai
  script's UTF-8 encoding is extremely narrow at the byte level (the whole Thai block shares the same
  lead byte, `0xE0`, and only two second bytes, `0xB8`/`0xB9`) — so at real-dictionary scale, tens of
  thousands of words collide at the same few shallow trie nodes, and the collision-resolution cascade
  becomes severely expensive. The 17-23-word toy vocabularies used in every existing unit test are far too
  small to trigger this — **this is a real scale-dependent bug that passing tests actively hid.**
  Once built, the trie itself is fast (confirmed: 3.5µs for a 1-word segment, 23µs for a 5-word sentence)
  — this is purely a one-time **build-time** cost, not a per-query cost.
- **Is it a blocker?** Not for a persistent-process demo (REPL, or a web server that builds the engine
  once at startup and serves many requests) — pay the 42s once before the judges see it, never restart
  mid-demo. It **would** be a serious problem for any one-shot-CLI-invocation-per-word usage pattern (e.g.
  the way `cargo run ... lookup X` was being tested — each invocation rebuilds from scratch) or a
  serverless/lambda-per-request architecture. Added as a guardrail in `AGENT_HANDOFF.md`.
- Not yet investigated: whether serializing the built trie to disk (build once, load fast on every
  subsequent run) or reducing to a subset word list would be a better fix than living with 42s — open
  item, not yet decided.

**Status board and `AGENT_HANDOFF.md` updated to reflect this.** Overall assessment: the hybrid still
works and the output quality is genuinely good — this is a real, fixable engineering fact to plan the demo
architecture around, not a reason to doubt the approach.

**Handoff prepared:** wrote [`NEXT_STEPS.md`](./NEXT_STEPS.md) — a prioritized, self-contained task list
(seed-data audit, the 42s build fix, optional web UI, optional Typhoon 2) for another agent to execute.
Linked from `README.md`'s reading order. The user will hand this to a separate agent/session and will tell
this session when to check in and re-verify — no automated polling set up. **When you (the executing
agent) pick this up: your task list is `NEXT_STEPS.md`, not this paragraph — go read it.**

### 2026-09-07 — Project named วาจา (WACHA); crate renamed

The user proposed naming the project วาจา (Wacha), a real Thai word for "speech/word/utterance" (as in
สัตย์วาจา, "word of honor"), with an English backronym. Chosen: **WACHA = Word Architecture,
Cluster-aware Hybrid Analysis** — every letter mapped to a real, verified piece of the system rather than
a generic marketing phrase (W=segmentation, A=the Datrie structure itself, C=the TCC OOV fix, H=the
katgpt-rs+AXIOM hybrid, A=the PageRank/BFS explainable-ranking layer).

Scope decision (user confirmed): **branding-only rename**, not a full directory rename — lower risk, no
relative-path breakage. Did:
- Renamed `dict-engine/` → `wacha/`; `Cargo.toml` package/lib/bin all renamed `dict-engine`/`dict_engine`
  → `wacha`; fixed internal source references (`src/lib.rs`, `src/bin/cli.rs`, `examples/repro.rs`);
  updated `wacha/README.md` with the วาจา/WACHA name and a table mapping each backronym letter to its
  real code location.
- Blanket-replaced `dict-engine` → `wacha` across this project's own docs (`README.md`,
  `AGENT_HANDOFF.md`, `PLAN.md`, `NEXT_STEPS.md`, this file) plus added วาจา/WACHA branding to each file's
  title/intro (not just the mechanical path rename).
- Updated the root workspace `README.md` and the `hackathon-dictionary-reimagined-2026` memory entry to
  carry the name forward.
- **Verified, not just asserted:** re-ran `cargo test` in `wacha/` (29/29 still pass) and `cargo build` in
  `poc/` (unaffected, different crate) after the rename, before considering this done.
- `poc/`'s own crate name (`dict-hackathon-poc`) was deliberately left unchanged — it's explicitly
  documented as the superseded historical feasibility harness, not the branded product.

**No functional change** — this was a naming/branding pass only. Nothing in `NEXT_STEPS.md`'s task list
changed as a result; the crate path in its instructions now reads `wacha/` instead of `dict-engine/`.
