# Next Steps — Round 5: ORST-Shaped Data Model & Multi-Source Ingestion

**Status:** written 2026-09-13. Supersedes `NEXT_STEPS.md` (Rounds 1–4, all complete) as the active task
list. `NEXT_STEPS.md` is kept for historical context — do not work from it.

**You are picking up an in-progress hackathon project.** Read in this order before touching anything:
[`README.md`](./README.md) → [`PROGRESS.md`](./PROGRESS.md) (living log, source of truth for what is
actually true right now) → [`AGENT_HANDOFF.md`](./AGENT_HANDOFF.md) (settled decisions + why) →
this file.

**Mandatory habit:** update [`PROGRESS.md`](./PROGRESS.md) — status board + a new dated entry — as you
complete each task. Do not just claim something works. This project's convention (see `PROGRESS.md`
2026-09-05) is that **every "done" claim must be backed by actually running the thing and pasting real
output**, because the last several "done" claims each hid a real bug that only surfaced on execution.
Every acceptance criterion below requires real output, not "tests pass."

---

## 0. Why this round exists (read before planning your work)

On 2026-09-13 the organizers published official practice data sources and — critically — stated that
teams **will receive the real competition dataset at the event** ("ก่อนที่จะได้รับชุดข้อมูลจริงของการแข่งขัน").

Two consequences that drive every task below:

1. **The data we have now is disposable practice data.** The long-standing risk in `AGENT_HANDOFF.md` §6
   ("no confirmed RID dataset") is resolved — but so is the value of over-investing in the current
   LEXiTRON + Thai WordNet layer. What matters now is the **ingestion layer**: whoever can load the
   organizer's real dataset fastest on day 1 wins the build phase.
2. **We know the shape of the real data.** `dictionary.orst.go.th` serves พจนานุกรม ฉบับราชบัณฑิตยสถาน
   พ.ศ. ๒๕๕๔, and its entry structure has been reverse-engineered (§1 below). Our `Entry` model should be
   reshaped to match it **now**, so competition data drops in without a rewrite.

**The problem this round fixes.** Measured 2026-09-13 against the current build:

| Metric | Now | Target after Round 5 |
|---|---|---|
| Words in our list with a definition | **20** (0.03%) | **> 25,000** |
| Words returning a completely empty card | **49,030 (78.9%)** | **< 40%** |
| False word pairs reachable at 2 hops (share no synset) | **8,879** | **0** |
| Words with part of speech shown | **0** | **> 25,000** |
| Sense-level provenance/licence labelling | entry-level only | per-sense |

`PITCH.md` §3 explicitly invites judges to type their own words. At 78.9% empty, a judge from
ราชบัณฑิตยสภา hits "ไม่พบนิยาม" within the first two or three queries. That is the single biggest
risk to this project and Tasks 1–3 exist to remove it.

---

## 1. Reference: the three source schemas (already investigated — do not re-derive)

### 1.1 RID 2554 (`dictionary.orst.go.th`) — the shape the real data will have

Verified by live lookup. One entry contains:

| Element | Example | Notes |
|---|---|---|
| Headword + homograph number | `แมว ๑`, `แมว ๒` | Thai digits. Compound variants shown together: `กรรม ๑, กรรม- ๑` |
| คำอ่าน (pronunciation) | `[กำ, กำมะ-]` | Once per headword, before first sense. Absent when derivable from spelling |
| Word-class marker | `[น.]` `[ก.]` `[ว.]` | **Per sense**, repeated |
| Subject-field marker | `(ไว)` = ไวยากรณ์ | Per sense. Real tag, not just a search filter |
| Numbered senses | `(๑) (๒) (๓)` | Thai numerals |
| Definition text | may embed `<i>Felis catus</i> Linn.` | italic Latin binomials |
| Etymology | `(ป. ปิตา; ส. ปิตฤ)` | ป.=บาลี ส.=สันสกฤต อ.=อังกฤษ ข.=เขมร |
| ลูกคำ (sub-entries) | แมวคราว, แมวเซา ๑, แมวดาว… | labelled line `ลูกคำของ "X" คือ` |
| Cross-reference | `ดู ใบขนุน (๑)` | appears inline (italic) **and** as a structured related-link list |

Controlled vocabularies (from the conditional-search form — these are the real enum values):

- **POS (8):** `ก.` กริยา · `น.` คำนาม · `นิ.` นิบาต · `บ.` บุรพบท · `อ.` อุทาน · `สัน` สันธาน ·
  `ส.` สรรพนาม · `ว.` วิเศษณ์
- **สาขาวิชา (32):** กฎ, การทูต, การเมือง, การศึกษา, เกษตร, คณิต, คอม, เคมี, จริย, ชีว, ดารา, ธรณี,
  บัญชี, ปรัชญา, พฤกษ, แพทย์, ฟิสิกส์, ไฟฟ้า, ภูมิ, มานุษย, แม่เหล็ก, เรขา, วิทยา, วรรณ, ไว, ศาสน,
  เศรษฐ, สถิติ, สรีร, สังคม, สัตว, แสง, โหร, อุตุ
- **ทะเบียนคำ / register (5):** แบบ (literary) · โบ (archaic) · ปาก (colloquial) · ราชา (royal) ·
  เลิก (obsolete)

No JSON API. `POST /func_lookup.php` with `word=<term>&funcName=lookupWord&status=lookup` returns an HTML
fragment; `GET /Lookup/lookupWord_conditional.php?...` returns a JS array literal meant for `eval()`.

### 1.2 ศัพท์บัญญัติ (`coined-word.orst.go.th`) — 89,410 terms, 40 disciplines

A **term-equivalence database, not a definitional dictionary**. Record = Thai term(s) ↔ English term +
discipline + optional หมวดย่อย + optional numbered sub-senses (๑./๒.) + optional bracketed scope note
(e.g. `[ในพีชคณิตนามธรรม]`). There is no long-form definition field.

Mapping is **many-to-many in both directions** — this is the dataset's whole value. Verified example,
`field`:

| สาขา | คำไทย |
|---|---|
| วิทยาศาสตร์ / ศัพท์วิศวกรรมไฟฟ้า | สนาม |
| คอมพิวเตอร์และเทคโนโลยีสารสนเทศ | ๑. เขตข้อมูล ๒. สนาม |
| ภูมิศาสตร์ | ทุ่งเกษตร; แหล่งแร่ |
| เทคโนโลยีทางภาพ | สนามภาพ, ฟิลด์ |
| จิตวิทยา | สนาม, ขอบเขต |
| คณิตศาสตร์ | ฟีลด์ [ในพีชคณิตนามธรรม] |

API: `POST /function_All.php` body `funcName=book_domain` → discipline `<select>` HTML.
`POST /func_lookup.php` body `word=<term>&funcName=lookupWord&book_id=<N>&status=lookup&loc=` —
`book_id=0` = all disciplines. Returns HTML fragments, one `div.panel-info` per matching discipline.

⚠️ **Known trap:** the live dropdown returns only **39** disciplines (ids 1–41, skipping 11 and 37) and
is **missing ธรณีวิทยา (3,303 terms)**, which does appear in `about.php`'s statistics table. An importer
driven off the dropdown loses that entire discipline silently. Hard-code the 40-discipline list.

### 1.3 Kaikki Thai Wiktionary — 43,883 senses / 29,562 word forms

The only one of the three that is legitimately bulk-downloadable. POS breakdown: Noun 23,366 ·
Verb 9,225 · Proper name 4,556 · Adjective 3,661 · Adverb 1,567 · Classifier 181 · Numeral 156 ·
Pronoun 141 · Interjection 290 · Conjunction 87 · Preposition 83 · Particle 107.

Top-level fields observed: `word`, `lang_code` (`"th"`), `lang` (`"ไทย"`), `pos`, `pos_title` (Thai
label, e.g. `"คำนาม"`), `etymology_texts` (**list**, not singular), `translations`, `synonyms`,
`derived`, `related`, `forms`, `anagrams`, `sounds`, `classifiers`, `categories`, `senses`.

`senses[]` elements carry: `glosses` (list of strings), `id`, `categories`, `tags`
(e.g. `["colloquial"]`, `["dated"]`, `["polite"]`), `topics` (e.g. `["computing"]`, `["mathematics"]`),
`examples` (list of `{text, ...}`), `classifiers` (list of `{classifier}` — **ลักษณนาม**).

`sounds[]` includes `{tags:["romanization","Royal-Institute"], roman:"phot-cha-na-nu-krom"}` — the
**ORST romanization standard**, plus Paiboon romanization and IPA.

Four fields here are disproportionately valuable and nearly no competing team will have them:
**ลักษณนาม (classifiers)**, **Royal-Institute romanization**, **usage examples**, and **register tags**
(which map onto RID's ทะเบียนคำ).

Downloads: `https://kaikki.org/thwiktionary/ไทย/kaikki.org-dictionary-ไทย.jsonl` (78.7 MB, Thai-only,
but **marked DEPRECATED** and may disappear) or `raw-wiktextract-data.jsonl.gz` (70 MB gz / 1.6 GB
uncompressed, **all languages** — must be stream-filtered to `lang_code == "th"`).

**Licence: CC BY-SA + GFDL.** See §2 guardrails.

---

## 2. Guardrails — settled decisions, do not re-litigate without new evidence

Everything in `AGENT_HANDOFF.md` §3 still stands (hybrid architecture, `graph.rs`-only from AXIOM, no
`katgpt-transformer`). Additionally, as of Round 5:

1. **Do NOT mass-scrape either ORST site.** Neither offers bulk download; harvesting 89,410 terms by
   POST from a government site days before their own competition is both a courtesy problem and a
   reputational risk. Fetch a **demo subset only** (Task 6: ≤ 500 terms, ≥ 300 ms delay between
   requests, cached to disk so it is fetched exactly once). The real data arrives at the event anyway.
2. **Do NOT present RID content as open data.** `dictionary.orst.go.th`'s disclaimer (an image;
   OCR-read 2026-09-13) states the service is for **educational, non-commercial** use, with data
   copyright held by สำนักงานราชบัณฑิตยสภา and platform copyright by NECTEC. `wacha/API.md` must be
   corrected (Task 9) so we never promise to republish RID as open data. We open *our code* and the
   CC0/CC-BY-SA data — not RID.
3. **Adding Kaikki changes our licence story.** Kaikki is CC BY-SA + GFDL (share-alike), not CC0. The
   combined dataset is therefore CC BY-SA. This is fine and still "open data" — but it must be stated
   accurately and **per field**, which is exactly what the existing provenance system is for. Do not
   silently relicense anything.
4. **Do not delete the WordNet layer.** It gets demoted beneath Kaikki/ศัพท์บัญญัติ, not removed —
   the measured 84.2% precision figure and the confidence-tagging story are still assets.
5. **The demo must stay runnable after every task.** Never leave `main` in a state where
   `wacha-web --data ../data` does not serve a working lookup. Commit per task.

---

## 3. Target architecture

```
                 ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
 sources         │  LEXiTRON    │  │   Kaikki     │  │ ศัพท์บัญญัติ  │  │  RID (real)  │
                 │  words_th    │  │  th JSONL    │  │  demo subset │  │  comp. day   │
                 └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
                        │                 │                 │                 │
                        └─────────────────┴────── trait Importer ─────────────┘
                                                   │
                                          merge by (headword, homograph)
                                                   │
                                          ┌────────▼────────┐
                                          │  Entry / Sense  │  ← RID-shaped, per-sense provenance
                                          └────────┬────────┘
                                                   │
                              ┌────────────────────┼────────────────────┐
                              ▼                    ▼                    ▼
                         Segmenter            RelationGraph          LearnerStore
                        (Datrie)          (Word/Sense/Subject nodes)
```

The key structural change is that **`Sense` becomes a first-class graph node**. Today the graph has only
Word nodes joined by flattened synonym edges, which is what creates the 8,879 false 2-hop links
(`ครอบครัว → บ้าน → บ้านเกิด`, where `บ้าน` belongs to 9 distinct WordNet synsets). Routing every
relation through a Sense node makes cross-sense leakage structurally impossible rather than something
we hand-patch per demo word (which is what `SUPPRESSED_SEED_MEMBERS` currently does for 20 words).

---

## Task 1 — `Entry` / `Sense` model reshaped to RID (foundation; everything depends on this)

**Why first:** every other task writes into this model. Getting it right now is what makes competition-day
ingestion a config change instead of a rewrite.

**What to do.** In `wacha/src/dictionary.rs`, replace the current flat `Entry` with:

```rust
pub struct Entry {
    pub headword: String,
    pub homograph: Option<u8>,          // แมว ๑ -> Some(1)
    pub pronunciation: Option<String>,  // "[กำ, กำมะ-]"
    pub romanization: Option<String>,   // Royal-Institute, from Kaikki sounds[]
    pub senses: Vec<Sense>,
    pub etymology: Vec<Etymology>,      // (ป. ปิตา; ส. ปิตฤ)
    pub sub_entries: Vec<String>,       // ลูกคำ
    pub see_also: Vec<String>,          // ดู
}

pub struct Sense {
    pub pos: Option<Pos>,               // enum: Kri, Nam, Nibat, Buraphabot, Uthan, Santhan, Sapphanam, Wiset
    pub subject: Option<Subject>,       // enum over the 32 RID สาขาวิชา
    pub register: Option<Register>,     // enum: Baep, Bo, Pak, Racha, Loek
    pub definition: String,
    pub examples: Vec<String>,
    pub classifiers: Vec<String>,       // ลักษณนาม
    pub provenance: Provenance,         // PER SENSE, not per entry
}

pub struct Provenance {
    pub source: Source,                 // Lexitron | Kaikki | CoinedWord | Rid | HumanSeed
    pub license: License,               // CC0 | CcBySa | NictPermissive | OrstEducational
    pub confidence: RelationConfidence, // reuse existing enum
}
```

`Pos`, `Subject`, `Register` must be real enums over the §1.1 vocabularies with a `Display` impl giving
the Thai marker (`Pos::Nam => "น."`), plus `from_marker(&str) -> Option<Self>` for parsing. Keep an
`Other(String)` variant on `Subject` only — POS and Register are closed sets.

Migrate the existing 20 hand-curated seed entries into the new shape with
`Provenance { source: HumanSeed, license: Cc0, confidence: Confirmed }`. Preserve every audited relation
decision from the 2026-09-13 audit — do not lose `SUPPRESSED_SEED_MEMBERS`.

**Acceptance criteria**
- `cargo test` passes (54 tests; update tests that construct `Entry` literally, do not delete them).
- `cargo run --release --bin wacha -- --data ../data lookup ครู` produces output equivalent to before —
  paste it into `PROGRESS.md` next to the pre-change output to show no regression.
- New unit test: round-trip `Pos`/`Subject`/`Register` through `from_marker` → `Display` for all
  8 + 32 + 5 values.

---

## Task 2 — `Importer` trait + move existing loaders behind it

**What to do.** New `wacha/src/import/mod.rs`:

```rust
pub trait Importer {
    fn name(&self) -> &'static str;
    fn license(&self) -> License;
    fn load(&self, path: &Path) -> anyhow::Result<Vec<Entry>>;
}
```

Implement `LexitronImporter` (current `words_th.txt`; headword-only entries, no senses) and
`SeedImporter` (the 20 curated entries) against it. Add `merge(Vec<Vec<Entry>>) -> Vec<Entry>` that
unifies by `(headword, homograph)`: senses concatenate (never overwrite), scalar fields
(`pronunciation`, `romanization`) fill only if currently `None`, with source priority
**Rid > HumanSeed > CoinedWord > Kaikki > Lexitron**.

Merge must be **deterministic** — sort sources by priority before merging, and sort senses within an
entry by `(source_priority, pos, subject)`. Two runs over the same inputs must produce byte-identical
output. Add a test asserting that.

**Acceptance criteria**
- `cargo test` passes; new test proves merge determinism across two independent runs.
- New test: merging a Lexitron headword-only entry with a Seed entry for the same word yields one entry
  with the seed's senses and no duplicates.
- Engine still builds and `lookup ครู` output is unchanged (paste it).

---

## Task 3 — Kaikki importer (the task that closes the 78.9% empty-card gap)

**Why this is the highest-value task in the round:** it takes definitions from 20 words to >25,000, and
brings ลักษณนาม, Royal-Institute romanization, usage examples, and register tags along with it.

**What to do.**
1. Add `scripts/fetch_kaikki.sh`: try the Thai-only file first
   (`https://kaikki.org/thwiktionary/ไทย/kaikki.org-dictionary-ไทย.jsonl`); if it 404s (it is marked
   DEPRECATED), fall back to `raw-wiktextract-data.jsonl.gz` and stream-filter `lang_code == "th"`.
   Write the filtered result to `data/kaikki_th.jsonl`. **Do not commit the raw download.**
2. Implement `KaikkiImporter` in `wacha/src/import/kaikki.rs`. **Stream the file line by line** — never
   read 1.6 GB into memory. Map:

| Kaikki | → `Entry`/`Sense` |
|---|---|
| `word` | `headword` |
| `pos` / `pos_title` | `Sense::pos` (map `noun`→`Pos::Nam`, `verb`→`Pos::Kri`, `adj`/`adv`→`Pos::Wiset`, …) |
| `senses[].glosses[0]` | `Sense::definition` |
| `senses[].examples[].text` | `Sense::examples` |
| `senses[].classifiers[].classifier` | `Sense::classifiers` |
| `senses[].tags` | `Sense::register` (`colloquial`→`Pak`, `dated`/`archaic`→`Bo`, `obsolete`→`Loek`, `polite`/`honorific`→`Racha`, `literary`→`Baep`) |
| `senses[].topics` | `Sense::subject` (map `computing`→คอม, `mathematics`→คณิต, `grammar`/`linguistics`→ไว, …) |
| `sounds[]` where `tags` contains `"Royal-Institute"` → `roman` | `Entry::romanization` |
| `etymology_texts` (list!) | `Entry::etymology` |
| `synonyms[].word`, `derived[].word`, `related[].word` | relation triples (Task 4) |

   Every sense gets `Provenance { source: Kaikki, license: CcBySa, confidence: Unverified }` — Kaikki is
   a community source and must not be presented at the same confidence as hand-audited seeds.
3. Entries whose `pos` is `"name"` (proper names, 4,556 of them) should be imported but tagged so the UI
   can distinguish them from common vocabulary.

**Acceptance criteria** — all require real numbers, pasted into `PROGRESS.md`:
- Report the actual count of entries and senses imported. Expect ≈29,562 forms / ≈43,883 senses; if you
  get materially fewer, the filter or parse is wrong — investigate, do not paper over it.
- `lookup ปัญญาประดิษฐ์` (currently "ไม่พบนิยามของคำนี้") now returns a real definition. Paste it.
- `lookup บ้าน` shows its ลักษณนาม (`หลัง`). Paste it.
- Report the new coverage number: % of `words_th.txt` with ≥1 definition. **Target > 40%.**
- `cargo test` passes with ≥3 new tests: a malformed JSONL line is skipped without panicking; a
  `Royal-Institute` romanization is extracted; `etymology_texts` is handled as a list (a singular
  `etymology_text` field does **not** exist — do not write code expecting it).

---

## Task 4 — Rebuild the relation graph with `Sense` nodes (fixes the 8,879 false links)

**The bug being fixed.** `wordnet.rs` discards `synsetid` from `word_synset(synsetid, li)` and emits flat
word↔word pairs. Because `บ้าน` sits in 9 different synsets, 2-hop traversal invents relations across
unrelated senses. Measured on `data/wordnet_th.db`: **26,246 genuine 1-hop pairs, plus 8,879 additional
pairs reachable at 2 hops that share no synset at all** — a 34% inflation with pairs WordNet itself says
are unrelated. `ครอบครัว → บ้าน → บ้านเกิด` currently surfaces at rank 7 **with no ⚠ flag**.

Round 4 already fixed this by hand for 20 seed words (`SUPPRESSED_SEED_MEMBERS`). This task generalizes
that fix to all ~29k words structurally.

**What to do.**
1. Re-extract the WordNet TSV **keeping `synsetid`** (`wacha/data/wordnet_synonyms.tsv` gains a column).
   Regenerate it from `data/wordnet_th.db`; keep the old file until the new path is verified.
2. Give the graph typed nodes — `Word`, `Sense`, `Subject`, `EnglishTerm` — and these edge types:
   - `Word --has_sense--> Sense`
   - `Sense --synonym_of--> Sense` (same synset / same ศัพท์บัญญัติ record)
   - `Sense --in_subject--> Subject` (สาขาวิชา)
   - `Sense --equivalent_en--> EnglishTerm` (Task 6)
   - `Word --sub_entry_of--> Word` (ลูกคำ), `Word --see_also--> Word` (ดู)
3. Traversal rule: a path may pass **through** a Sense node but must not connect two Words via a Sense
   that belongs to a different synset. Keep Personalized PageRank + the BFS explanation path; they now
   operate over a graph where multi-hop is meaningful instead of decorative.

**Acceptance criteria**
- **Write a test that asserts `ครอบครัว` does NOT return `บ้านเกิด`.** This is the regression guard for
  the whole class of bug.
- Re-run the 2-hop false-pair count against the new graph and report it. **Target: 0.**
- `lookup ครอบครัว` and `lookup รถยนต์` pasted into `PROGRESS.md`, before and after.
- Report the new graph size (entities / triples) — it will grow; confirm cold-build time is still
  acceptable and note it.
- Confirm the 84.2% precision figure's scope in `PROGRESS.md`: it was sampled from the 26,242 **direct**
  pairs and never covered multi-hop output. After this task it describes the graph honestly for the
  first time; say so explicitly, because `PITCH.md` quotes it on stage.

---

## Task 5 — Trie cache invalidation (small, but it will ruin the demo if skipped)

Tasks 3 and 6 change the word list, which invalidates `data/words_th.datrie.cache`. The current loader
does not detect this. Two failure modes, both bad on stage: silently segmenting against a stale
vocabulary, or a surprise 43-second rebuild mid-demo.

**What to do.** Write a cache header containing a format version and a hash of the sorted word list.
`load_cache` verifies both and rebuilds on mismatch, logging clearly which condition triggered it.

**Acceptance criteria**
- Modify one word in the source list, re-run, and show the log reporting a hash mismatch and a rebuild.
- Re-run again unchanged and show it loading from cache in milliseconds. Paste both.

---

## Task 6 — ศัพท์บัญญัติ demo subset (the closing demo beat)

**Re-read guardrail §2.1 before starting: demo subset only, ≤ 500 terms, polite delays, fetched once.**

**What to do.**
1. `scripts/fetch_coined_word.sh` — POST to `/func_lookup.php` with
   `word=<term>&funcName=lookupWord&book_id=0&status=lookup&loc=`, ≥ 300 ms between requests, caching
   each raw HTML response to `data/coined_word_cache/`. Fetch a curated list of ~200–500 terms chosen to
   demo well (include `field`, `computer`, `algorithm`, plus terms from คอมพิวเตอร์, คณิตศาสตร์,
   นิติศาสตร์, ภาษาศาสตร์). **Hard-code the 40-discipline list** — the dropdown is missing ธรณีวิทยา.
2. `CoinedWordImporter` parses the cached HTML (one `div.panel-info` per discipline; heading = discipline,
   body = Thai term(s) + English term + optional หมวดย่อย). Handle: semicolon/comma-separated Thai
   synonym lists, numbered sub-senses (`๑.`/`๒.`), and bracketed scope notes (`[ในพีชคณิตนามธรรม]`).
   Emit one `Sense` per (Thai term, discipline) with `subject` set and
   `Provenance { source: CoinedWord, license: OrstEducational, confidence: Confirmed }` — ORST authored
   these, so they are the highest-precision relations in the system.
3. Surface it: looking up a word shows its discipline-specific equivalents, and the graph gains
   `Sense --in_subject--> Subject` edges so "คำอื่นในสาขาเดียวกัน" becomes a real, explainable 2-hop path.

**Acceptance criteria**
- `lookup สนาม` shows the term split across disciplines. Paste it.
- A CLI or web view for `field` reproducing the §1.2 table — this is the closing demo beat: **one English
  word, eight Thai equivalents, disambiguated by ORST's own discipline tags.** Paste the output.
- Report how many terms were fetched and confirm the cache means a re-run makes zero network requests.

---

## Task 7 — Fix the FolkRank citation (15 minutes, do it any time)

`graph.rs`'s comment attributes the hub-correction formula `log π_q(e) − log π(e)` to **Milne & Witten**.
This is **wrong** and `BIBLE.md` §3.3 already flags it as unverified. Verified 2026-09-13: Milne &
Witten's measure is a Normalized-Google-Distance-style overlap over incoming Wikipedia links and the
CIKM'08 paper contains no PageRank content whatsoever.

The correct precedent is **FolkRank** (Hotho et al. 2006; Jäschke et al. 2007) — personalized PageRank
with global PageRank subtracted, both from the same iteration, the global term using a uniform preference
vector. **Caveat that must be stated:** FolkRank uses a plain *difference*; our log-ratio is a variant,
not the published measure.

**What to do.** Correct the comment in `wacha/src/graph.rs`, `BIBLE.md` §3.3 and §6.4, and any pitch
material referencing it. Phrasing to use: precedent is FolkRank; the log form is our own variation,
labelled as such.

**Acceptance criteria:** no occurrence of "Milne" survives `rg -i milne dict-hackathon/`, and BIBLE §3.3's
honesty note is rewritten from "could not verify" to the corrected attribution.

---

## Task 8 — RID importer stub + competition-day runbook

**What to do.**
1. `RidImporter` implementing `Importer`, parsing the §1.1 structure: headword + Thai-numeral homograph,
   bracketed คำอ่าน, per-sense `[POS]` markers, `(สาขาวิชา)` tags, Thai-numeral sense numbering,
   `(ป. …; ส. …)` etymology, ลูกคำ list, `ดู` cross-references. Build it against **fixtures** — 5–10
   entries hand-copied from `dictionary.orst.go.th` into `wacha/tests/fixtures/rid/` (fair use for
   testing; do not bulk-harvest).
2. `COMPETITION_DAY.md` at the repo root: the exact steps to ingest the organizer's dataset — where to
   drop the file, which command to run, how to regenerate the trie cache, how to verify (spot-check 5
   words end-to-end), and what to do if their format differs from the fixtures (which adapter function
   to edit, and what to check first).

**Acceptance criteria**
- `cargo test` passes with fixture-driven tests covering: multi-sense entries, homograph numbers,
  etymology parsing, ลูกคำ, and `ดู` cross-references.
- Time yourself executing `COMPETITION_DAY.md` against the fixtures as if it were real data. **Target:
  under 10 minutes.** Record the actual time in `PROGRESS.md` — if it is over 20 minutes, the runbook is
  not good enough yet.

---

## Task 9 — UI, API contract, and licence accuracy

**What to do.**
1. `web/index.html` + `bin/cli.rs`: show per sense — POS badge (คำนาม/คำกริยา/…), สาขาวิชา, register
   (โบ/ปาก/ราชา), ลักษณนาม, usage examples, and a **source + licence badge**. Keep the existing
   provenance/confidence badges; they now carry licence too. Preserve the existing HTML-escaping on every
   user-input insertion point (Round 3 verified this at the frontend layer — do not regress it).
2. `wacha/API.md`: add the per-field licence table (CC0 / CC BY-SA / NICT / ORST-educational), and
   **remove any claim that RID content is or will be open data** (guardrail §2.2). State plainly that the
   combined dataset is CC BY-SA because of the Kaikki layer.

**Acceptance criteria**
- Open the web UI in a real browser and confirm badges render for a Kaikki word, a seed word, and a
  ศัพท์บัญญัติ word. Describe what you actually saw.
- Re-run the Round 3 adversarial battery (11 cases: empty, 5000 chars, emoji, invalid UTF-8, XSS…) and
  confirm all still pass.

---

## Task 10 — Re-measure everything and update the pitch

Every number in `PITCH.md`, `BIBLE.md` §8 and `PROGRESS.md` is now stale.

**What to do.** Re-measure and update: definition coverage, words with relations, graph size, cold/warm
start, per-query latency, 2-hop false-pair count (should be 0), test count. Then revise `PITCH.md`:
- Replace "~29,000 words" with the honest coverage number.
- Add the ศัพท์บัญญัติ `field` demo as the closing beat.
- Add the multi-source ingestion story — *"ระบบเรารับข้อมูลของท่านได้ทันที นี่คือหลักฐาน เราโหลดมาแล้ว 3 แหล่ง"* —
  which answers the organizers' "คลังคำ → คลังข้อมูล / ต่อยอดได้" objective more directly than any feature.
- Keep every existing honesty device. They are this project's strongest asset.

**Acceptance criteria:** no number in `PITCH.md` or `BIBLE.md` that cannot be reproduced by a command
pasted in `PROGRESS.md`.

---

## Optional Task 11 — segmentation accuracy (only if Tasks 1–10 are done)

We report speed but have never reported segmentation **accuracy**, and `segmenter.rs:109` is *greedy*
longest-match, not the *maximal matching* that PyThaiNLP's newmm uses. Published figures (AttaCut paper,
arXiv:1911.07056, Table 2, read directly from the PDF):

| Dataset | PyThaiNLP (dict) | Sertis | DeepCut | AttaCut-C | AttaCut-SC |
|---|---|---|---|---|---|
| BEST-2010 | 0.67 ± 0.19 | 0.87 ± 0.16 | **0.93 ± 0.13** | 0.89 ± 0.16 | 0.91 ± 0.14 |
| **TNHC** (classical literature) | **0.73 ± 0.21** | 0.70 ± 0.23 | 0.63 ± 0.26 | 0.66 ± 0.24 | 0.63 ± 0.26 |
| Wisesight-1000 | 0.74 ± 0.21 | **0.81 ± 0.18** | **0.81 ± 0.20** | 0.80 ± 0.20 | **0.81 ± 0.20** |

The paper states: *"Despite having the lowest word segmentation performance on BEST-2010 and
Wisesight-1000, PyThaiNLP is the best word segmenter on TNHC"* — §5.4 explains that learning-based
segmenters fail on classical literature because its archaic, poetic vocabulary diverges from BEST-2010
training data. Speed: 7.43 s vs DeepCut's 846.98 s (**113.9×**).

**⚠️ We may not quote 0.73 as ours.** That is newmm's maximal-matching number; we run greedy
longest-match, which is a different and generally weaker algorithm. Borrowing it would violate this
project's core principle.

**What to do:** switch greedy → maximal matching (minimise word count via DP over the existing trie, the
newmm algorithm — still fully deterministic and modelless); then evaluate on `pythainlp/wisesight1000`
(**CC0**, ~74 kB, character-level `is_beginning` labels, vendorable as a test fixture) and report our own
boundary-F1 as per-sample mean±std, following the published protocol. Then the honest and genuinely
strong pitch line becomes available: dictionary-driven segmentation is the right choice for a *dictionary*
because it degrades gracefully on exactly the archaic and literary vocabulary that is ราชบัณฑิตยสภา's
core material — with our own measured number to back it.

---

## Suggested order

**Critical path:** 1 → 2 → 3 → 5 → 4 → 9 → 10.
**Parallel-safe any time:** 7 (citation, 15 min), 6 (needs 1+2 only), 8 (needs 1+2 only).

If time is short, **Tasks 1, 2, 3, 5 alone** convert the project from "a dictionary with 20 definitions"
into "a dictionary with 25,000+ definitions and a proven ingestion path" — which is the difference that
decides how this is received.
