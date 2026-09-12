# Plan — วาจา (WACHA)

Anchor date: 2026-09-03 (planning complete, event dates not yet confirmed). Phases below are relative to
the event's own timeline (Day 0 = before the event; Day 1/Day 2 = the 2-day-1-night hackathon itself), not
calendar dates — fill in real dates in [AGENT_HANDOFF.md](./AGENT_HANDOFF.md) once known and update this
file to match, don't let it silently drift.

## Day 0 (before the event) — De-risk, don't build the product yet

**Goal:** every open question that would otherwise burn hackathon-day hours is resolved ahead of time.

> **Status (2026-09-04):** Day-0 substantially done ahead of schedule — data source resolved, OOV bug
> fixed, Datrie segmenter and vendored graph both scaled into a real crate (`wacha/`) and running on
> the real 62k word list. Remaining Day-0 items: Typhoon 2 access (optional, direction 3) and the team
> UI/product-direction decisions (event-dependent). See `PROGRESS.md`.

- [done] Resolve the data-source question (`AGENT_HANDOFF.md` §6): CC0 `words_th.txt` (62,107 words) +
  `tnc_freq.txt` downloaded to `data/`; Thai WordNet noted as a permissive later option.
- [done] Confirm `katgpt-tokenizer` builds standalone — used as a path dependency by `wacha`; also
  found and fixed a latent `Datrie` collision-resolution panic in it.
- [done] Prototype the `Datrie`-based longest-match segmenter — done in `poc/`, then productized in
  `wacha/src/segmenter.rs` with a TCC-aware OOV fallback (the fix for the POC bug).
- [done] Vendor `graph.rs` from AXIOM and prove `add_triple` + `personalized_pagerank` + `bfs_subgraph`
  on Thai triples — done; now driving the explainable-related-words query in `wacha/src/relations.rs`.
- [todo, optional] Verify Typhoon 2 (or SEA-LION) generation access — only needed for direction 3.
- [todo, event-dependent] Team decision on product direction + rough UI wireframe.

**Exit criteria:** a real (or credible substitute) Thai word list identified and license-checked; the
Datrie segmenter runs correctly on sample text; the vendored graph engine returns a sane explainable path
on hand-written test triples; Typhoon 2 returns a real Thai completion; one product direction chosen by
the team.

## Day 1 (build day)

**Goal:** a thin, working, end-to-end vertical slice — not full feature coverage.

- **Morning:** integrate whatever real dataset the event provides (supersedes the Day 0 substitute if one
  is handed out); scale the Datrie segmenter to the real word list; stand up the search/lookup backend;
  extract real triples from dictionary entries into the vendored graph engine (start with whatever
  explicit relations the data actually carries — don't invent inferred relations yet).
- **Midday–afternoon:** build the minimal UI for the chosen product direction (search box → results →
  definition view → "related words" panel powered by `personalized_pagerank`/`bfs_subgraph`); wire the
  segmenter into the search path.
- **Late afternoon–evening:** wire the generation call (Typhoon 2) if pursuing direction 3 — treat this as
  a single well-tested API call, not a new subsystem.
- **End of Day 1:** get one full user journey working end-to-end (type a word → see a segmented,
  searchable result with an explainable related-words graph, possibly AI-enhanced), even if visually
  rough.

**Exit criteria:** one complete user journey demoable, even without polish.

## Day 1 night → Day 2 morning — Differentiator + polish

- Polish the explainable relationship-graph visualization (the hybrid differentiator) — this is the
  headline feature now, not a stretch item; prioritize it over direction 3 if time is tight.
- Basic UI polish and error-path handling (empty search, word not found, no-triples-found for the graph
  panel, generation-call failure/timeout if direction 3 is in scope — nothing should hang or crash live).
- If genuinely ahead of schedule only: expand triple coverage (more relation types, inferred relations)
  or add direction 3 (AI-simplified definitions) — never let either touch the working core.

**Exit criteria:** the working slice looks presentable; failure paths don't crash the demo.

## Day 2 — Finish, demo prep, pitch

- Bug fixes on the working slice only — no new features started this late.
- Prepare pitch materials: the "two of our own research systems, hybridized — katgpt-rs's tokenizer reads
  the dictionary, AXIOM's graph engine explains it" story (`AGENT_HANDOFF.md` §1) is the differentiation
  narrative; lead with it.
- Rehearse the live demo at least twice, including a rehearsed fallback if the live generation call is
  slow/unavailable (e.g. a pre-recorded backup clip or a cached example response).
- Submit.

**Exit criteria:** submission made; demo rehearsed; team can explain both what works and what was
descoped, honestly (per this workspace's own convention of recording negative results, not just wins).

## Post-hackathon (not deadline-bound)

- If the data-source problem turned out to have a real, reusable answer, consider open-sourcing the
  compiled Thai word list / Datrie segmenter as a standalone artifact — independent of whether the
  hackathon prototype itself continues.
- Consider upstreaming the vendored-`graph.rs` learnings (e.g. any Thai-specific fixes/extensions) back
  into AXIOM's own repo, since it's otherwise dormant ("forensic archive").
- Revisit AXIOM's full text-decomposition/QA engine and its Thai-language gap properly (its README already
  flags "Thai planned") if there's continued interest, as its own multi-session research track, not scoped
  to a 2-day event — this hybrid deliberately never touched that part.
