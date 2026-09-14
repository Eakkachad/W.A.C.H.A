# Round 11 — "วันนี้อยากให้ภาษาไทยทำอะไรให้คุณดี": a job-menu front door + two real new
# capabilities (sound-symbolism scoring, pretrained semantic vectors)

**Written by the strategy/review agent, following a planning conversation with the user** (naming
market research, sound-symbolism research, a live audit of `katgpt-rs` that found nothing usable,
and a decision to use PyThaiNLP's pretrained `thai2fit_wv` instead of training anything from scratch).
Same standing rules as R5B–R10: commit per phase, every claimed number comes from something actually
run, no silent substitution (STOP + document if a phase can't be done cleanly), **never touch
`katgpt-rs`** (re-confirmed this round — audited fresh, nothing in it is usable; see §0), `cargo test`
green after every phase, doc-staleness policy unchanged (fix only false statements).

**The reframe this round implements:** the product stops presenting itself as "search a word, get a
definition" and becomes **"tell us what you want Thai to do for you today"** — a small set of
task-shaped entry points that all draw on the *same* engine (segmentation, relation graph, etymology,
RID definitions, reverse-dictionary). This is not five new products; it is one engine with five doors,
which is what keeps it from becoming unfocused (a concern the user raised explicitly and this plan is
designed around).

---

## 0. Standing decisions from this round's planning conversation (no code, read first)

1. **`katgpt-rs` is definitively out of scope**, re-confirmed by direct inspection this round (not just
   trusting old docs): `katgpt-transformer`'s own code comments say Kimi-K3 runs "**random-init
   weights... output tokens are therefore meaningless (gibberish)**," needs ~17.7 GB RAM even at that
   stage, and the other ~28 crates (attention kernels, KV-cache compression, quantization, spatial
   "sense" octrees) are LLM-*serving-speed* infrastructure with nothing to serve — none of it contains
   trained Thai semantics. **Do not integrate anything from it beyond `katgpt-tokenizer`, which is
   already in use.** If asked in the pitch: say this plainly (see `MENTOR_BRIEF.md` update in Phase P) —
   it reads as engineering maturity, not a gap.
2. **No embedding gets trained from scratch.** Instead: **PyThaiNLP's `thai2fit_wv`** (51,556 words,
   300-dim, word2vec/gensim format, MIT-family PyThaiNLP ecosystem — **confirm the exact licence
   yourself in Phase VEC before embedding anything**, same discipline as the R9 D1 / R10 R3 licence
   audits) — pretrained, free, from the same upstream project already trusted for the 62,107-word CC0
   list. Zero training time; the work is integration + honest coverage measurement, not model-building.
3. **Sound symbolism is real enough to use, with a hedge.** Cross-linguistic bouba/kiki-type effects are
   well-replicated (Styles & Gawne 2020, Royal Society; Lockwood & Dingemanse 2015 review); Thai-specific
   evidence is thinner (mostly onomatopoeia studies, e.g. Rungrojsuwan on Thai ideophones) — present the
   hard↔soft word-character score as "a pattern from general phonetic-symbolism research, applied to
   Thai," never as "proven Thai psycholinguistics." The scoring heuristic is fully deterministic (no ML),
   which fits the project's existing "modelless, explainable" identity better than the embedding idea it
   replaced.
4. **Naming and the etymology "romantic bridge" are the same feature, not two.** Market research this
   round found every existing Thai naming tool (Myhora, Mongkolname, Thaibabyname, ชื่อดี AI, etc.) is
   numerology/astrology-based; **none** ground a name in real etymology — this is a genuine, citable gap
   (see `MENTOR_BRIEF.md`'s updated §4 for the full citation list). The English-cognate "bonus" layer
   (inspired by the separate `ThaiDict_Script` / "Etymological Bridge" repo the user pointed at) is
   real *in spot-checks* but has **zero source citations in its own data** — treat its 43 entries as a
   candidate seed to cross-verify against `kaikki_th.jsonl` (already on disk, has real Wiktionary
   etymology sections), not as ready-to-ship fact. Scope it small and honest (Phase BRIDGE, droppable).

---

## PHASE ROUTER — the job-menu front door (must-do; ties everything below into one coherent product;
UX/UI 15% + reframes Problem Fit 25% around a broad, relatable audience)

**What it actually is:** almost entirely a **frontend presentation layer change**, not new backend logic
— `Engine::lookup` (`wacha/src/lib.rs:565`, already returns segmentation + entry + related + learner,
and after Phase U2 also etymology/sub_entries/translit/evolution) already answers "everything about this
word" in one call. The job menu decides *which entry point* a user starts from and *what emphasis* the
same result gets, not what data exists.

1. **New landing state in `wacha/web/index.html`** (before the current single search box takes over):
   5 cards, each a short label + one-line promise, matching the menu decided in the planning
   conversation:
   - **ตั้งชื่อ** — "ตั้งชื่อลูก คนรัก สัตว์เลี้ยง บริษัท ผ่านรากศัพท์จริง ไม่ใช่เลขศาสตร์"
   - **หาคำให้ใช่กับสิ่งที่กำลังเขียน** — register-filtered search + rhyme finder (Phase WRITE)
   - **เข้าใจศัพท์เฉพาะทาง** — routes into the existing specialized-domain lookup (Phase E, already
     shipped)
   - **สำรวจรากคำ/วิวัฒนาการคำ** — routes into the existing etymology + evolution-timeline display
     (U1/W, already shipped)
   - **ตรวจคำทับศัพท์** — routes into the existing transliteration assistant (Phase T, already shipped)
2. Each card sets a **mode flag** (query param or client-side state, e.g. `?mode=naming`) that:
   - changes the search box's placeholder text/prompt copy to match the job ("พิมพ์คำที่มีในใจ หรือ
     บอกความรู้สึกที่อยากได้" for naming; "พิมพ์คำ แล้วเลือกระดับภาษาที่ต้องการ" for writing; etc.)
   - decides whether a plain `lookup()` call or a `reverse` (BM25) call fires first — naming's
     "context-clue" path and the writing-helper's "find a word matching this feel" path both go through
     the **existing** `ReverseIndex`/`reverse_json` (`wacha/src/reverse.rs`, `wacha/src/bin/web.rs:262`)
     — no new search backend.
   - reorders which sections of the existing unified profile (`render(d)` in `index.html`) are shown
     first — naming mode leads with รากคำ + คำใกล้เคียง; writing mode leads with register/rhyme;
     specialized mode leads with the domain cross-links; etc. The data is identical either way.
3. A "ค้นหาทั่วไป" (general search) escape hatch stays available from every mode — the job menu is a
   *front door*, not a cage; nothing already working should become harder to reach.

**Acceptance:** all 5 cards navigate to a working mode with correctly reordered/relabeled output for at
least one real demo word per mode; the underlying `/api/lookup` and `/api/reverse` responses are
byte-identical to before this phase (proves this phase changed presentation, not data).

---

## PHASE NAME — Naming assistant (must-do; the flagship; Problem Fit 25% + Innovation 20%)

Builds *inside* the ROUTER's "ตั้งชื่อ" mode. Two paths, both reusing existing engines:

1. **Direct path** (user already has a candidate word): call the existing unified `lookup()`; render the
   existing profile with naming-oriented copy ("ความหมายของชื่อนี้", "รากคำ", "คำใกล้เคียงที่อาจเป็น
   ทางเลือกอื่น") instead of dictionary-oriented copy. No new backend work.
2. **Context-clue path** (user describes a desired quality — "อยากได้ชื่อที่แปลว่าเข้มแข็ง"): call the
   existing `ReverseIndex` reverse-dictionary. **Known limitation, inherited from Q2:** un-curated testing
   previously found 3/10 clear hits, 2 structural failures on arbitrary queries. For this phase:
   - Curate and **publish the demo query list you know works** (pick from the earlier 3 confirmed
     good queries plus any new ones you verify), exactly like `PITCH.md`'s existing pre-verified word
     list — do not improvise untested queries live.
   - Phase VEC below (if it lands) directly improves this path's recall, so land VEC before finalizing
     this phase's demo query list if both are in scope this round.
3. Each candidate name result carries, where the data supports it (not every word has every field):
   meaning (RID) → รากคำ (etymology, U1) → คำใกล้เคียง (graph) → [optional bonus, Phase BRIDGE] English
   cognate → [optional, Phase SOUND] hard/soft character badge.

**Acceptance:** both paths produce real, correct results for at least 5 demo names/words end-to-end
(spot-checked by actually running `wacha lookup`/`wacha reverse`, not assumed); the "no PIE cognate for
this word" case renders as a normal, complete-looking result (not a broken/empty-looking one) — native
Thai (Kra-Dai) words are the majority case, not the exception, and must look intentional, not degraded.

---

## PHASE SOUND — deterministic sound-symbolism badge (should-do; cheap; Innovation 20%, small delight
layer inside PHASE NAME, not a standalone feature)

Pure function, no new data needed — operates on the `pronunciation`/reading string already in
`Entry`/`EntryView`. Scoring heuristic from this round's research (apply per syllable, average, map to
an axis — verify the sign conventions against a handful of known words before trusting):

- **+2** per plosive/stop consonant (onset or coda: ก ต ป ข ค ท พ ...)
- **+1** per onset consonant cluster (กร ปล ตร ...)
- **+1** short vowel / **−1** long vowel
- **−1** back rounded vowel (โ-, -ู, -อ) / **+1** front unrounded vowel (อิ, เ-)
- **−2** per nasal/liquid/glide (ง น ม ย ร ล ว) in onset/coda
- dead syllable (stop-final/short) **+1**; live syllable (sonorant-final/long) **−1**
- tone: high/falling **+1** (weakest-evidence component — consider a smaller weight or making it
  optional; say so if you do)

Sum → average per word → map to a small discrete scale (e.g. 5 buckets, "แข็ง/หนักแน่น" ↔ "นุ่ม/อ่อนโยน")
with a one-line "why" (top 1-2 contributing features), same explain-everything discipline as the
relation graph. **Mandatory hedge in the UI copy, not just this doc:** something like *"รูปแบบจากงานวิจัย
เรื่องสัทสัญลักษณ์ทั่วไป ยังไม่ใช่ข้อพิสูจน์เฉพาะภาษาไทย"* — do not present this as settled fact.

**Acceptance:** function is unit-tested against a handful of hand-picked contrastive pairs where the
direction is intuitively unambiguous either way (e.g. a word loaded with stops/short vowels vs. one
loaded with nasals/long vowels) and produces the expected relative ordering; UI shows the hedge text
every time the badge appears, not just once somewhere easy to miss.

---

## PHASE WRITE — "หาคำให้ใช่กับสิ่งที่กำลังเขียน" (should-do; register filter + rhyme finder;
Language Accuracy 15% + a second cheap, novel-relative-to-market feature)

### WRITE-1 — Register filter (near-zero cost — the data already exists)

`Register` (`wacha/src/dictionary.rs:215`: แบบ/โบ/ปาก/ราชา/เลิก) is already parsed and stored per sense
but never exposed as a **filter/facet** — only shown passively as a tag on results that already have one.
Add: a search mode where the user picks a target register ("อยากพูดแบบทางการ" / "ราชาศัพท์" / …) and gets
words matching that register for a topic/near-meaning query (compose with the reverse-dictionary: filter
its candidate set by `Sense.register` before ranking).

### WRITE-2 — Rhyme finder (near-zero marginal cost — reuses pronunciation data already loaded)

Research this round found existing Thai rhyme tools (RhymeThai, ThaiRhyme) are old and purely
spelling-based, never combined with meaning. Implement a **loose rhyme matcher**, not classical-poetry-
strict matching (full เอก/โท tone-and-meter matching was explicitly flagged this round as a scope trap —
do not build that; this is the bounded, safe subset of the idea):

- Extract each headword's **final syllable's vowel + final-consonant class** from its `pronunciation`
  string (real samples are plain Thai-script phonetic respellings like `กอ`, `กะตุก`, `กะถินนะทาน` — not
  IPA; verify your extraction against ~20 real samples from `data/rid/dict_2554.txt` before trusting it,
  same as every other parser in this project).
- Index headwords by that key; `wacha rhyme <word>` / `/api/rhyme?q=` returns same-key words, ranked by
  frequency (`tnc_freq.txt`, already loaded) as a relevance tiebreak.
- Cross-link into PHASE NAME: rhyming with an existing family member's name is a genuine naming
  consideration — surface rhyme candidates as an option inside the naming flow, not only standalone.

**Acceptance:** register filter returns only words carrying the requested register tag, verified on a
real query; rhyme finder returns real headwords for ≥5 test words, verified by ear/spelling that they are
plausible Thai rhymes, not just an accidental key collision.

---

## PHASE VEC — pretrained semantic vectors from `thai2fit_wv` (should-do; the one genuinely new,
higher-risk piece this round; enriches reverse-dictionary recall + unlocks future word-relatedness games)

**Do not train anything.** Integrate PyThaiNLP's existing pretrained vectors.

1. **Licence gate first (do this before downloading anything into the repo).** Confirm the exact licence
   terms for `thai2fit_wv` / the thai2fit/ULMFit-Thai project redistribution as shipped by PyThaiNLP.
   Write the answer down (same format as `VERIFY_R9.md` §D1 / `VERIFY_R10.md` §R3) before proceeding. If
   unclear, treat it the same way RID/ศัพท์บัญญัติ were treated: fine for the local/judge build, gated out
   of any public deploy until confirmed.
2. **Download once** (`pythainlp.word_vector.WordVector(model_name="thai2fit_wv").get_model()`, needs
   Python + `gensim` — a one-time offline step, not a runtime dependency of `wacha`).
3. **Measure real coverage before claiming anything:** intersect the 51,556 thai2fit vocabulary against
   the current `wacha` vocabulary (72k+ words). Report the actual overlap count/percentage — do not
   assume full coverage; RID's rarer/archaic/specialized headwords are the likely gap.
4. **Export only the overlapping subset** to a compact binary blob (same pattern as the existing
   `.datrie.cache`/`.pagerank.cache` files — generated once, gitignored, regenerable) — no need to ship
   all 51,556×300 floats if a meaningful fraction don't even exist in the dictionary's own vocabulary.
5. **Nearest-neighbour lookup: a flat array + cosine similarity is sufficient.** Do not reach for any
   specialized indexing structure — at this vocabulary size a linear scan or a simple approximate index
   is fast enough; this was a deliberate rejection of `katgpt-sense`'s octree substrate this round (wrong
   scale, wrong problem, and it's not real anyway — see §0.1).
6. **Sanity-check quality before wiring it into anything user-facing:** spot check that known-related
   words (หมา/แมว, ครู/อาจารย์) are closer in this space than unrelated pairs. If the pretrained vectors
   disagree badly with the hand-verified relation graph on the seed words, STOP and report it honestly
   rather than shipping a contradictory secondary signal.
7. **Integration point:** feed cosine-similarity neighbours as an additional, clearly-labeled candidate
   source into the reverse-dictionary's ranking (alongside its existing BM25 signal), not as a
   replacement — same "graded confidence, not silent override" discipline the relation graph already
   uses for WordNet-vs-hand-verified tiers.

**Acceptance:** licence confirmed and documented; real coverage % reported; nearest-neighbour sanity
check passes on ≥5 known pairs; reverse-dictionary demo queries re-run before/after to show a measured
recall change (even if small — report the real number, don't round up).

**Droppable checkpoint:** if licence terms are unclear or ambiguous by the time you'd otherwise start
downloading, **stop and report it** rather than guessing — this mirrors the R9 D1 / R10 R3 pattern
exactly, and this phase is should-do, not must-do.

---

## PHASE BRIDGE — English-cognate "romantic" bonus layer (droppable; cut first if short on time)

Only attempt if PHASE NAME, SOUND, and WRITE are done with real time left. Source: the 43 hand-crafted
entries in the separate `ThaiDict_Script` repo's `seed_corpus.py` (Sanskrit/Pali → PIE root → English
cognates, e.g. มารดา ↔ mother/maternal/matriarch/matrix). Spot-checked twice this round (มารดา, ศูนย์) and
found linguistically correct and appropriately careful (distinguishes true cognates from loan-translation
chains) — but **the data carries zero source citations**, so:

1. **Cross-verify each of the 43 entries** against `kaikki_th.jsonl` (already on disk, has real
   Wiktionary etymology sections) before using any of them. Keep only entries you can independently
   corroborate; drop or flag the rest.
2. Port the surviving entries into the existing `Etymology` model (`wacha/src/dictionary.rs`) as an
   additional, clearly-sourced field — e.g. `english_cognates: Vec<String>` — rather than a parallel data
   structure.
3. **Label honestly in the UI:** this only applies to the subset of Thai vocabulary with Sanskrit/Pali →
   Indo-European ancestry (a minority of words) — make clear this is a bonus for words that have it, not
   a universal claim, exactly like PHASE NAME's native-Thai-word handling above.

**Acceptance (if attempted):** every surfaced cognate claim traces to a corroborating source you actually
checked (Kaikki/Wiktionary or an independent reference), not just the seed file's own say-so; the feature
degrades gracefully (no broken UI) for the majority of words that don't have one.

---

## PHASE P — Pitch materials pass (after the code phases)

1. Reframe the deck's Problem Statement / Target User section around the job-menu concept — lead with
   *"วันนี้อยากให้ภาษาไทยทำอะไรให้คุณดี"* rather than any single persona; use the naming-market-gap
   citations from `MENTOR_BRIEF.md` §4 as the headline evidence for Problem Fit.
2. Add the honest `katgpt-rs` line from §0.1 above if the deck or live Q&A goes there — frame it as
   engineering discipline (chose the one proven piece, left the rest), not as a gap to hide.
3. Add the sound-symbolism feature with its hedge intact in any slide/demo script that shows it — never
   let the hedge get edited out for punchiness.
4. Re-run `scripts/verify_pitch.sh` after all of the above — same standing rule as every prior round.

---

## Order

```
ROUTER → NAME → SOUND → WRITE → VEC → BRIDGE → P
```

- **ROUTER first** — it's the frame everything else hangs on, and it's mostly presentation-layer, so it
  de-risks early.
- **NAME is the must-do floor** alongside ROUTER — it's the flagship, the market gap is real and cited,
  and most of its plumbing already exists.
- **SOUND and WRITE are cheap, should-do, in either order** — both are small, self-contained, and don't
  block each other.
- **VEC is the highest-risk should-do** — not because it's hard, but because it's the one phase with an
  external dependency (licence terms, download, a new data format) outside this project's existing
  patterns. Time-box it; if the licence check stalls, drop it per its own droppable checkpoint.
- **BRIDGE is cut first.** It's genuinely charming (the user's own "romantic" framing is correct) but it
  is bonus content sitting on unverified data — do not let it consume time that NAME/SOUND/WRITE need.
- **Standing rule, unchanged:** if a phase can't finish cleanly, stop it, commit what's green, and write
  down why in this round's `VERIFY_R11.md`. Every previous round that followed this produced a better
  outcome than pushing through would have.

**Deliverable:** `VERIFY_R11.md` (status table + per-phase real numbers, same format as `VERIFY_R10.md`),
`BENCHMARKS.md` updated if any new latency-sensitive path is added (rhyme index build time, vector
lookup time), dated `PROGRESS.md` entry, `MENTOR_BRIEF.md` updated to reflect what actually shipped vs.
what's still 🔭 vision.
