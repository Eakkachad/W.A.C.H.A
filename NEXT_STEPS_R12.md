# Round 12 — Intent detection (one free-text box, not five buttons) + reviving PIE/BRIDGE
# with real source data

**Written by the strategy/review agent, following on from R11.** Same standing rules as R5B–R11:
commit per phase, every claimed number comes from something actually run, no silent substitution
(STOP + document if a phase can't be done cleanly), **never touch `katgpt-rs`** (settled, do not
re-open), `cargo test` green after every phase, licence-gate before using any external data
(unchanged discipline — this round touches two more external-ish sources).

**What changed since R11, and why this round exists:**
1. The user wants **PIE/BRIDGE developed for real**, not cut. R11 cut it because the source
   (`ThaiDict_Script`, a separate repo) wasn't reachable from this project — **it now is**: copied to
   `data/thaidict_script_ref/` (git-ignored, reference-only, ~25MB, mostly a redundant re-parse of the
   same official DICT_2542/2554 we already ingested — the one asset actually worth extracting is
   `etymological-bridge/data/seed_corpus.py`'s 43 hand-crafted entries).
2. **The product concept has grown past "click a job card, then type."** The user's own framing:
   *"วันนี้คุณอยากให้ภาษาไทยทำอะไรให้คุณ?"* is meant to work as **one free-text box** — the user types
   whatever they want (a bare word, a description, a request), and the system's job is to work out the
   **intent** and route internally, the same way a person behind a counter would. The 5 job cards
   (`wacha/web/index.html:144-148`) currently require an explicit click *before* typing — this round
   adds a real intent classifier so typing alone is enough. The cards stay as an explicit override, not
   as a requirement.

---

## 0. Design decision this round makes explicit (confirm-by-reading, not asking again): intent
detection stays deterministic/rule-based, not a new ML model

Consistent with every architecture decision in this project (`katgpt-rs` ruled out twice now,
`thai2fit_wv` chosen over training anything): **do not build or call a text classifier model for
intent.** Use a small, layered, fully explainable pipeline instead — every step below is either a
literal keyword/pattern rule or a reuse of an asset already in the codebase:

1. **Keyword/pattern rules first** (fast, certain, trivially explainable — "this fired because the
   query contained X"). One rule per intent, checked in priority order.
2. **`thai2fit_wv` cosine-similarity fallback** (already integrated in R11 VEC) *only* when no rule
   fires with confidence: embed the query (average word vectors for multi-word input), compare against
   a small hand-written set of seed phrases per intent (e.g. naming: "ตั้งชื่อ", "ชื่อลูก", "ชื่อบริษัท",
   "อยากได้ชื่อที่แปลว่า..."), take the nearest centroid. This reuses an asset already licence-checked
   and integrated — no new dependency.
3. **Default fallback, unchanged:** if nothing clears a confidence bar, behave exactly like the current
   general lookup (try `Engine::lookup` first; if it returns no entry, try the reverse-dictionary) —
   this is the existing behavior, so an ambiguous query never regresses below what already works today.
4. **Show the detected intent, don't silently commit to it.** Every routed result displays a small,
   dismissable line: *"เราคิดว่าคุณอยาก [ตั้งชื่อ] — ใช่ไหม?"* with the other 4 modes one tap away. This
   is the explainability requirement, not a nice-to-have — it is what keeps a wrong guess from being a
   dead end instead of a one-click fix, and it is what makes "intent detection" honest rather than a
   black box making silent decisions for the user.

---

## PHASE INTENT — deterministic intent router (must-do; this is the user's explicit ask this round)

### INTENT-1 — Rule table (new module, e.g. `wacha/src/intent.rs`)

Design as a pure function `classify_intent(query: &str) -> IntentGuess { intent: Intent, reason: String,
confidence: Confidence }` — `Intent` is an enum mirroring the 5 existing modes
(`Naming | Writing | Specialized | Roots | Translit`) plus `General` (today's default behavior). Keep
`reason` as a short human-readable string (which rule/keyword fired) — this is what powers the "เพราะ..."
explanation if you choose to show one, and it's what makes this testable per-rule rather than as one
opaque function.

Starting rule set (tune against real queries as you test, don't ship the first draft untested):

| Intent | Trigger pattern (examples — expand from real testing) |
|---|---|
| Naming | "ตั้งชื่อ", "ชื่อลูก", "ชื่อบริษัท", "ชื่อสัตว์เลี้ยง", "อยากได้ชื่อ", "ชื่อที่แปลว่า" |
| Writing (register/rhyme) | "ทางการ", "ราชาศัพท์", "ภาษาปาก", "โบราณ", "คล้องจอง", "สัมผัส", "แต่งเพลง" |
| Specialized | "ทางการแพทย์", "ศัพท์แพทย์", "จิตวิทยา", "ปรัชญา", "ศัพท์บัญญัติ" |
| Roots/evolution | "รากศัพท์", "รากคำ", "มาจากไหน", "ประวัติคำ", "วิวัฒนาการ", "ยุคไหน" |
| Translit | contains Latin-script characters, or "ทับศัพท์", "สะกดยังไง", "ภาษาอังกฤษ" |
| General (default) | none of the above matched — existing `lookup()`-then-`reverse()` fallback, unchanged |

**Acceptance:** unit tests for every rule (one query that should hit it, one near-miss that shouldn't),
plus a table-driven test over ≥20 real example queries you write yourself (not hypothetical — actually
type the kind of thing a user would type) with the expected intent, checked in as a regression fixture
so future rule changes can't silently break a previously-correct classification.

### INTENT-2 — `thai2fit_wv` fallback disambiguator

Only invoked when INTENT-1 finds no confident keyword match. Precompute the 5 intent-seed-phrase
centroids once (offline, at engine build time — same "precompute, not runtime dependency" pattern as
the learner content and the R11 VEC blob). At query time: average the query's in-vocabulary word
vectors, cosine-compare against the 5 centroids, return the nearest if above a similarity threshold you
determine empirically (report the threshold and how you picked it — don't hand-wave a number).

**Acceptance:** measure this layer's hit rate on a held-out set of queries INTENT-1's keywords don't
cover (write ~10 such queries yourself); report the real number, including if it's mediocre — this is a
fallback for the *long tail*, it doesn't need to be perfect, and an honestly-reported 60% beats a claimed
90% that isn't measured.

### INTENT-3 — Wire into the web layer + UI transparency

- **`wacha/src/bin/web.rs`**: new route (e.g. `/api/intent?q=`) returning `{intent, reason, confidence}`
  as JSON, following the existing hand-written JSON serialization pattern in `lookup_json`/`reverse_json`.
- **`wacha/web/index.html`**: the single search box (not gated behind clicking a card first) calls
  `/api/intent` on submit, routes to the matching mode's existing render path
  (`render(d)`/`renderReverse(...)`/etc. — whatever R11's mode-specific rendering already does), and
  shows the "เราคิดว่าคุณอยาก [X] — ใช่ไหม?" confirmation line with one-tap links to the other 4 modes.
  The 5 job cards from R11 stay exactly as they are — an explicit override for a user who already knows
  what they want, sitting alongside the new type-first flow, not replaced by it.

**Acceptance:** typing a naming-shaped query with no card clicked lands on the naming render path with
the confirmation line shown; typing something ambiguous falls back to General and still returns a
usable result (not an error, not a blank page) — the whole point of this phase is that guessing wrong
must degrade gracefully, never break.

---

## PHASE BRIDGE — English-cognate layer, revived (should-do; source data now available)

Same design as R11's cut BRIDGE phase, now unblocked. Source: `data/thaidict_script_ref/etymological-
bridge/data/seed_corpus.py` (43 entries: Thai word → Pali/Sanskrit form → PIE root → English cognates,
with difficulty tags like "SAT (C1)").

### BRIDGE-1 — Cross-verify against `kaikki_th.jsonl` (the honesty gate — do not skip)

Confirmed this round: `kaikki_th.jsonl` entries carry a real, checkable `etymology_texts` field (e.g.
the entry for `มารดา` has `"etymology_texts": ["ยืมมาจากบาลี มาตา"]` — "borrowed from Pali mātā," which
independently corroborates `seed_corpus.py`'s claim for the same word). For each of the 43 seed entries:

1. Look up the Thai word in `kaikki_th.jsonl`.
2. Check whether its `etymology_texts` (if present) mentions the same source language (Pali/Sanskrit)
   and, ideally, a matching or compatible source form.
3. **Keep only entries with independent corroboration.** For entries with no `etymology_texts` field,
   or a contradicting one, **drop them or flag them as unverified** in a separate honesty-tagged bucket
   — do not silently keep them at the same confidence as corroborated ones.
4. Record the real numbers: how many of 43 corroborated, how many dropped, how many flagged. This is
   the number that goes in `VERIFY_R12.md` and the pitch — report it exactly, including if it's a
   minority of the 43.

### BRIDGE-2 — Port into the data model

Extend `Etymology` or `Entry` (`wacha/src/dictionary.rs`) with the surviving entries — e.g.
`english_cognates: Vec<EnglishCognate>` per entry, each carrying the word, the PIE root (display only,
not asserted as definitively correct beyond what's corroborated), and a provenance tag distinguishing
"corroborated by Kaikki" from anything you chose to keep as flagged/unverified (if any).

### BRIDGE-3 — Surface it honestly in the naming flow

Add as the existing planned "bonus" line in `PHASE NAME`'s output: *"คำนี้มีคำอังกฤษที่เป็นรากเดียวกัน:
mother, maternal, matriarch"* — shown **only** for words that survived BRIDGE-1's verification, and
**never** implying every word has one (most won't — native Thai/Kra-Dai vocabulary has no PIE ancestry
at all, and that is the majority case, not an edge case).

**Acceptance:** the corroboration count/percentage is in `VERIFY_R12.md`; at least the well-known
spot-checked examples from earlier (`มารดา`, `ศูนย์`) pass verification and render correctly end-to-end
(`wacha lookup มารดา` shows the cognate line); a native Thai word (e.g. `หมา`, `บ้าน`) renders its normal
complete profile with no cognate line and no visual "something's missing" gap.

**Droppable checkpoint, unchanged from R11:** if BRIDGE-1's corroboration rate turns out very low (say,
under a third of the 43), that is a valid, honestly-reported reason to ship a smaller feature or cut it
again — report the number and let it speak for itself rather than forcing a weak feature to exist.

---

## Order

```
INTENT-1 → INTENT-2 → INTENT-3 → BRIDGE-1 → BRIDGE-2 → BRIDGE-3
```

- **INTENT is the must-do floor this round** — it's the user's explicit, direct ask, and it's what
  makes the "one box, many jobs" concept actually true instead of "one box, if you clicked the right
  card first."
- **BRIDGE is should-do** — genuinely wanted, now unblocked, but gated hard on real corroboration
  (BRIDGE-1) exactly like every other data-integrity gate in this project's history (R9 D1, R10 R3).
- Do **INTENT before BRIDGE** if forced to choose — INTENT changes how every existing feature is
  *reached*, BRIDGE only adds one more thing to show once you're in the naming flow.
- **Standing rule, unchanged:** if a phase can't finish cleanly, stop it, commit what's green, and
  write down why in `VERIFY_R12.md`.

**Deliverable:** `VERIFY_R12.md` (same format as `VERIFY_R10.md`/`VERIFY_R11.md`), `BENCHMARKS.md`
updated (intent-classification hit rate, any new latency), dated `PROGRESS.md` entry + Status board,
`MENTOR_BRIEF.md` updated (the product description changes from "search a word" to "tell it what you
want" — make sure the TL;DR and Solution Concept sections reflect that, and that any new capability is
tagged ✅/🔧/🔭 honestly, same convention as always).

---

## One open question for the user, not blocking this plan

The user mentioned having already built UI for this free-text concept ("ฉันได้พัฒนา UI ไปแล้ว"). This
repo's `wacha/web/index.html` still shows only the R11 click-first job cards (verified — no uncommitted
changes, no newer commits touch that file). If there's a separate mockup/design file elsewhere, point
the coding agent at it before INTENT-3 so the new single-box flow matches it rather than improvising a
new layout; otherwise INTENT-3 as scoped above (search box + confirmation line + existing cards kept as
override) is a reasonable default to build against.
