# WordNet-derived relations touching the 20 seed words — full manual audit (2026-09-13)

Scope: every WordNet synonym pair where at least one endpoint is one of the 20 hand-curated seed words
(`wacha/src/dictionary.rs`). 47 pairs total — small enough to check **every** pair (not a sample). These
are the words most likely to be demoed / typed by judges, so they must be judge-defensible.

The overall ~84.2% precision figure for the *whole* 29k-word WordNet graph (PROGRESS.md 2026-09-12,
random sample) is unchanged and still quotable — this audit only cleans the seed-touching subset.

Verdict legend: KEEP = genuine Thai synonym/near-synonym; CUT = not a true synonym (reason given).
Cut pairs are removed via a **group-level** suppression list in code
(`wordnet.rs` `SUPPRESSED_SEED_MEMBERS`): when a WordNet synset group contains a seed word, that seed's
audited-wrong co-members are pruned from the group. This is stronger than suppressing the direct pair —
it also removes 2-hop bridges through the shared synset (e.g. นักเรียน↔นร.↔นิสิต, where นร. legitimately =
นักเรียน but the synset also wrongly contains นิสิต). The source TSV is left intact, so the decision is
traceable and reversible.

| # | seed | ⟷ other | verdict | reason |
|---|---|---|---|---|
| 1 | ภาษา | การสื่อสารด้วยภาษา | KEEP | related/near-syn |
| 2 | เขียน | ขีดเขียน | KEEP | true syn |
| 3 | ครู | ครูบาอาจารย์ | KEEP | true syn |
| 4 | ครู | ผู้สอน | KEEP | true syn |
| 5 | ครู | ผู้สาธิตวิธีการ | **CUT** | "demonstrator" ≠ teacher; WordNet over-broad mapping |
| 6 | ครู | ผู้ให้ความรู้ | KEEP | true syn |
| 7 | ครู | อ. | KEEP | abbreviation |
| 8 | ครู | อาจารย์ | KEEP | true syn (also a seed relation) |
| 9 | อาจารย์ | ครูบาอาจารย์ | KEEP | true syn |
| 10 | พจนานุกรม | ดิก | KEEP | colloquial/loanword syn |
| 11 | พจนานุกรม | ดิกชันนารี | KEEP | loanword syn |
| 12 | เขียน | ทำหนังสือ | KEEP | loosely (write/author) |
| 13 | นักเรียน | นร. | KEEP | abbreviation of นักเรียน |
| 14 | นักเรียน | นศ. | **CUT** | นศ.=นักศึกษา (tertiary); ORST distinguishes นักเรียน (school) from นักศึกษา |
| 15 | นักเรียน | นักวิชาการ | **CUT** | "academic/scholar" ≠ pupil — clearly wrong |
| 16 | นักเรียน | นักศึกษา | **CUT** | level confusion: school pupil ≠ university student |
| 17 | นักเรียน | นิสิต | **CUT** | นิสิต = university student (tertiary), not นักเรียน |
| 18 | นักเรียน | นิสิตนักศึกษา | **CUT** | tertiary student, not นักเรียน |
| 19 | นักเรียน | ผู้ศึกษา | KEEP | = learner, genuine |
| 20 | นักเรียน | ผู้เรียน | KEEP | = learner, genuine |
| 21 | นักเรียน | เด็กนักเรียน | KEEP | true syn |
| 22 | เล็ก | น้อย | KEEP | true syn (small/few) |
| 23 | พจนานุกรม | ปทานุกรม | KEEP | true syn |
| 24 | เขียน | ประพันธ์ | KEEP | compose/write |
| 25 | อาจารย์ | ผู้สอน | KEEP | true syn |
| 26 | อาจารย์ | ผู้ให้ความรู้ | KEEP | true syn |
| 27 | ภาษา | ภาษาธรรมชาติ | KEEP | natural-language (a kind of ภาษา) |
| 28 | โรงเรียน | ร.ร. | KEEP | abbreviation |
| 29 | เขียน | รจนา | KEEP | compose (literary) |
| 30 | โรงเรียน | รร. | KEEP | abbreviation |
| 31 | หนังสือ | สมุด | KEEP | book/notebook — loosely acceptable |
| 32 | สัตว์ | สัตว์ป่า | KEEP | animal/wild-animal (hyponym, acceptable) |
| 33 | สัตว์ | สัตว์เดียรัจฉาน | KEEP | true syn |
| 34 | สัตว์ | สิ่งมีชีวิต | KEEP | loosely (animal/living-thing) |
| 35 | สัตว์ | เดียรัจฉาน | KEEP | true syn |
| 36 | สุนัข | หมา | KEEP | true syn (also a seed relation) |
| 37 | สุนัข | หมาบ้าน | KEEP | domestic dog |
| 38 | หนังสือ | หนังสือหนังหา | KEEP | colloquial reduplication |
| 39 | หนังสือ | เล่ม | KEEP | book/volume — loosely acceptable |
| 40 | หมา | หมาบ้าน | KEEP | domestic dog |
| 41 | ใหญ่ | หลัก | **CUT** | "big" ≠ "main/principal" — different senses, not synonyms |
| 42 | อาจารย์ | อ. | KEEP | abbreviation |
| 43 | โรงเรียน | อาคารเรียน | KEEP | school/school-building — loosely acceptable |
| 44 | เขียน | เขียนหนังสือ | KEEP | true syn |
| 45 | เขียน | แต่ง | KEEP | compose/write |
| 46 | เสือ | เสือโคร่ง | KEEP | tiger/Bengal tiger (hyponym, acceptable) |
| 47 | แมว | แมวบ้าน | KEEP | cat/domestic cat |

**Cut: 7 of 47 pairs** (14.9%) — indices 5, 14, 15, 16, 17, 18, 41.

## Repeated patterns observed (noted, not over-engineered into a general filter)
- **นักเรียน ⟷ tertiary-student terms** (นศ./นักศึกษา/นิสิต/นิสิตนักศึกษา, and นักวิชาการ): 5 of the 7 cuts.
  Thai WordNet does not preserve the นักเรียน (secondary) vs นักศึกษา (tertiary) distinction that ORST
  keeps. This is the dominant error cluster for the seed set.
- The other 2 cuts (ครู/ผู้สาธิตวิธีการ, ใหญ่/หลัก) are one-off cross-lingual sense-mapping artifacts.
- No "day-of-week / planet" (ครู→อังคาร) artifact appears among these 47 — that pair connects via a
  *different* synset that doesn't directly pair ครู with a seed word at the pair level here; it surfaces
  only through multi-hop PPR, not as a direct seed-touching synonym pair, so it's out of this audit's
  scope (and already carries a `wordnet` provenance badge).

Deliberately did **not** build a general POS/level classifier — the scope is these 47 pairs; a targeted
suppression list is the right-sized fix.

---

## Addendum — 2026-09-13 (Task 13): the `อ.` ambiguous-abbreviation bridge (`ครู` → `วันอังคาร`)

The original audit (above) correctly found no *direct* wrong `ครู`→`วันอังคาร` pair and noted the
Tuesday/Mars result as an out-of-scope multi-hop artifact. Follow-up live tracing pinned the exact cause
and it's now fixed here (so the audit file stays the authoritative record — not "out of scope" for it).

**Root cause (confirmed live):** `อ.` is a genuinely ambiguous Thai abbreviation — it abbreviates *both*
`อาจารย์`/`ครู` (teacher) *and* `อังคาร` (Tuesday). Thai WordNet lists `อ.` in two unrelated synsets:
- teacher sense: `ครู / ครูบาอาจารย์ / ผู้สอน / ผู้ให้ความรู้ / อ. / อาจารย์`
- calendar sense: `วันอังคาร / อ.  / อังคาร` (TSV line ~13536)

Our loader has no word-sense layer, so `อ.` is a single graph node bridging both. PPR from `ครู` crossed
`ครู → อ. → วันอังคาร` / `→ อังคาร`. `ครู`↔`อ.` itself is correct (audit #7 KEEP).

**Fix (single targeted cut, same discipline as the main audit — NOT a general disambiguator):** in
`wordnet.rs`, `SUPPRESSED_AMBIGUOUS_MEMBERS = [("อ.", ["วันอังคาร","อังคาร"])]` — remove `อ.` from any
group that also contains a calendar marker, i.e. the calendar-sense group only. `อ.` stays fully intact in
the teacher-sense group. Removing `อ.` leaves the calendar group as `วันอังคาร / อังคาร` (still valid), so
looking up `วันอังคาร` directly still works.

**Verified live (2026-09-13):** `ครู` → อาจารย์, ครูบาอาจารย์, ผู้สอน, ผู้ให้ความรู้, **อ.**, โรงเรียน,
นักเรียน, การศึกษา — **no วันอังคาร/อังคาร**, `อ.` retained. `วันอังคาร` direct → `อังคาร` (intact).
2 regression tests added (`ambiguous_abbrev_pruned_from_calendar_group_only`,
`kru_has_no_calendar_bridge_via_embedded_data`). 54 tests pass.
