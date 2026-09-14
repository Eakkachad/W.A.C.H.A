# VERIFY_R11.md — Round 11 verification report

**Run:** unattended, 2026-09-14 evening. Reframe round: the product becomes **"วันนี้อยากให้ภาษาไทย
ทำอะไรให้คุณดี"** — a job-menu front door over the one existing engine, plus two genuinely new
capabilities (deterministic sound-symbolism, pretrained semantic vectors). Standing rules from R5B–R10,
followed exactly: one commit per phase, every number from something actually run, no silent substitution
(STOP + document), **never touch `katgpt-rs`** (re-confirmed unusable this round), `cargo test` green after
every phase, licence-gate before embedding external data. Phase order ROUTER → NAME → SOUND → WRITE → VEC →
BRIDGE → P; must-do floor ROUTER + NAME.

**Headline:** the must-do floor landed plus all three should-do phases (SOUND, WRITE, VEC); BRIDGE was cut
honestly (its source data is unavailable and uncorroborable). The engine is unchanged in spirit — modelless,
deterministic, explainable — now presented as five task doors and enriched by MIT-licensed pretrained
vectors used locally.

---

## 1. Status table

| Phase | State | Commit | Note |
|---|---|---|---|
| **ROUTER** job-menu front door | ✅ done | `52c56f0` | 5 task cards + client-side mode; presentation-only — `/api/lookup` + `/api/reverse` **byte-identical** before/after (proven). |
| **NAME** naming assistant (flagship) | ✅ done | `bcbd26b` | Direct (lookup) + context-clue (reverse) paths; curated demo list; native-Thai names render complete. |
| **SOUND** sound-symbolism badge | ✅ done | `c3ffd18` | Deterministic hard↔soft score; **hedge in the UI every time**; tone component dropped (documented). |
| **WRITE** register filter + rhyme | ✅ done | `e93adbc` | Fixed a real `{register}` parse bug (1,212 tags were dropped); loose rhyme index 38,550 words / 234 keys / 35 ms. |
| **VEC** pretrained thai2fit vectors | ✅ done (GATED) | `04d95f4` | MIT licence confirmed **before** download; 45.9% vocab overlap; NN sanity passes; local-only, gated from public deploy. See §VEC. |
| **BRIDGE** English-cognate bonus | ⏹️ CUT | — | Droppable/cut-first. Source repo unavailable **and** Kaikki can't corroborate the PIE→English chain → cutting beats shipping unverifiable claims. See §BRIDGE. |
| **P** pitch pass | ✅ done | `a56e599` | Job-menu reframe, naming market gap, sound hedge intact, honest katgpt line; verify_pitch re-run PASS. |
| Deliverables | ✅ done | `<this>` | This file + `BENCHMARKS.md` §4.5/4.6/4.7 + `PROGRESS.md` + `MENTOR_BRIEF.md`. |

**Final certification (measured this session):** 114 lib + 4 poc + 1 alloc tests pass, 0 warnings;
`verify_pitch.sh` ALL PASS; `verify_r5.sh` §8 pitch + §9 WASM smoke + §10 rank guard (51.3%) all pass; audit
cross_sense=0 / KEEP 40/40 / CUT 7/7; `katgpt-rs` untouched; `README.md`/`.gitignore` not touched.

---

## ROUTER — job-menu front door (presentation-only)

Reframes the product from "search a word" to **"วันนี้อยากให้ภาษาไทยทำอะไรให้คุณดี"**: 5 cards
(ตั้งชื่อ / หาคำให้ใช่กับสิ่งที่กำลังเขียน / เข้าใจศัพท์เฉพาะทาง / สำรวจรากคำ-วิวัฒนาการ / ตรวจคำทับศัพท์),
each a client-side `mode` that changes the placeholder, adds a mode note, and reorders the SAME rendered
profile sections (a DOM reorder over `data-sec` keys — no section's markup or wiring changes). A
"ค้นหาทั่วไป" escape hatch + "เมนูหลัก" back button stay reachable from every mode; `?mode=` deep-links a door.

**Acceptance proven:** `/api/lookup?q=กระดาษ` and `/api/reverse?q=เข้มแข็ง` are **byte-identical** before/after
(`cmp -s` YES) — the phase changed presentation, not data. All 5 modes + `setMode`/`showMenu`/`reorderForMode`
+ all 7 `data-sec` section tags present in the served HTML. cargo test green.

## NAME — naming assistant (the flagship; must-do floor)

Built inside ROUTER's naming mode, reusing the existing engine (no new backend). `namingRelabel()` rewrites
the shared section headers to naming copy ("ความหมายของชื่อนี้", "รากศัพท์ของชื่อ",
"คำใกล้เคียงที่อาจเป็นทางเลือกอื่น"). Two paths:
- **Direct** (`lookup`): verified end-to-end via real CLI on ≥5 names — ดารา (ดาว·สันสกฤต ตารา), วารี (น้ำ),
  อรุณ (รุ่งอรุณ), ชัย (การชนะ·RID), มณี (แก้วมีค่า), กมล (บัว·RID). **Native-Thai (Kra-Dai) names render
  complete, not degraded** — ฟ้า/ดาว show a real "สืบทอดจากไทดั้งเดิม" root, which is the majority case.
- **Context-clue** (`reverse`): a **curated, pre-verified** demo list (not improvised live) — ความกล้าหาญ →
  วีรบุรุษ/วีรชน, แสงในตอนเช้า → อรุณ/อรุณสวัสดิ์, ความรู้สึกเสียใจ → ความเสียใจ, น้ำที่ตกลงมาจากฟ้า → ฝน,
  เครื่องดนตรีที่มีสาย → พิณ/อูกูเลเล.

**Market gap (cited, MENTOR_BRIEF §4):** every existing Thai naming tool (Myhora, Mongkolname, Thaibabyname,
ชื่อดี AI) is numerology/astrology-based; **none grounds a name in real etymology** — a genuine, defensible
differentiator.

## SOUND — deterministic sound-symbolism badge

New `wacha/src/sound.rs`: a pure, no-ML function mapping a Thai respelling to a hard↔soft axis (5 buckets +
a "why"). Heuristic: +2 plosive, +1 onset cluster, +1 short / −1 long vowel, +1 front-unround / −1 back-round
vowel, −2 nasal/liquid/glide. **Deviation (documented):** the tone component from the plan was **dropped** —
it was flagged as the weakest-evidence signal, and the Thai respelling strings don't cleanly encode tone, so
including it would be guesswork.

**Mandatory hedge in the UI** (not just this doc), shown every time the badge appears, verbatim:
"รูปแบบจากงานวิจัยเรื่องสัทสัญลักษณ์ทั่วไป ยังไม่ใช่ข้อพิสูจน์เฉพาะภาษาไทย" — a unit test pins the exact
string, the JSON carries it on every response, and the render prints it every time.

**Acceptance:** 4 unit tests incl two contrastive pairs (กะตุก/ตึก score harder than มาลี/นอน). Live scoring
discriminates sensibly: ตุ๊กตา 5/5 (แข็ง) > ดารา 2/5 > มาลี 1/5 (นุ่ม).

## WRITE — register filter + loose rhyme finder

**WRITE-1 register filter** — fixed a real data bug first: `Register` (แบบ/โบ/ปาก/ราชา/เลิก) was parsed
per-sense, but the R10 reshape emitted it in `{braces}` while the importer only read `(parens)`, so **1,212
{โบ} tags (and all others) were silently dropped**. Parser now reads the brace form. `Engine::register_search`
(headwords by register, freq-ranked, deduped) + `word_has_register`; CLI `register` + `/api/register` (composes
with the reverse-dictionary candidate set when a topic query is given). Verified: register ราชา → ผม, คุณ,
ทราบ, ดิฉัน, เท้า, สตรี, โค (genuine ราชาศัพท์).

**WRITE-2 loose rhyme finder** — new `wacha/src/rhyme.rs`: keys each headword by its final rime
(final-consonant มาตราตัวสะกด class + vowel nucleus + long/short) from the RID respelling. **Loose by design,
NOT classical เอก/โท meter** (the scope trap this round flagged). 38,550 words / 234 keys, built in **35 ms**
at web startup (BENCHMARKS §4.5). Verified real rhymes, not key collisions: บ้าน → การ/งาน/ด้าน/ท่าน/อาหาร/
ผ่าน/รัฐบาล; กด → กฎ/กรด. 4 unit tests.

## VEC — pretrained semantic vectors (thai2fit_wv), the licence-gated risk phase

**Licence gate FIRST (before any download).** Confirmed the `cstorm125/thai2fit` repo LICENSE = **MIT**
(the `thai2vec.bin` 51,556×300 word2vec embeddings, v0.1). The vectors were trained on CC BY-SA Thai
Wikipedia, but the author's explicit **MIT release governs the artifact** — permissive and redistributable.
Still, out of the same caution applied to RID / ศัพท์บัญญัติ (R9 D1 / R10 R3), the vectors are used in the
**local/judge build only and gated out of any public deploy** until an explicit confirmation. **We trained
nothing** — this is integration.

| item | value |
|---|---|
| licence | **MIT** (thai2fit repo), trained on CC BY-SA thwiki — artifact governed by author's MIT release |
| download | `pythainlp.word_vector.WordVector("thai2fit_wv").get_model()` — one-time offline (gensim + pythainlp) |
| thai2fit vocab | 51,358 × 300-dim |
| **overlap with wacha vocab (measured)** | **28,589 = 45.9% of wacha, 55.7% of thai2fit** (RID rare/archaic gap is real, as predicted) |
| blob | `data/thai2fit_overlap.vec.blob` ~35 MB — **gitignored/untracked, regenerable** via `scripts/gen_thai2fit_overlap.py` |
| NN search | flat array + cosine, linear scan — **~3.9 ms** avg (no ANN/octree index, a deliberate rejection) |

**NN sanity (measured, 5 pairs):** related ≫ unrelated — หมา~แมว 0.52, ครู~อาจารย์ 0.45, แม่~พ่อ 0.65,
กิน~ดื่ม 0.29, รถ~เรือ 0.36 vs หมา~คอมพิวเตอร์ 0.07, ครู~ก้อนหิน 0.05. **Agrees with the hand-verified relation
graph — no contradiction, safe to ship.** **Reverse recall (measured):** for ความกล้าหาญ, BM25 returns 6
definitional hits; the vector source *adds* 4 semantically-adjacent candidates (ความบริสุทธิ์ 0.74,
ความจงรักภักดี 0.73, ความยิ่งใหญ่ 0.72, ความซื่อสัตย์ 0.71), clearly labeled `source:"vector"` and composed
with (never replacing) BM25.

## BRIDGE — English-cognate bonus layer: CUT (documented)

`NEXT_STEPS_R11` marks BRIDGE droppable / "cut first if short on time," and its own §0.4 warns its 43-entry
seed carries **zero source citations**. Two independent blockers made a clean, honest build impossible:
1. **The seed is unavailable.** The external `ThaiDict_Script` / `seed_corpus.py` (43 Sanskrit/Pali → PIE →
   English cognate entries) is not present anywhere locally and I have no confirmed URL to fetch it.
2. **The designated corroborating source can't corroborate.** The plan says to cross-verify each entry
   against `kaikki_th.jsonl`'s Wiktionary etymology — but that data stops at the immediate borrowing:
   for มารดา, Kaikki's `etymology_texts` is "ยืมมาจากบาลี มาตา" (Pali) with **no** mother/maternal/matrix
   PIE→English chain. So even if I had the seed, I could not independently verify its cognate claims.

With neither the data nor a citation source, shipping BRIDGE would mean surfacing exactly the unverifiable
claims the plan warned against. Per the standing rule and BRIDGE being explicitly droppable, it is **cut and
recorded here** rather than pushed through. No code was written; the `Etymology` model is unchanged (no
`english_cognates` field added).

## P — pitch pass (and the R6/R7 silent-break guard, honored)

Applied to `PITCH_DECK.md` / `PITCH.md`: reframed slide 2 around the job-menu front door + the 5 doors/one
engine framing + the R11 dimensions; added the naming assistant as a flagship Innovation beat with the cited
market gap; added the sound-symbolism feature **with its mandatory hedge intact and a "ห้ามตัดออก" note**;
strengthened the katgpt-rs Q6 with the fresh-audit facts (Kimi-K3 random-init gibberish, ~17.7 GB, 28
LLM-serving crates with nothing to serve) framed as engineering discipline. `BIBLE` §2.2 honesty note kept.

**Item 4 (the guard):** re-ran AFTER all edits — `verify_pitch.sh` ALL PASS, `verify_r5.sh` §8 pitch + §9
WASM smoke ALL PASS, §10 rank guard 51.3% OK. No silent R6/R7-style break.
