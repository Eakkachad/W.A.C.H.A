# Round 10 — Real Event Data Arrived Tonight: Ingest It Before the Pitch

**Written 2026-09-14, night before the 2026-09-15 Google Form deadline, by the strategy/review
agent (not the execution agent).** This round is triggered by a real external event, not a code
review: the organizer's actual data bundle (`nextect.zip`, 34 MB, downloaded today) is now staged at
`data/official/` (git-ignored, raw). It contains the **real RID excerpt, official transliteration
list, and 3 specialized-domain dictionaries** this project's own `COMPETITION_DAY.md` and
`AGENT_HANDOFF.md` §6 have been anticipating since Round 5 — this is that moment.

**Deadline: tonight.** Pitch deck (PDF/PPT) is due tomorrow, 2026-09-15, via Google Form
(`data/official/Pitching Details_Dictionary Reimagined.pdf`, confirmed 6-page scan). Scope is
triaged hard below into must/should/cut — **stop at any phase boundary if time runs out and ship
what's green**, per this project's standing rule (R5B–R9 all used it correctly).

**Do not re-litigate the architecture.** The hybrid (deterministic Datrie segmenter + explainable
PPR graph, no LLM at runtime) stays. It is a genuine, defensible differentiator specifically
*because* every other team at this hackathon is an AI/dev expert who will very likely wrap an LLM
around the same dictionary text — that path hallucinates and can't reliably reproduce ORST's own
official transliteration rules or an exact 3-edition definition diff, which is exactly where this
round's new features win. This round is about **feeding the existing engine data it has never
seen**, not rebuilding it.

---

## 0. Two corrections to internalize before writing anything (5 min, no code)

1. **The official scoring weights are 25 / 15 / 20 / 15 / 15 / 10** (Problem Fit / Language
   Accuracy / Innovation / Technical Feasibility / UX-UI / Scalability), confirmed directly from
   `data/official/page5_criteria.png` (the actual NECTEC slide). The internal
   `data/official/DATA_SOURCE_CATALOG.md` this bundle shipped with had a **transcription error**
   (15/15 for Innovation/Scalability) — already corrected in that file (see the note dated
   2026-09-14 at its §3.1). **Problem Fit (25%) + Innovation (20%) = 45% of the score** — the two
   phases below that serve those two criteria (R and W) matter more than polish elsewhere.
2. **The current build uses zero organizer-provided data.** `wacha`'s engine is built entirely from
   PyThaiNLP `words_th.txt` (CC0), Thai WordNet, and Kaikki/Wiktionary — none of which is what
   criterion 2 means by "ข้อมูลและเนื้อหาที่ได้รับมอบหมาย" (the data assigned to teams). This was
   the correct call *before* tonight (the real dataset genuinely wasn't available — see
   `PROGRESS.md` 2026-09-13). It stops being correct the moment `data/official/` exists. Closing
   this gap is Phase R below and is the single highest-leverage thing to do tonight.

## 0.5. Mentor feedback tonight (drives the new Phase U below)

A mentor (ORST/NECTEC side) reviewed the direction live and gave two concrete notes — both are
refinements of what's already planned, not a pivot, and both are added as **Phase U**:

1. **"One search should surface many connected dimensions in one place for learning"** — his own
   examples (a fuller definition from another dictionary, near-synonyms, word roots) are illustrative,
   not a checklist. Translation into this codebase: stop treating RID ingestion (R), transliteration
   (T), specialized terms (E), and the evolution timeline (W) as four separate surfaces — **render
   them as sections of one "word profile" view per query**, because the data model already supports
   nearly all of it (see Phase U1 — one field is parsed and silently dropped today).
2. **"The target audience shouldn't be just one group."** `BIBLE.md` §2.2 already honestly flags that
   only the "Thai NLP developer/researcher" audience has real evidence behind it, and that language
   learners/students/translators are an unproven hypothesis. The fix is **not** to fabricate user
   research for those groups — it's to ground the breadth claim in what the product's *own data
   dimensions* demonstrably serve (a translator gets value from near-synonyms, a specialist from the
   discipline-tagged terms, a language-curious user from etymology/evolution), which is both honest
   and now literally more true once Phases R/T/E/W ship. See Phase U2 and the updated Phase P.

---

## PHASE U1 — Surface etymology + ลูกคำ that are already parsed but never shown (must-do first;
15–20 min; no new data needed — do this before R, it's independent and the cheapest real win found
tonight)

**Confirmed by direct code read, not guessed:** `Entry` (`wacha/src/dictionary.rs:418-430`) already
carries `pub etymology: Vec<Etymology>` (รากคำ — ป./ส./อ./ข. markers, parsed by `RidImporter`'s
`ETYM:` handling, `wacha/src/import/rid.rs:93-94,174-190`) and `pub sub_entries: Vec<String>`
(ลูกคำ). **Neither field exists on `EntryView`** (`wacha/src/lib.rs:63-81`), the display projection
`print_lookup` (CLI) and `lookup_json` (web) actually read from — so both are silently dropped
between ingestion and every user-facing surface today. This is exactly the "รากคำ" dimension the
mentor asked for, and it costs nothing to add because the parsing already exists.

1. **`wacha/src/lib.rs`** — add `pub etymology: Vec<(String, String)>` (lang, form — or reuse
   `dictionary::Etymology` directly if it derives the traits `EntryView` needs) and
   `pub sub_entries: Vec<String>` to `EntryView` (line ~63-81); populate both in the `EntryView { ...
   }` construction at line ~578-588 from the matched `Entry` (not just `primary: Option<&Sense>` —
   etymology and sub_entries live on `Entry`, one level up from `Sense`, so pull them from the
   `Entry` being matched, not the sense projection).
2. **`wacha/src/bin/cli.rs`** — in `print_lookup` (line ~578+), right after the `(ที่มานิยาม: …)`
   line (~600), add:
   ```rust
   if !e.etymology.is_empty() {
       println!("รากคำ: {}", e.etymology.iter().map(|(l, f)| format!("{l}. {f}")).collect::<Vec<_>>().join("; "));
   }
   if !e.sub_entries.is_empty() {
       println!("ลูกคำ: {}", e.sub_entries.join(", "));
   }
   ```
3. **`wacha/src/bin/web.rs`** — in `lookup_json` (line ~289+), add `"etymology":[...]` (array of
   `{"lang":...,"form":...}`) and `"sub_entries":[...]` to the JSON entry object alongside the
   existing `examples` field (~343-350).
4. **`wacha/web/index.html`** — in `render(d)` (line ~194+), right after the examples block (~229-233)
   and still inside the `.card.def` definition card, add a small "รากคำ" line (if
   `d.entry.etymology.length`) and a "ลูกคำ" line (if `d.entry.sub_entries.length`, each rendered as
   a clickable chip that re-runs the search — sub_entries are real headwords, so this is free
   cross-navigation, not just a label).
5. Add one test (`wacha/tests/` or inline in `rid.rs`) asserting `ปิตุ`'s or another fixture word's
   etymology survives all the way to `EntryView`, not just `Entry` — the existing `rid.rs` tests only
   check parsing, not that it reaches the display layer, which is precisely the bug this phase fixes.

**Acceptance:** `wacha lookup ปิตุ` (or any fixture/real word with etymology) prints a รากคำ line;
the web JSON response for the same word includes non-empty `etymology`; existing 94+ tests still
pass.

---

## PHASE R — Ingest the real RID excerpt (must-do; Problem Fit 25% + Language Accuracy 15%)

**Source:** `data/official/พจนานุกรม ฉบับราชบัณฑิตยสภาน/DICT_2554 (ก_ซ).xlsx` — confirmed real by
direct read: 13,395 rows, sheet `หน้าหลัก`, columns `headword, headword_sense, read, subject_field,
pos, definition, book, "แม่คำ_0 ; ลูกคำ_1", usage` (plus lower-signal extra columns:
`etymology`, `register`, `see`, `full_form`, `short_form`, `royal_veneration` — ignore these for
tonight, they're a stretch, not a blocker).

**Do not write a new importer from scratch.** `wacha/src/import/rid.rs` (`RidImporter`) and
`COMPETITION_DAY.md` already exist for exactly this file drop, tested (7 fixture tests), and
`COMPETITION_DAY.md` Step 2 explicitly prefers **reshaping the organizer's file to the grammar the
importer already parses** over editing the parser. Follow that preference:

### R1 — Reshape script (30–45 min)

A throwaway Python script (`pandas` is already installed; `openpyxl` was just installed this
session and works) that reads the xlsx and writes `data/rid/dict_2554.txt` in the block grammar
documented in `wacha/tests/fixtures/rid/sample_entries.txt`:

```
HEADWORD: <headword> [<homograph Thai numeral, from headword_sense IF it's ๑/๒/... i.e. a real
           homograph, not a plain sense counter — spot-check this on กก (headword_sense ๑/๒,
           genuinely different entries) vs a word with one row and no headword_sense>]
READING:  [<read>]                          (skip if read is NaN)
SENSE: [<pos mapped through Pos::from_marker — trim trailing '.', "สัน." -> "สัน">] (<subject_field
           if present, else omit>) <definition>
```

Group rows by the `"แม่คำ_0 ; ลูกคำ_1"` column: a run of consecutive `1` rows following a `0` row is
that row's ลูกคำ (compound sub-entries) — reshape each `0` row's block to also carry a `SUBENTRIES:`
line listing the immediately-following `1`-rows' headwords, stopping at the next `0` row. Skip this
grouping if it eats more than ~20 minutes; a flat entry list without `SUBENTRIES` still ingests and
still proves "we used your real data" — `SUBENTRIES` is Phase R's stretch goal, not its gate.

**Known mapping gotcha, spot-checked directly against the real file:** `pos` values in this xlsx are
printed with the trailing dot (`น.`, `สัน.`, `ก.`) exactly as `Pos::marker()` already produces —
these should map through `Pos::from_marker` unchanged; only `สัน` (no dot, per the enum's own comment)
vs `สัน.` (with dot, per the actual data) is a real mismatch to check first, since it is the one POS
tag whose marker constant disagrees with the source file.

### R2 — Ingest per `COMPETITION_DAY.md` Steps 3–5 verbatim (15 min)

```bash
mkdir -p data/rid && mv dict_2554.txt data/rid/
cd wacha && cargo build --release
./target/release/wacha --data ../data stats     # first run rebuilds the trie (~60s cold, expected)
for w in ก กก กระดาษ ก็ เขียน; do ./target/release/wacha --data ../data lookup "$w"; done
```

**Acceptance:** the 5 spot-check words return `ที่มานิยาม: RID ๒๕๕๔ (ORST)` (the existing
`Source::Rid` / `License::OrstEducational` provenance tags — already implemented, nothing to add),
the full `cargo test` suite (94+4 tests) still passes, and `wacha stats` shows the entry count grew
by roughly the reshaped-file's real headword count (not the fixture's 6).

### R3 — Re-run the licence gate this round's own `D1` already opened, now for this file too

`NEXT_STEPS_R9.md` Phase D1 flagged that ศัพท์บัญญัติ content might be inside the public WASM
artifact with no stated public-redistribution licence. **The same caution applies to this RID
excerpt, arguably more strictly** — it is an unpublished (ก–ซ only, "for prototype development")
excerpt handed to teams under an ORST-educational framing, not a general open licence. Before any
public deploy: confirm whether `data/rid/dict_2554.txt`-derived content reaches the public WASM
build; if it does, keep the public demo on the pre-existing public-data layers only and gate the RID
layer behind the local/judge-facing build (the `Importer` seam already makes this a build-time
choice — see `COMPETITION_DAY.md`'s own closing "What NOT to do"). Write the answer down in
tonight's `VERIFY_R10.md`, mirroring `VERIFY_R9.md`'s D1 format — this is exactly the question a
ราชบัณฑิตยสภา judge could ask live.

---

## PHASE T — Transliteration assistant (must-do if R lands; cheap, ~30–45 min; Language Accuracy 15%)

**Source:** `data/official/คำทับศัพท์/คำทับศัพท์ที่ใช้บ่อย.xlsx` — confirmed real: 2,256 rows, sheet
`จากเล่ม`, columns `ลำดับที่, ศัพท์ (English), คำทับศัพท์ (Thai), หมายเหตุ`.

This does **not** touch `Entry`/`Sense`/the graph at all — it's a flat bidirectional lookup, the
cheapest new surface-area win available tonight and a genuinely useful demo beat ("type an English
loanword, get ORST's own official transliteration, both directions").

1. Reshape to `data/translit.tsv` (english\tthai\tnote).
2. New module `wacha/src/translit.rs`: load into two `BTreeMap<String, Vec<String>>` (EN→TH and
   normalized-TH→EN); a handful of lines, no new dependency.
3. New CLI subcommand `wacha translit <term>` + new API route `/api/translit?q=` + one small UI
   affordance in `web/index.html` (a toggle or a second search mode is enough — do not redesign the
   page).

**Acceptance:** 5 round-trip pairs correct in both directions (e.g. `algorithm` ↔ `แอลกอริทึม`/
whatever the file actually says — read it, don't guess the spelling), source line reads "ประมวลคำ
ทับศัพท์ภาษาอังกฤษ พ.ศ. 2563 (ราชบัณฑิตยสภา)", a new small test module.

---

## PHASE E — Cross-discipline specialized terms (should-do if R+T land comfortably; ~45–60 min;
Innovation 20% + Problem Fit 25% "encyclopedia" framing)

**Source:** 3 files, all same schema, confirmed real by direct read:

| File | Rows | Notable |
|---|---:|---|
| `ศัพท์จิตวิทยา.xlsx` | 1,461 | — |
| `ศัพท์ปรัชญา.xlsx` | 1,600 | — |
| `ศัพท์แพทย์.xlsx` | 4,180 | spot-checked: `คำอธิบาย` (the deep-explanation column) is **sparse/often empty** in practice |

Columns: `number, ศัพท์ตั้ง (English), ศัพท์บัญญัติ (Thai coined), สาขาวิชาของศัพท์ตั้ง, เดือนและปีที่
จัดพิมพ์หนังสือ, ศัพท์ตั้งอื่น, คำอ้างอิง, ศัพท์อื่นที่เป็นคำเดียวกันในต่างสาขา, คำอธิบาย`.

**This is the same data shape `wacha/src/import/coined_word.rs` (`CoinedWordImporter`) already
parses** — English term ↔ Thai term(s), grouped by discipline, discipline-scoped `Synonym` links,
`Source::CoinedWord` / `Provenance::Confirmed` (ORST-authored, "the highest-precision relations in
the system" per that file's own doc comment). Don't invent a new importer type — reshape these 3
xlsx files into the same row shape `CoinedWordImporter` consumes (or add a thin TSV-reading sibling
that produces the identical `Entry` construction it already does), so `field`-style cross-discipline
disambiguation just gets 7,241 more rows.

**Honesty note for the pitch, not just the code:** do **not** claim "full encyclopedia definitions"
— the measured reality is authoritative English↔Thai term equivalence with cross-discipline links,
occasionally with a description, matching this project's existing measured-honesty style (see how
R5–R9 always reported the real number, not the aspirational one). The true, still-strong claim is
"7,241 ORST-coined cross-discipline term equivalences, ORST-authored, highest confidence tier in the
graph."

**Acceptance:** at least one lookup per discipline returns the coined Thai term with a subject tag;
at least one demo word shows a `ศัพท์อื่นที่เป็นคำเดียวกันในต่างสาขา` cross-discipline link surfaced
in the UI/CLI output (this is the "dictionary → encyclopedia" claim made checkable on stage).

---

## PHASE W — Word Evolution Timeline, ก only (should-do; the single best Innovation-criterion
beat available tonight, ~45 min; Innovation 20%)

**Why this one matters disproportionately:** it is the feature an LLM-wrapper competitor cannot
credibly fake (it requires the actual historical text of two more dictionary editions, which nobody
else preparing tonight is likely to have thought to diff), it costs almost nothing given Phase R's
reshape script already exists as a template, and it directly answers the organizer's own framing
("ราชบัณฑิตยสภาชำระพจนานุกรมอย่างต่อเนื่อง 2542→2554→2569").

**Scope it to ก only** — that's all `DICT_2569` covers, so there's no reason to reshape more of
`DICT_2542`/`DICT_2554` than that:

- `data/official/.../DICT_2542 (ก_ฮ).xlsx` (37,706 rows total — filter to `ก`-headwords only)
- `data/official/.../DICT_2569 (ก_Incomplete).xlsx` (1,074 rows, already ก-only)
- Phase R's already-reshaped `data/rid/dict_2554.txt` (ก-slice already present)

1. Reshape the ก-slice of all three into one keyed comparison table
   `data/evolution_ko.tsv` (headword, edition, definition).
2. `wacha evolution <headword>` CLI + a small web panel: 3 definitions stacked, edition-labelled.
   **Mandatory integrity label** on the 2569 column: "ร่าง อยู่ระหว่างดำเนินการ ยังไม่เป็นข้อมูล
   ทางการ" (the organizer's own caveat, verbatim from `data/official/DATA_SOURCE_CATALOG.md` §4.1) —
   this is not optional; presenting draft data as final would be the kind of honesty lapse this
   project's whole history (R5–R9) has explicitly avoided.

**Acceptance:** at least 3 real ก headwords rendered across all 3 editions with visibly different
wording — proves the claim on stage rather than asserting it.

---

## PHASE U2 — Unified Word Profile (should-do; do this after R/T/E/W so there's real data from each
to consolidate; ~30–45 min; directly implements the mentor's #1 note — UX/UI 15% + reinforces
Problem Fit 25%)

**The problem this fixes:** as planned above, R/T/E/W each add a *separate* CLI subcommand
(`lookup`, `translit`, `evolution`, plus whatever E exposes) and a scattered set of API routes. That
is fine for verifying each phase works, but it is the opposite of what the mentor asked for — one
search should show the connected picture, not send the user hunting across four tools. This phase
does **not** add new data or algorithms; it only changes what one query call returns and how one
page renders it.

1. **`wacha/src/lib.rs`** — extend `Lookup` (line ~47-59) with the fields the other phases produced:
   `pub translit: Option<TranslitEntry>` (Phase T, only set if the query matches an English loanword
   or its Thai transliteration), `pub evolution: Vec<(String, String)>` (Phase W, edition → definition
   pairs, only non-empty for ก-headwords that exist in ≥2 editions), and confirm `related` already
   surfaces Phase E's cross-discipline terms (it should, if Phase E reused `CoinedWordImporter`'s
   `Synonym`-relation pattern as instructed — verify this rather than assume it).
2. **`Engine::lookup`** (`wacha/src/lib.rs:565+`) — call the (new, small) Phase T/W lookups
   alongside the existing `self.relations.related(...)` and `self.learner.get(...)` calls already
   there, populating the two new `Lookup` fields. Keep each optional/empty-by-default so a query with
   no transliteration or evolution data (the overwhelming majority of words) costs nothing extra to
   render.
3. **CLI** (`print_lookup`, `wacha/src/bin/cli.rs:578+`) — after the existing related-words block,
   add two more optional sections, same style as the existing ones (header line, then indented
   content, then a provenance line): "คำทับศัพท์ที่เกี่ยวข้อง" (if `translit` is `Some`) and
   "พจนานุกรม 3 ยุค" (if `evolution` is non-empty, printed as `edition: definition` per line, with
   the mandatory 2569 draft-label from Phase W carried through unchanged).
4. **Web** (`lookup_json` in `web.rs`, `render(d)` in `index.html`) — same two additions as JSON
   fields and two more cards in the existing card layout (`.card.def` pattern already established at
   `index.html:210+`), positioned directly under the definition card so they read as "more about this
   word," not a separate tool.
5. **One unified page, still one search box** — no new input field, no mode switcher. The existing
   single search box already returns everything; this phase is purely about *what* that one response
   contains and how it's laid out, which is exactly the mentor's ask restated precisely.

**Acceptance:** a single query for a demo word that has data in all four phases (pick one such word
deliberately, e.g. a ก-headword that's also a common loanword-adjacent term if one exists, or accept
showing 2-3 different demo words each lighting up a different section) renders definition + related
words + etymology + evolution + translit sections all from one `wacha lookup <word>` call / one page
load — not four separate commands. This is the single most demo-able proof that "one search, many
dimensions" is real and not just a slide claim.

---

## PHASE X — Regional dialect navigator (cut first if short on time)

6 `.doc`/`.docx` files (North/Isaan/South kinship + body-part terms). Lowest leverage-per-hour
tonight: parsing legacy `.doc` binaries is real friction (`textutil`/`pandoc`, and even then the
content mixes local-script glyphs + IPA that need careful extraction), and it primarily serves UX/
Language Accuracy (15% each) rather than the two 45%-weight criteria above. **Explicitly droppable**
— if attempted at all, do 10–20 curated entries by hand (matching this project's existing
`human_seed` pattern for the learner content) rather than a full parse, purely for one pitch-deck
slide's worth of colour.

---

## PHASE P — Pitch materials pass (after the code phases; can run in parallel if a second person is
free)

1. Fix the scoring-weight assumption everywhere it was used to allocate pitch time/slide emphasis
   (25/15/20/15/15/10, not the miscopied 25/15/15/15/15/15).
2. Add one explicit, checkable line to the deck: *"เราใช้ชุดข้อมูลทางการที่ผู้จัดให้จริงคืนนี้:
   RID 2554 (13,395 รายการ), คำทับศัพท์ทางการ (2,256 รายการ), ศัพท์เฉพาะทาง 3 สาขา (7,241 รายการ)"*
   — pre-empt the "did you actually use our data" question before a judge asks it.
3. Promote the Word Evolution Timeline to the headline beat of slide 4 (Solution Concept &
   Innovation) — it's the feature least reproducible by a same-night LLM-wrapper competitor.
4. **Rewrite slide 2 (Problem Statement & Target User) around Phase U's "one search, many
   dimensions" framing, per mentor feedback:** lead with *"ค้นคำเดียว เห็นทุกมิติในหน้าเดียว —
   ความหมาย, คำใกล้เคียง, รากคำ, วิวัฒนาการข้ามยุค, ศัพท์เฉพาะทาง, คำทับศัพท์"* rather than a single
   narrow persona. Ground the multi-audience claim in what the *product's own sections* serve, not
   invented research — e.g. "นักเรียน/นักแปล ได้ความหมาย+คำใกล้เคียง; ผู้เชี่ยวชาญเฉพาะทางได้ศัพท์
   บัญญัติสาขาตัวเอง; ผู้สนใจภาษาได้รากคำ+วิวัฒนาการ" — one screenshot of Phase U2's unified profile
   view is worth more here than a claim. Do **not** delete `BIBLE.md` §2.2's honesty note that this
   is a product-capability argument, not user-research evidence — keep both true at once.
5. Re-run `scripts/verify_pitch.sh` / `verify_r5.sh` **after** all of the above — new data ingestion
   has broken the rehearsed demo silently before (R6, R7); do not let it happen a third time the
   night before submission.

---

## Order

```
0 → U1 → R1 → R2 → R3 → T → E → W → U2 → P → X
```

- **U1 moved to the front on purpose** — it touches no new data, only wires already-parsed fields
  through to the display layer, and is the cheapest concrete step toward the mentor's "one search,
  many dimensions" note. Do it before R so the habit of extending `EntryView`/`print_lookup`/
  `lookup_json`/`render(d)` together (not just the dictionary model) is established early, since R/T/
  E/W all touch those same four places again.
- **0, U1, R, T are the must-do floor** — R+T alone flip "uses zero assigned data" to "demonstrably
  uses three of the four official data groups" (the single biggest scoring exposure this review
  found); U1 is the cheapest mentor-alignment win available.
- **E and W are should-do**, in either order, if R+T finish with time to spare — pick W first if
  forced to choose only one; it serves the 20%-weight criterion, E serves a 25%-weight criterion
  but with softer differentiation (competitors can also show a coined-word lookup; almost none will
  think to diff three dictionary editions).
- **U2 comes after R/T/E/W, not before** — it consolidates their outputs into one view, so it needs
  at least some of them to already exist. If only R+T ship, do a smaller U2 (definition + related +
  etymology + translit only) rather than skip it — the consolidation matters more than how many
  phases feed it.
- **X is cut first.**
- **Standing rule, unchanged from every prior round:** if a phase cannot finish cleanly, stop it,
  commit what's green, and write down why in tonight's `VERIFY_R10.md`. Every previous round that
  followed this produced a better outcome than pushing through would have — this is truer tonight
  than ever, with a hard submission deadline instead of a soft one.

---

## Questions worth asking the ORST/NECTEC judges and mentors live tonight

These are feasibility/scope questions this review could not resolve from the documents alone —
genuinely worth confirming in person rather than guessing, since the team has the organizers in the
room right now:

1. **Licence for public deploy:** is there a stated licence for redistributing ศัพท์บัญญัติ /
   this RID excerpt in a *publicly deployed* prototype (not just a closed demo), or does "ORST
   educational, non-commercial" mean the public-facing build must exclude it?
2. **What "ใช้ข้อมูลที่ได้รับมอบหมาย" means for scoring:** must the prototype use *only* the
   assigned files, or is blending them with existing public open data (PyThaiNLP/WordNet, already
   in the product) the intended "open data ecosystem" story — i.e., is the hybrid data strategy a
   plus or a minus under criterion 2?
3. **Coverage expectations:** `DICT_2554` covers only ก–ซ (~1/4 of the dictionary). Do judges expect
   full-alphabet coverage via a fallback source for the rest, or is partial official coverage
   acceptable for a 2-day prototype?
4. **`DICT_2569` usage:** it's explicitly "แนวทางออกแบบระบบ" (draft, for design guidance) — do
   judges want to actually *see* a feature built from it (e.g. an evolution comparison), or is
   citing its existence in the deck sufficient?
5. **Timeline clarity:** is 2026-09-15's Google Form the *final* submission, or a pre-submission of
   slides ahead of a separate live-pitching date — i.e., is there more build time after tomorrow, or
   is tonight genuinely the last working session?
6. **Scalability & Impact (10%) emphasis:** does this criterion weigh the open-data/API angle
   (already built — `wacha/API.md`) more heavily than a business/sustainability narrative, or the
   reverse?

---

## What this round deliberately does not touch

- `katgpt-rs` (standing rule, all rounds).
- The segmentation/ranking/graph algorithms themselves — Phases R/T/E/W are pure data-ingestion and
  thin new surfaces, not architecture changes.
- The regional-dialect data beyond an optional, explicitly-labelled-as-cut Phase X.
- Any claim not measured tonight — if a phase runs out of time, the honest move (per every prior
  round) is to say so in `VERIFY_R10.md`, not to imply it shipped.
