# Round 13 — Astro + pnpm frontend, built on the `UIdesign/` reference (~5-hour budget)

**Written by the strategy/review agent.** Budget: **~5 hours total, today (2026-09-15), for this round.**
Scope is tiered hard into must/should/cut so a "fast" execution agent can self-triage against the clock —
**stop at any phase boundary and ship what's green rather than leaving everything half-done.** Standing
rules unchanged from R5B–R12: commit per phase, every claimed number/behavior verified by actually running
it, no silent substitution, **never touch `katgpt-rs`**, `cargo test` green if any Rust changes (there
should be none — this round is frontend-only), do not weaken or remove any of the 7 working API endpoints.

---

## 0. What this round is and isn't

**Is:** take the already-designed, WACHA-specific frontend in `data/uidesign_ref/` (index.html, style.css,
app.js, d3_graph.js — ~4,000 lines, a calm "digital library of a language institute" aesthetic, explicitly
built with our own job-menu concept and headline copy already in it) and (a) stand it up as a real Astro
project via `pnpm`, (b) rewire its API calls to our **actual, single, already-tested** `wacha-web` Rust
backend (it currently expects a second, nonexistent Python service on port 8089 for 2 of its features —
see §1), and (c) extend it to cover all 5 job-menu modes + the intent router, not just the 2 it currently
implements.

**Is not:** a rewrite of the backend. Zero Rust changes are expected this round. If a needed capability is
missing from the 7 existing API endpoints, that is a stop-and-report situation, not a "quickly add a Rust
endpoint" situation — this round's clock is for frontend work.

**Design reference, read before writing any code:**
- `data/uidesign_ref/DESIGN_BRIEF.md` — the full UX/UI spec (§2 Design Direction, §4 Color System, §5
  Typography, §6-8 Layout/Header/Home are the load-bearing sections; skim the rest).
- `data/uidesign_ref/{index.html,style.css,app.js,d3_graph.js}` — the actual (partial) implementation.
  **Not everything in the brief is implemented** — the real code only has 2 of the eventual modes wired
  up (`general` search and `etymology`). Trust the code over the brief where they'd conflict.

---

## 1. The compatibility problem this round exists to fix (read before touching app.js)

`data/uidesign_ref/app.js` calls three URLs, verified by direct inspection:

| Call | Points at | Compatible with our real backend? |
|---|---|---|
| `fetch('http://127.0.0.1:8080/api/lookup?q=' + ...)` (line ~677) | our real `wacha-web` (default port 8080) | **Yes, almost as-is** — see §2 for the exact real response shape, which now has *more* fields (etymology, cognates, evolution potential) than this old code reads. |
| `fetch('http://127.0.0.1:8089/api/etymology/' + ...)` (line ~770) | a **second, Python, nonexistent** service | **No.** Rewire to call our real `/api/lookup?q=` and read `entry.etymology` / `entry.english_cognates` instead (§2). |
| `fetch('http://127.0.0.1:8089/api/graph/' + ...)` (line ~872) | same nonexistent second service | **No.** Rewire to build the D3 graph input from our real `/api/lookup`'s `related` array (§2) instead of a separate graph endpoint. |

**Do not stand up a second server.** One Rust binary, already tested end-to-end, is the whole point of this
project's reliability story — adding a second Python process the night before/day of judging is pure risk
for zero benefit, since everything the second service was going to provide already exists in `/api/lookup`.

---

## 2. Authoritative API contracts (copy these exactly — do not re-derive or guess)

Base URL in production: whatever `wacha-web` is bound to (currently `http://100.76.70.14:8090` on the
team's Tailscale net — confirm the live port before hardcoding anything; better, make it a single
`const API_BASE` constant configurable at build time). All 7 real endpoints, verified against
`wacha/src/bin/web.rs` directly:

### `GET /api/lookup?q=<word>` — the main one; powers general search, naming (direct path), specialized,
roots
```json
{
  "query": "มารดา",
  "segmentation": [{"text": "มารดา", "in_vocab": true}],
  "entry": {
    "word": "มารดา", "pos": "น.", "definition": "แม่",
    "classifiers": [], "subject": null, "register": "แบบ",
    "source": "Kaikki (Wiktionary)", "license": "CC BY-SA",
    "examples": ["..."],
    "etymology": [{"lang": "...", "form": "ยืมมาจากบาลี มาตา"}],
    "sub_entries": ["..."],
    "english_cognates": [{"word": "mother", "pie_root": "PIE *méh₂tēr"}]
  },
  "related": [
    {"word": "แม่", "score": 15.8, "source": "wordnet", "confidence": "unverified", "path": ["มารดา --มีความหมายเหมือนกับ--> แม่"]}
  ]
}
```
`entry` is `null` if the word has no dictionary entry (still show `segmentation`). `related[].source` is
one of the tags already rendered in the old `wacha/web/index.html` (`ตรวจแล้ว`/`wordnet`/`wiktionary`/etc
— check `wacha/src/relations.rs`'s `RelationSource` if you need the full enum). **`english_cognates` may be
an empty array — most words (all native Thai/Kra-Dai vocabulary) will have none; render that state as
normal/complete, not as an error or a gap** (this was an explicit R12 requirement, don't regress it).

### `GET /api/reverse?q=<description>` — naming's context-clue path
```json
{"query": "...", "hits": [{"word": "...", "score": 0.0, "matched": ["..."]}]}
```

### `GET /api/intent?q=<free text>` — the router; call this first from the homepage's one search box
```json
{"query": "...", "intent": "naming", "label": "ตั้งชื่อ", "reason": "มีคำว่า \"ชื่อลูก\"", "confidence": "rule"}
```
`intent` is one of `naming | writing | specialized | roots | translit | general`. `confidence` is
`rule | vector | default` — **show the "เราคิดว่าคุณอยาก [label] — ใช่ไหม?" confirmation UI whenever
confidence is `vector` or `default`** (i.e. not a certain rule match), with one-tap links to the other 4
modes, exactly as `wacha/web/index.html`'s existing intent-confirm UI already does — port that same UX
into the new design, don't drop it. `confidence: "vector"` is honestly only ~60-67% accurate (see
`BENCHMARKS.md` §4.8) — this confirmation UI is not optional polish, it's the safety net for that.

### `GET /api/translit?q=<english-or-thai>` — translit mode
```json
{"query": "...", "source": "...", "hits": [{"english": "...", "thai": "...", "note": "..."}]}
```

### `GET /api/evolution?q=<headword>` — roots mode, the 3-edition timeline (ก-headwords only)
```json
{"query": "...", "timeline": [{"edition": "๒๕๔๒", "definition": "...", "is_draft": false, "draft_label": ""}]}
```
When `is_draft` is true, `draft_label` carries the mandatory "ร่าง อยู่ระหว่างดำเนินการ..." text — **always
render it visibly next to that edition, never omit it** (an R11 hard requirement, don't regress it).

### `GET /api/rhyme?q=<word>` — writing mode, rhyme finder
```json
{"query": "...", "rhymes": ["...", "..."]}
```

### `GET /api/register?reg=<แบบ|โบ|ปาก|ราชา|เลิก>&q=<optional topic>` — writing mode, register filter
```json
{"reg": "ราชา", "query": "", "words": [{"word": "...", "freq": 12345}]}
```
`q` is optional — omit it to just list top words in that register; include it to filter reverse-dictionary
candidates to that register (composed search).

---

## PHASE 0 — Astro scaffold (must-do; ~30 min)

1. `pnpm create astro@latest` a new project at repo root, e.g. `webapp/` (confirm the exact folder name
   doesn't collide with anything already in the repo — it shouldn't). Minimal template, TypeScript
   optional (don't burn time on strictness this round).
2. Single-page app is fine — this doesn't need Astro's routing/content-collections features, just its
   dev server + build pipeline. One `src/pages/index.astro` is a reasonable target.
3. Copy `data/uidesign_ref/style.css` in as a global stylesheet import; copy `app.js`/`d3_graph.js` in as
   client-side scripts (Astro supports plain `<script>` tags or `client:*` directives — plain `<script
   is:inline>` / a copied `.js` file referenced normally is the simplest path, don't over-engineer this
   into React components this round).
4. `pnpm install && pnpm dev` must actually run and show the ported page before moving on — verify this,
   don't assume it from file-copying alone.

**Acceptance:** `pnpm dev` serves the ported UI locally, visually matching `data/uidesign_ref/index.html`
opened directly in a browser (i.e., the port didn't break anything cosmetic).

---

## PHASE 1 — Rewire the 2 existing modes to the real backend (must-do; ~1-1.5 hr)

1. Add a single configurable `API_BASE` (env var or a constant at the top of the ported JS) instead of the
   hardcoded `127.0.0.1:8080`/`:8089` — point it at the real running `wacha-web` (confirm the current port
   with `ps aux | grep wacha-web` before hardcoding a default; it was `100.76.70.14:8090` at last check,
   but re-verify).
2. **General search mode:** should work with only the `API_BASE` fix (the response shape it reads already
   matches §2's `/api/lookup` contract for the fields it uses). Verify live with a real query, not just
   code review.
3. **Etymology/roots mode:** rewrite `executeEtymologySearch` to call `/api/lookup?q=` (not
   `/api/etymology/`) and read `entry.etymology`, `entry.english_cognates`, `entry.sub_entries` from that
   response instead of the old `data.entry.pali_sanskrit_form`/`pie_root`/etc shape. Field names differ —
   this is a real rewrite of the parsing logic, not a URL swap. **Also wire in `/api/evolution?q=` here**
   (this mode is the natural home for the 3-edition timeline, which the old 2-mode design didn't have at
   all) — render it below the etymology section, with the draft-label requirement from §2 intact.
4. **D3 graph:** rewrite the graph-data fetch to build its input from `/api/lookup`'s `related` array
   (word/score/source/path) instead of calling `/api/graph/`. If `d3_graph.js`'s expected input shape is
   very different from `related[]`, a small adapter function that maps one to the other is the right
   move — don't reshape the real API to fit an old mock.

**Acceptance:** both modes return real, correct data end-to-end for at least 3 real test words each
(verify by actually running queries against the live backend and reading the rendered output, the same
discipline every prior round used) — not just "the fetch call didn't error."

---

## PHASE 2 — Add the 3 missing modes using the same design system (should-do; ~1.5-2 hr)

Follow the existing `general`/`etymology` mode-card pattern exactly (a card in the mode picker + an
`executeXSearch(query)` function that fetches, renders into `responseBody`, shows the response drawer) —
this is a template to repeat, not a new pattern to invent three times over.

1. **Naming mode** — the flagship (do this one first if only one more mode fits in the remaining budget).
   Two paths per `NEXT_STEPS_R11.md`'s original NAME phase: direct word → `/api/lookup`; context-clue
   ("อยากได้ชื่อที่แปลว่า...") → `/api/reverse`. Reuse the *same* rendering as the etymology mode for a
   resolved word's profile (etymology/cognates/related) — naming is a framing of the same data, not new
   data.
2. **Writing mode** — register filter (`/api/register`) + rhyme finder (`/api/rhyme`), as two sub-panels
   or a small toggle within one mode card.
3. **Specialized mode** — this is just `/api/lookup` again; specialized-domain terms already come back in
   the normal `entry`/`subject` fields (see R10 Phase E) — this mode may not need a distinct fetch call at
   all, possibly just a distinct mode-card entry point with framing copy pointing at the same
   general-search flow. Confirm this by testing a specialized-domain word (e.g. a medical/psychology term)
   through plain `/api/lookup` before assuming a new endpoint is needed — it almost certainly already
   works.
4. **Translit mode** — thin wrapper around `/api/translit`.

**Acceptance per mode landed:** a real end-to-end test (not just visual/code review) with at least 2 real
queries.

**Droppable order if the clock runs out:** Naming → Writing → Translit → Specialized (specialized is listed
last because it likely needs zero new fetch logic, so it's the cheapest to finish even at the very end, or
to explicitly punt with a one-line "coming soon" card rather than leaving it half-wired).

---

## PHASE 3 — Free-text intent router on the homepage (must-do; ~30-45 min)

This is the single most important UX change from R12 and must survive the redesign, not just the 2 old
modes:

1. One search box on the home/hero section (the brief's own §8 Home Page section already calls for the
   search box to be the largest element on the page — this requirement and R12's intent router are the
   same idea, lean into it).
2. On submit: call `/api/intent?q=`, then dispatch to the matching mode's existing render function
   (`executeGeneralSearch`, `executeEtymologySearch`/naming/etc — whatever the phase-2 functions ended up
   named), and show the confirmation line per §2's intent contract — **this is not optional, it is the
   mechanism that makes a wrong guess (which happens ~1/3 of the time on the vector fallback) recoverable
   instead of a dead end.**
3. The mode cards from the original 2-mode design (and the 3 added in Phase 2) stay as an explicit
   override — a user who already knows what they want can still click straight there.

**Acceptance:** typing an unambiguous naming-shaped query with no card clicked lands on the naming render
path with the confirmation line visible; typing a bare word with no clear intent still returns a usable
result (general lookup), never a blank page or error.

---

## PHASE 4 — Verify + deploy (must-do; ~30 min)

1. `pnpm build` succeeds with no errors.
2. Serve the built output (or run `pnpm dev` bound appropriately) reachable from the team's Tailscale
   network, alongside (not replacing, until this is confirmed solid) the currently-running `wacha-web` on
   `100.76.70.14:8090` — e.g. a static file server or Astro's preview server bound to the same Tailscale
   interface on a different port, so there is always a known-working fallback if something in the new
   frontend breaks minutes before pitching.
3. Smoke-test all 5 modes + the intent router live, from a second device on the tailnet if possible (not
   just `localhost`).

**Acceptance:** a URL the team can open on the day, on the tailnet, showing the new design, with all
backend calls hitting the real, single `wacha-web` — and the old UI still available as a fallback URL
until the new one is trusted.

---

## Order & time budget (~5 hours total)

```
0 (30m) → 1 (60-90m) → 3 (30-45m) → 2 (90-120m, droppable per-mode) → 4 (30m)
```

Phase 3 (intent router) is placed **before** Phase 2's remaining modes on purpose — it's must-do and
smaller than finishing all 3 remaining modes, so bank it early. If the clock is genuinely tight, a
defensible stopping point is: Astro scaffold + general search + etymology/roots (with evolution folded
in) + intent router + naming mode, deployed — that alone is a real, coherent upgrade over the current UI
and covers the flagship (naming) plus the concept-defining feature (intent router).

**Standing rule, unchanged:** if a phase can't finish cleanly, stop it, commit what's green, and write
down why. **Do not leave the currently-working `wacha-web` UI in a worse state than before this round
started** — it must stay running and reachable throughout, as the fallback.

**Deliverable:** a short `VERIFY_R13.md` (what actually shipped, per mode, with real test evidence — same
format as prior rounds, doesn't need to be long given the time budget), a dated `PROGRESS.md` entry, and
the live URL(s) for both the new and fallback UI written down somewhere the team will actually see it
before pitching.
