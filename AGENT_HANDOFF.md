# AGENT HANDOFF: วาจา (WACHA) — Dictionary Reimagined Hackathon (เปิดคลังคำ พลิกคลังคิด)

**Document version:** 2.5 (positioning locked in — modelless/deterministic/explainable, §1; finalized completion roadmap P1-P5, §7.5; web UI built)
**Date of creation:** 2026-09-03 · **Last updated:** 2026-09-12
**Project workspace:** [`dict-hackathon/`](file:///Users/wuttichaimaneesangsakorn/Eak_ject/Reserch/dict-hackathon)
**Organizer:** สำนักงานราชบัณฑิตยสภา (Office of the Royal Society of Thailand, ORST)
**Format:** Hackathon, 2 days / 1 night, 10 teams of 2-4
**Event dates:** not yet known — fill in once confirmed; treat every day-numbered item in
[PLAN.md](./PLAN.md) as relative to event day 1, not a calendar date.

**New to this project?** Start at [README.md](./README.md) instead — it gives the reading order and a
one-line orientation. Check [PROGRESS.md](./PROGRESS.md) for what's actually been done vs. what this file
and `PLAN.md` merely planned; update it whenever you finish something or change a decision here.

---

## 1. Mission

Build a Next-Generation Dictionary Platform prototype that modernizes how Thai-language dictionary data
is searched, accessed, and reused (education / research / language innovation), using AI and Open Data.

### What วาจา (WACHA) ultimately *is* — the positioning (locked in 2026-09-12)

**วาจา is a modelless, deterministic dictionary-intelligence engine — not an AI chatbot wearing a
dictionary costume.** Its two core layers (segmentation, explainable relationships) run **zero neural
models at query time**: no GPU, no API key, no inference cost, no hallucination risk, the same answer
every time, and every answer comes with a literal, inspectable reasoning path (a BFS edge list), not a
black-box generation. This traces directly back to `katgpt-rs`'s own self-description as a "modelless
inference primitives" repo (see `katgpt-tokenizer`'s own `Cargo.toml`: *"standalone **modelless**
tokenizer crate"*) — the user's original intent for this whole track was efficiency and precision under
constrained resources, and the build ended up matching that intent more literally than initially planned:
the segmenter is a pure double-array trie (µs-scale lookups after a one-time cached build), and the
relationship layer is a pure graph algorithm (Personalized PageRank + BFS) over hand-verified facts — both
100% deterministic, auditable, and correct-by-construction rather than statistically probable.

**Where AI actually fits, and where it deliberately doesn't:** the only place any LLM (Typhoon 2, §7
direction 3) touches this system is an explicitly bounded, **offline, precomputed content-enrichment
step** (simplified definitions / example sentences), generated once and cached — never a live inference
dependency, and never allowed to touch the segmentation or relationship layers that must stay correct. If
that enrichment text is imperfect, it cannot corrupt the verified dictionary data or the explainable graph
underneath it. This is the honest, differentiated pitch: most competing hackathon entries will likely wrap
an LLM API around dictionary text (probabilistic, costly, hallucination-prone, needs internet/a GPU); วาจา
inverts that — determinism and explainability are the default, AI is an optional, clearly-labeled garnish.

The chosen differentiator is a **hybrid of two of the user's own research assets**, each used only for
the part it's actually good at:
- **`katgpt-rs`'s tokenizer primitives** (`Datrie`, vendored into `wacha/src/datrie.rs` — see §3) —
  segments and normalizes Thai text, literally making the dictionary's own word list the engine that
  reads Thai.
- **AXIOM's `KnowledgeGraph`** (vendored standalone from `crates/tle-axiom-gen/src/graph.rs`, NOT the
  rest of AXIOM) — a triple-store + Personalized PageRank + BFS-subgraph engine, populated directly with
  structured word-relationship triples (`add_triple(subject, relation, object)`), giving an explainable
  "how/why are these words related" multi-hop query feature.

This deliberately avoids AXIOM's actual weaknesses: `graph.rs` has **zero dependencies beyond
`std::collections`** and no English-specific string handling, and by feeding it triples directly we never
touch AXIOM's fragile, English-only `decompose_sentence()`/`extract_query_entities()` layer — the part
that's actually broken for this use case (see §3). Both pieces continue the user's own research threads
(same repo as the separate, unrelated
[Green Mind AI 2026 track](../neural-engines/mango-a100/AGENT_HANDOFF.md)) rather than treating the
hackathon as a pure off-the-shelf integration exercise.

## 2. Current status

**Core hybrid vertical slice is built and tested** (as of 2026-09-04 — see `PROGRESS.md` for the dated
log). The `wacha/` crate (library + CLI, separate from the `poc/` feasibility harness) implements
directions 1+2 from §7: a `Datrie` segmenter with a TCC-aware OOV fallback (the POC bug is fixed) plus the
vendored `graph.rs` giving explainable relationship queries over dictionary-derived triples. It builds
clean, passes 38 tests (2026-09-12), and runs end-to-end on the real 62,107-word CC0 list. A web UI
(`wacha-web`) is also built (2026-09-12). Direction 3 (Typhoon 2, now scoped as offline-precomputed only —
see §1's positioning note) remains the one open item. The planning context below is preserved for
provenance.

Two research passes preceded the build:

- **Deep-research workflow** (108 agents, 25 sources, 3-vote adversarial verification) on open data
  availability, global dictionary prior art, Thai NLP tooling, and AXIOM/katgpt-rs feasibility. Full
  findings: [`knowledge-base/topics/thai-dictionary-hackathon.md`](../knowledge-base/topics/thai-dictionary-hackathon.md).
- **Local audit of `katgpt-rs`'s actual maturity** for this use case (2026-09-03) — see §3 below for what
  it found.
- **Local audit of AXIOM's `graph.rs`** (2026-09-04) — confirmed it's a standalone, dependency-free
  (`std` only) triple-store/PageRank/BFS module with a direct `add_triple` API, decoupled from AXIOM's
  broken text-decomposition layer — the finding that unlocked the hybrid architecture in §3.

## 3. Key decisions already made, and why

| Decision | Why | Don't relitigate this without new evidence |
|---|---|---|
| Segmentation/tokenizer layer = `Datrie` (double-array trie), fed by a compiled Thai word list | Real, standalone, working code today — no LLM dependency. `Datrie` is a generic byte-key trie (Aoe 1989 double-array structure) fast enough to build a longest-match dictionary segmenter directly from real word data — the dictionary becomes the tokenizer. This is the genuine, demoable "why us" story. | Re-derive only if the vendored code turns out to not build standalone (it does — verified 2026-09-12). |
| **`Datrie` is vendored into `wacha/src/datrie.rs`, not a live path dependency on `katgpt-rs`** — same treatment as AXIOM's `graph.rs` | Decided 2026-09-12: `katgpt-rs` is not this project's repository, and two real fixes made to its `datrie.rs` (the 2026-09-04 `grow_to` panic fix, and the 2026-09-12 serde derives for the trie cache) existed only as **uncommitted working-tree changes** in that repo — found during verification, a real fragility risk (any `git stash`/`checkout`/`pull` there could silently erase them and reintroduce both bugs with no obvious error). Vendoring removes the dependency entirely; `wacha` now builds with nothing outside this repo. Confirmed: `cargo tree` shows no `katgpt-tokenizer`; the old cache file (built by the pre-vendor version) still loads correctly (postcard's binary format didn't change, only the Rust module path did); 38/38 tests pass, zero warnings. | Don't reintroduce the path dependency. If a future fix is needed to the trie logic, make it directly in `wacha/src/datrie.rs` — there is no upstream to sync with or protect. |
| Generation/reasoning layer (AI-simplified definitions, examples, Q&A) = call an existing open Thai LLM (**Typhoon 2**, SCB 10X, open weights 1B–70B on Llama3/Qwen2; SEA-LION as a fallback), **not** `katgpt-transformer` | Confirmed this session by reading `katgpt-rs/examples/kimi_k3_4b_hello_world.rs` directly: it only runs **random-init weights** — the file's own doc comment says output is "gibberish, not real text." No safetensors loader exists anywhere in the repo for a real checkpoint, except one test gated behind an external Gemma2-2B file (`WEAVER_CHECKPOINT_PATH`) that isn't even present in this workspace. Building a working loader + correct forward pass for a real model from scratch would consume the entire 2-day budget on infrastructure instead of product. | Don't attempt real-checkpoint inference in `katgpt-transformer`/`katgpt-forward` during the hackathon itself — see Guardrails. |
| `katgpt-speculative` / `katgpt-quant` / `katgpt-hla` are **out of scope** for this track | They exist to speed up LLM generation that doesn't work yet for this use case (see row above) — nothing to speed up. These are the Green Mind track's assets, not this one's. | — |
| **AXIOM's `graph.rs` only** (triple-store + Personalized PageRank + BFS-subgraph) is vendored as the explainable-relationship layer — **the rest of AXIOM (VSA, `tle-axiom-gen`'s text decomposition/QA engine) is out of scope** | `graph.rs` (666 lines) has zero dependencies beyond `std::collections` and no English-specific text processing — confirmed by direct source read (2026-09-04). It's populated via `add_triple(subject, relation, object)`, built from our own dictionary-derived triples, never through AXIOM's own `decompose_sentence()`/`extract_query_entities()`. Those latter functions are where AXIOM's actual known weaknesses live: self-labeled **"forensic archive"** (per `knowledge-base/local-inventory/neural-engines-workspace.md`), English-only today (Thai "planned"), ~30%-noisy decomposition, 52-point find-vs-select gap. None of that is exercised by this hybrid. | Don't pull in `tle-axiom-gen` as a crate dependency (drags in `tle-vsa`/`tle-vsa-lm`/`tle-afc`/`tle-ghrr`/`tle-neural-core`) — copy just `graph.rs` into this project. Don't call any of AXIOM's decompose/QA functions on Thai text — that reintroduces every known weakness this design avoids. |
| Open dataset for the Royal Institute Dictionary lexicon: **assume none exists, verify a substitute before building the pipeline around it** | ORST publishes no general-purpose API/bulk dataset — the only confirmed official open dataset is a narrow personal-names CSV (`gdcatalog.go.th/dataset/gdpublish-orst-0301`). PyThaiNLP's claimed "lexicon-thai" open dataset did not survive verification in the research pass — check it directly before relying on it (§6). | This is the single biggest risk to the whole plan — resolve first, before writing segmenter code against assumed data. |

## 4. What's safe to prepare before the event vs. what must happen live

Adapting the "what transfers" framing from the Green Mind track's planning process: not everything is
equally safe to invest in ahead of a 2-day event where the organizers may hand out their own seed data.

**Safe to build/verify before the event (won't be wasted):**
- ~~Confirm `katgpt-tokenizer` builds and runs standalone~~ — **done, then superseded**: `Datrie` is now
  vendored directly into `wacha/src/datrie.rs` (§3), so this item no longer applies as originally framed.
- Resolve the data-source question (§6) — this is pure research/legal-check work, not data-shaped work,
  so it doesn't get invalidated by whatever the event provides.
- Get Typhoon 2 (or SEA-LION) generation access working end-to-end on a throwaway prompt, so it's a known
  quantity, not a live unknown, when the event starts.
- Rough UI/product-concept sketch with the team — but hold it loosely; a real dataset drop or mentor
  feedback on Day 0/1 may reshape it.

**Must happen live at the event:**
- Final data integration — if ORST or the organizers hand out an official dataset at the event, that
  supersedes whatever was scraped/compiled beforehand; don't over-invest in a pre-hackathon scrape's exact
  schema.
- Actual UI/UX and feature-scope decisions — these depend on mentor guidance and teammate input, not
  something to lock in solo ahead of time.

## 5. Immediate next actions (before Day 1)

1. **Resolve the data question** (§6) — this blocks everything else. Don't start segmenter/product work
   against assumed data.
2. **Prototype the `Datrie` segmenter** against a small hand-typed word list (a few hundred words) to prove
   the algorithm and API before scaling to a real word list.
3. **Vendor `graph.rs`** from `neural-engines/AXIOM/crates/tle-axiom-gen/src/graph.rs` into this project
   and write a small triple-extraction pass over dictionary entries (start with explicit relations RID
   entries already carry — synonyms, cross-references — before attempting anything inferred).
4. **Verify Typhoon 2 access** — self-hosted small variant vs. a hosted API; test one real Thai
   prompt end-to-end.
5. **Sketch the product concept** with the team: which of the directions in §7 to commit to.

## 6. Open data-source question — RESOLVED (2026-09-04)

**Resolved:** the segmenter word list is **PyThaiNLP `words_th.txt` — 62,107 Thai words, CC0-1.0 (public
domain, no attribution required)**, downloaded to `data/words_th.txt`. License verified directly from
`pythainlp/corpus/corpus_license.md` (not a re-cite of the research pass). Provenance: NECTEC LEXiTRON
(the §6 lead that was previously unchecked — now confirmed). Also fetched `tnc_freq.txt` (Thai National
Corpus frequencies, CC0-1.0) to `data/` for ranking. **Thai WordNet** (`wordnet_th.db`) is available under
a permissive NICT license for richer synonym/hypernym triples later, but is a SQLite DB and not yet wired
in — the curated seed entries in `wacha/src/dictionary.rs` cover the demo.

If ORST hands out an official RID dataset at the event, it supersedes this substitute — the `Entry` +
`Relation` model in `wacha/src/dictionary.rs` is the integration point.

Original notes (kept for provenance — each was checked directly, not re-cited):
- `pythainlp` corpus data — checked: `words_th.txt` (CC0) is the answer; the refuted "lexicon-thai"
  dataset claim was a red herring, the plain bundled word list is what's real and usable.
- Open Multilingual Wordnet / Thai WordNet — available under NICT license (permissive); parked as a
  later enhancement, not needed for the slice.
- Scraping `dictionary.orst.go.th` — not needed; the CC0 list makes scraping unnecessary and avoids the
  ToS question entirely.
- NECTEC's LEXiTRON — confirmed as the upstream provenance of the CC0 `words_th.txt` list.

## 7. Candidate product directions

1. **Dictionary-driven semantic search** (backbone, always in scope) — Datrie/BPE segmentation
   (katgpt-rs) → PyThaiNLP or embedding search (BGE-M3 or fastText+ConceptNet, per the research pass) for
   "search by meaning," not just exact word match.
2. **Explainable word-relationship graph** (the hybrid differentiator, §3) — AXIOM's vendored `graph.rs`
   over triples extracted from dictionary entries (synonyms, cross-references, hypernyms if the data
   supports it), surfaced as a multi-hop "how are these words related" query with a visualized BFS path
   (inspired by Etytree/DBnary from the research pass — graph, not tree, since real relationships have
   cycles) and PageRank-ranked "related words." This is now the primary differentiator, not a fallback.
3. **AI-simplified definitions** (optional add-on, **now scoped as offline-precomputed only** — see §1's
   positioning note) — take a dense/formal RID-style definition and use Typhoon 2 to generate a
   plain-language definition or example sentence for learners, generated once per seed word and cached as
   a static asset, never called live during a demo.

Recommended scope for 2 days: (1) + (2) as the core submission. Add (3) only if Day 1 finishes ahead of
`PLAN.md`'s exit criteria — it's independent of the other two and safe to cut without unraveling anything.

## 7.5. Finalized completion roadmap (2026-09-12) — mapped against the organizer's own brief

The organizer's actual brief (received 2026-09-12, Thai) states the goals as: turn the dictionary from a
"word bank" into a "data bank" (คลังคำศัพท์ → คลังข้อมูล); make it more than a lookup tool, encyclopedia-like
for specialized knowledge (สารานุกรม → หาความรู้เฉพาะด้าน); elevate search for the digital age; build a
prototype with a UI/functions that help people learn how to use Thai (ช่วยเรียนรู้ภาษาไทยใช้ยังไง); build
networks; and promote AI/Open Data. Scorecard against what's built:

| Brief objective | Status |
|---|---|
| Word bank → data bank | ✅ done — `Entry`/`Relation`/`graph.rs` is structured data, not flat text |
| More than lookup (explainable, not just definitions) | ✅ done — explainable multi-hop related-words with a visible reasoning path |
| Elevate search for the digital age | ✅ done — CLI + `wacha-web` JSON API, µs-scale queries after warmup |
| Encyclopedia-like specialized knowledge | ⚠️ partial — only 20 hand-curated words have rich relations; **P2 below is the fix** |
| Help learn how to use Thai | ⚠️ partial — related-words shows word relationships, but no learner-facing simplified text yet; **P1 below is the fix** |
| Build networks | ⚠️ addressable in the pitch, not the code — the open CC0 data + open JSON API *is* the "build on this" story; make it explicit in Day 2 pitch materials, don't leave it implicit |
| Promote AI/Open Data | ✅ done — CC0-cited data throughout; "AI" here is the graph-reasoning layer (a real, classical AI technique, just not deep learning) plus the optional Typhoon 2 layer |

**Priority-ordered remaining work** (P1 highest — do these before anything else optional):

- **P1 — Offline-precomputed learner content (closes the "help learn Thai" gap, ~lowest risk).** Generate
  a plain-language definition + one example sentence for each of the 20 seed words via Typhoon 2, **once,
  offline**, and hardcode/cache the results as static data (a JSON/Rust data file, not a live call). This
  is direction 3 from §7, deliberately de-scoped from "live API integration" to "static enrichment asset"
  — it satisfies the brief's objective and the "use AI" requirement without introducing any live-demo
  failure mode or breaking the modelless-at-runtime positioning (§1). Wire it into both the CLI and
  `wacha-web` as a "คำอธิบายง่าย" (simple explanation) field alongside the existing formal definition.
- **P2 — Expand relation coverage via Thai WordNet (closes the "encyclopedia/specialized knowledge" and
  "data bank" gaps at scale).** `wordnet_th.db` was already found and license-checked (permissive NICT
  license) during the 2026-09-04 data-source resolution but never wired in (see `PROGRESS.md`). Extracting
  synonym/hypernym/hyponym relations from it and feeding them into `graph.rs` as triples would scale the
  relationship graph from 20 words to a real fraction of the 62k-word list — the single highest-leverage
  remaining improvement for both "data bank" depth and demo impressiveness (a judge searching an
  unpredictable word should still get an explainable result, not just the 20 pre-picked ones).
- **P3 — Visualize the relationship graph, not just list it.** `wacha-web`'s related-words panel is
  currently a ranked text list with path strings. A small interactive graph rendering (nodes + edges,
  click to re-center) would communicate "explainable AI reasoning" far more viscerally to judges than text
  — directly serves the brief's "เห็นภาพ" (visualize) language. Scope as a client-side-only enhancement
  (the `/api/lookup` JSON already has everything needed); no backend change required.
- **P4 — Pitch materials leading with the positioning in §1.** The differentiation story is "deterministic,
  explainable, and modelless at runtime — efficient enough to run on a judge's laptop with no GPU and no
  API key, unlike a typical LLM-wrapper entry" — make this the opening 30 seconds of the pitch, not a
  footnote. Explicitly name the open CC0 data + JSON API as the "build a network on top of this" answer to
  that brief objective.
- **P5 — Final robustness pass.** Re-run the full verification loop (tests, real CLI/curl calls, not just
  code review) once P1-P3 land, the same way every prior change in this project has been verified before
  being called done.

None of P1-P5 touches or risks the already-verified core (segmentation + relationship graph, directions
1+2) — each is additive and independently droppable if time runs out, in the priority order above.

## 8. Guardrails (don't repeat these mistakes)

- **Don't attempt to make `katgpt-transformer` produce real generated text during the hackathon.**
  Confirmed non-functional for real checkpoints this session (§3). Use Typhoon 2/SEA-LION for anything
  generative.
- **Don't assume a bulk RID word list will be handed to you.** Verify a real source before writing
  pipeline code around it (§6).
- **Don't pull in `tle-axiom-gen` or any other AXIOM crate beyond the vendored `graph.rs`.** The rest of
  AXIOM is a self-declared forensic archive with no external literature support for this domain, and
  pulling in the full crate drags in its VSA/decomposition dependencies unnecessarily (§3).
- **Don't call AXIOM's `decompose_sentence`/`extract_query_entities`/any text-NLU function on Thai text.**
  Those are the English-only, noisy parts — the hybrid design's entire risk reduction depends on never
  touching them. Build triples ourselves from structured dictionary data instead.
- **Don't cite Thai-NLP benchmark numbers as unqualified facts to judges.** AttaCut's "6x/91% F1" and
  BGE-M3's "90.50 R@1" both passed only 2-1 adversarial verification (one verifier voice unconvinced,
  likely on self-reported-benchmark grounds) — solid but not bulletproof; hedge appropriately in a pitch.
- **Don't confuse this track with Green Mind AI 2026.** Different event, different hardware assumptions,
  different codebase parts of `katgpt-rs` are relevant (tokenizer here, speculative-decoding/quant there).
- **The `Datrie` build from the real 62k-word list is a ~43s one-time cost — now cached to disk, so this
  is no longer a live-demo hazard (but know why the cache exists).**
  Confirmed 2026-09-05 and re-measured 2026-09-12: building the `Datrie` from the real 62,106-word list
  takes **~43 seconds** (Aoe's double-array collision resolution degrades badly on Thai's narrow UTF-8
  byte range — every word shares the same lead byte, so tens of thousands of entries collide at the same
  few shallow trie nodes). **Fixed 2026-09-12:** the CLI now serializes the built segmenter to
  `DIR/words_th.datrie.cache` and reloads it in **~10ms** on subsequent runs (~4,400× faster; verified
  with real before/after numbers in `PROGRESS.md` 2026-09-12). The cache is mtime-keyed on the word list.
  So a one-shot CLI call is now ~0.02s, not 43s. The old "never restart the process mid-demo" advice is
  still good hygiene but is no longer load-bearing. If you build a *new* entry point (e.g. a web server),
  still prefer building/loading the engine once at startup, not per-request — and delete the
  `.datrie.cache` if you ever suspect it's stale (editing the word list auto-invalidates it).

## 9. Handoff protocol (how sessions/agents stay in sync)

This project is designed to be picked up cold by any agent or session, including ones with no memory of
how this plan was made. The protocol:

- **`README.md`** is the entry point — orientation + reading order. Point any new agent there first.
- **This file (`AGENT_HANDOFF.md`)** is the stable record of decisions and *why* — the "don't relitigate
  this without new evidence" column exists specifically so a fresh agent doesn't waste time re-deriving
  settled calls. Update it only when a decision actually changes (as happened 2026-09-04, v1.0 → v2.0,
  when AXIOM went from "parked" to "hybrid via `graph.rs` only").
- **`PLAN.md`** is the schedule — update it if reality diverges from the plan (a phase runs long, a
  direction gets cut), don't let it silently go stale.
- **`PROGRESS.md`** is the living log — append a dated entry every working session, whether that's a
  research pass, a decision change, code written, or a POC run. This is the file to read first if you only
  have time for one, since it says what's *actually* true vs. what the other files *planned*.
- **`poc/`** is working code proving feasibility, not the product — don't confuse "the POC passed" with
  "the feature is built."

When you (any agent) finish a session on this project: update `PROGRESS.md`'s status board and add a dated
log entry, and update `AGENT_HANDOFF.md`/`PLAN.md` if a decision or the schedule actually changed. That's
what makes switching between sessions/agents seamless instead of requiring a full context re-read each
time.

## 10. Where things live

- This project: `dict-hackathon/` (this folder).
- **Product crate (the built vertical slice): `dict-hackathon/wacha/`** — library (`tcc`,
  `segmenter`, `dictionary`, `graph`, `relations`, `Engine` facade) + CLI (`bin/cli.rs`). Run
  `cargo run -- --data ../data lookup แมว` from inside it, or `cargo test`.
- **Data (CC0): `dict-hackathon/data/`** — `words_th.txt` (62,107 words), `tnc_freq.txt` (frequencies).
- Feasibility harness (not the product): `dict-hackathon/poc/`.
- Vendored source (no live external dependency): `wacha/src/datrie.rs` (originally
  `katgpt-rs/crates/katgpt-tokenizer/src/datrie.rs`) and `wacha/src/graph.rs` (originally
  `neural-engines/AXIOM/crates/tle-axiom-gen/src/graph.rs`) — see §3 for why both are vendored copies, not
  path dependencies.
- Research: [`knowledge-base/topics/thai-dictionary-hackathon.md`](../knowledge-base/topics/thai-dictionary-hackathon.md)
  (full cited findings), [`knowledge-base/local-inventory/katgpt-rs.md`](../knowledge-base/local-inventory/katgpt-rs.md)
  and [`knowledge-base/local-inventory/neural-engines-workspace.md`](../knowledge-base/local-inventory/neural-engines-workspace.md)
  (AXIOM's "forensic archive" entry).
- Root workspace docs: `../README.md` (add this folder to the workspace index once it exists),
  `../HANDOFF.md` (root handoff — points to the unrelated `dg-sdlm`/`mango-a100` tracks; this is a third,
  independent active track).
- Persistent cross-session context: Claude's memory system, `hackathon-dictionary-reimagined-2026` entry.
