# VERIFY_R9.md — Round 9 verification report

**Run:** unattended, 2026-09-14. Rules from R5B–R9: commit per task, every number from a script, no silent
substitution (STOP + document if a task can't be done cleanly), do not touch `katgpt-rs`, finish fewer
tasks cleanly over many half-done. Doc-staleness policy: leave it for the final pass; fix only statements
that are actually *false*.

---

## 1. Status table

| Task | State | Commit | Note |
|---|---|---|---|
| **G1** ranking-regression guard | ✅ done | `517ce0c` | `wacha rankguard` p@5 over KEEP gold set; verify_r5.sh §10, threshold 50%. Failure path demonstrated. |
| **G2** deadline on `symbol_trie.rs` | ✅ enforced | `052a786` | Q3 failed its gate → `symbol_trie.rs` deleted this round; findings kept in BENCHMARKS §3.3. |
| **D1** WASM licence audit | ✅ done (STOP) | `6394a37` | **ORST ศัพท์บัญญัติ content IS embedded** (113 defs) → no public deploy. See §D1. |
| **D2** phone deploy + test | ⚠️ constrained | `9a870b3` | D1 forces **tailnet-only**; servable + PWA-ready measured; physical phone test needs a human (recorded). |
| **Q1** maximal matching | ✅ done | `3d09cc8` | WL-F1 0.6809 (maximal) > 0.6611 (greedy) → shipped; still below newmm 0.74. verify_pitch + WASM re-verified. |
| **Q2** reverse dictionary v2 | ✅ done | `d46ea4b` | enrich + coverage damping; 10 queries un-curated (3 clear hits, 2 structural failures persist). |
| **Q3** S1b final attempt | ⏹️ STOPPED→DELETED | `052a786` | 7.02× build but differential fails (26); deleted per G2. |
| **X1** Wikidata Lexemes | ✅ done (thin) | (this) | **29 Thai lexemes / 46 senses** — negligible; not integrated (stop rule). See §X1. |
| Deliverables | ✅ done | (this) | `VERIFY_R9.md` + `BENCHMARKS.md` (per-task) + `PROGRESS.md`. |

---

## D1 — Licence audit of the WASM artifact (the answer)

**Question:** is ORST ศัพท์บัญญัติ content (no redistribution licence stated on the source site — only
"educational, non-commercial") compiled into `wacha_wasm.wasm`, making a public deploy a redistribution
of it?

**Method (all reproducible):**
- Traced the three generators that produce the embedded blobs:
  - `examples/gen_defs_blob.rs` → `assets/defs.blob`: built from **`Engine::load_from_dir`** — the
    **full** engine, which runs `CoinedWordImporter` over `data/coined_word_cache/`. **Includes ศัพท์บัญญัติ.**
  - `examples/gen_reverse_index.rs` → `assets/reverse.idx`: also from **`Engine::load_from_dir`** (full
    engine). Indexes every definition, **including the ศัพท์บัญญัติ ones.**
  - `examples/gen_pagerank_blob.rs` → `assets/pagerank.blob`: built from **seed entries only**
    (`build_from_segmenter`, no `coined_word_cache`). **No ศัพท์บัญญัติ.**
- Counted directly in the shipped blob:
  ```
  defs.blob: 29,601 entries, 113 with src_code=2 (CoinedWord/ศัพท์บัญญัติ)
  ```
- Confirmed a sample is ORST-labeled: `lookup ขั้นตอนวิธี` →
  `นิยาม: algorithm (ศัพท์บัญญัติ · คอมพิวเตอร์…)` · `ที่มานิยาม: ศัพท์บัญญัติ (ORST) · ORST (educational, non-commercial)`.

**Per-source verdict for a PUBLIC deploy (redistribution):**

| embedded source | in artifact | licence | publicly redistributable? |
|---|---|---|---|
| `words_th.txt` (LEXiTRON) | yes (WORDS_TH + seg cache) | CC0-1.0 | **Yes** |
| Thai WordNet | yes (relations) | NICT permissive | Yes, **with the NICT copyright notice** |
| Kaikki / Wiktionary defs | yes (defs.blob, reverse.idx) | CC BY-SA + GFDL | Yes, **with attribution + share-alike stated** |
| **ศัพท์บัญญัติ (ORST)** | **YES — 113 defs in defs.blob + reverse.idx + relation graph** | **"educational, non-commercial" only; no redistribution licence** | **NO** |

**Verdict: STOP — do not deploy the current artifact publicly.** The ศัพท์บัญญัติ layer is embedded and
its stated terms ("educational, non-commercial") are not a redistribution licence. Options (not executed
this round; the `Importer` seam makes the first a clean build-time choice):
1. **Public variant with the ORST layer excluded** — regenerate `defs.blob`/`reverse.idx` from an engine
   built *without* `coined_word_cache`, drop the CoinedWord relations, and ship that. The Wiktionary layer
   then needs a visible CC BY-SA attribution + the NICT WordNet notice on the page.
2. **Keep the full-data build private/unlisted (tailnet only)** and demo from a device on the tailnet.

This round takes option 2 for D2 (below) — it needs no data changes and keeps the demo intact. Building the
ORST-excluded public variant is a clean follow-up, not attempted here to avoid shipping a half-done deploy.

---

## D2 — Offline build on a device (tailnet-only per D1)

D1 forces **tailnet-only** (no public deploy), so D2 is a private-network test. **A real phone test cannot
be driven by an unattended agent** — it needs a human to open Safari/Chrome on a handset, tap *Add to Home
Screen*, and toggle airplane mode. Per the standing honest-scoping rule I do **not** fabricate phone
numbers; I measured everything measurable without a device and leave the physical test as a pre-event
manual checklist item (which is what the plan already flagged: the phone answer was *"น่าจะได้"*).

**Measured (reproducible), the mobile cold-load determinants:**
- **Artifact transfer size** (what a phone downloads): `wacha_wasm.wasm` **17.26 MB raw / 4.88 MB gzip**
  (+ index.html 3.7 KB gzip, sw.js 0.6 KB, manifest 0.4 KB, icons ~26 KB). **A gzip-capable static host is
  mandatory** — 4.88 MB vs 17.26 MB is the difference between a usable and an unusable mobile first load.
- **Engine init after download** (node wasm32, the compute part a phone repeats): instantiate 3.2 ms +
  `wacha_init` 45.4 ms (WASM smoke test). A phone's slower CPU will be some multiple of this but still
  well under a second — the download dominates, not the compute.
- **Static servability:** served `web/` over HTTP; all assets return 200 with correct paths
  (index.html, manifest.webmanifest, sw.js, icon-192.png, wacha_wasm.wasm).
- **PWA / Add-to-Home-Screen readiness:** `manifest.webmanifest` has `display: standalone`, `start_url: ./`,
  and three icons (192, 512, maskable-512) — all present on disk (`a0c1363` added them). `sw.js` cache-first
  caches the app shell + wasm for offline warm loads. So Add-to-Home-Screen and offline-after-first-load are
  **structurally ready**; only the on-device confirmation is outstanding.
- **Projection (NOT measured on a phone):** at 4.88 MB gzip, cold first load ≈ 4.9 MB ÷ mobile bandwidth —
  ~5 s on a typical 8 Mbps 4G, sub-second on Wi-Fi; second load is service-worker-instant offline. **The
  number quoted on stage must be a real on-device measurement, not this projection and not node's 45 ms.**

**Status: constrained-done.** Everything an unattended agent can verify is green (servable, PWA-ready,
gzip-sized, engine works); the physical phone cold/warm/airplane + Add-to-Home-Screen test is the one
item that requires a human and is recorded as the outstanding pre-event check.


---

## Q1 — Maximal matching (measured, shipped)

Added `SegMode::MaximalMatching` (newmm-style DP, min tokens) alongside greedy; measured both on
wisesight1000 (AttaCut protocol, `examples/seg_wl_f1.rs`):

| mode | word-level F1 (per-sample / micro) | boundary F1 (per-sample / micro) |
|---|---|---|
| greedy | 0.6611 / 0.6343 | 0.8015 / 0.7822 |
| **maximal (shipped)** | **0.6809 / 0.6508** | **0.8195 / 0.7957** |

Maximal wins on every metric → shipped as the default `Segmenter::segment`. **Honest:** the gain is
modest (+~0.02 WL), **not** the gap-closer hypothesised — neither mode reaches newmm's **0.74** WL
(we sit at 0.6508 micro). The residual is our LEXiTRON word list + TCC OOV, not just the algorithm.
Gates met: `verify_pitch.sh` passes; `reverse.idx` regenerated + WASM rebuilt (17.27 MB / 4.88 MB gzip)
+ smoke test passes; rank guard still 51.3%. 95 tests.

## Q2 — Reverse dictionary v2 (enrich + damp), all 10 queries un-curated

**Changes:** (1) enriched each indexed document with the entry's usage examples + related-word headwords
(index grew 18,894 → 28,829 terms); (2) added a coordination/coverage damping factor
(`score × coverage^1.0`, coverage = distinct query terms matched / distinct query terms) so one rare
high-IDF query word can't carry a hit alone.

**All 10 queries (4 reviewer + 6 new), reported un-curated — top result shown, ✓/✗ vs the intended word:**

| # | query | top-5 (after v2) | intended | verdict |
|---|---|---|---|---|
| R1 | ที่เก็บเงินของรัฐ | หัวเบี้ย, ค่าธรรมเนียม, ส่วนลด, เงินตรา, **ภาษี** | คลัง | ✗ (คลัง not top-5; ภาษี/ค่าธรรมเนียม plausible) |
| R2 | ความรู้สึกเสียใจอย่างมาก | **น้ำตาตกใน**, **สะอื้น**, รันทด, โอ๊ย | น้ำตาตกใน/สะอื้น | ✓ |
| R3 | สัตว์เลี้ยงสี่ขาเห่าได้ | การเห่า, จตุร, ๔, **โฮ่ง**, โฮ่ง ๆ | สุนัข/หมา | ✗ (structural — สุนัข gloss lacks เห่า/สี่ขา; damping did drop the noise scores 11→2.7 and surface โฮ่ง) |
| R4 | เครื่องมือสำหรับเขียนหนังสือ | โกรกกราก, ไฮโดรมิเตอร์, กรรไกร, เบ็ด | ปากกา/ดินสอ | ✗ (structural — those glosses don't say เครื่องมือ+เขียน) |
| N1 | ยานพาหนะที่บินได้ | การบิน, บิน, ผู้โดยสาร, รถเมล์ | เครื่องบิน | ✗ (partial — บิน/ยานพาหนะ matched, เครื่องบิน not top-5) |
| N2 | สถานที่รักษาคนป่วย | เล็ดลอด, รักษา, คุ้มครอง, ผู้ป่วย | โรงพยาบาล | ✗ |
| N3 | อาหารเช้าที่ทำจากไข่ | ซีเรียล, กระทงทอง, ตัวอ่อน | ไข่เจียว/ไข่ดาว | ✗ (partial — ซีเรียล is a breakfast food) |
| N4 | ผู้ที่สอนหนังสือ | **สอน**, คู่ชีวิต, นวกะ, ปฏิคม | ครู/อาจารย์ | ✗ (partial — สอน #1, ครู not top-5) |
| N5 | น้ำที่ตกลงมาจากฟ้า | ขวานฟ้า, **ฝน**, ใส, โผล่, หิม | ฝน | ✓ (ฝน #2; หิม=snow #5) |
| N6 | เครื่องดนตรีที่มีสาย | อูกูเลเล, **พิณ**, **เครื่องสาย**, **ไวโอลิน**, สตริง | พิณ/ไวโอลิน | ✓ (excellent — all stringed instruments) |

**Honest score: 3/10 clear hits (R2, N5, N6), several partials, and the 2 known structural failures (R3, R4)
remain.** The enrichment + damping genuinely improved the multi-word conceptual queries (เครื่องดนตรีมีสาย,
น้ำตกจากฟ้า→ฝน) and compressed single-rare-term noise (R3's top score fell 11→2.7). It did **not** fix the
structural cases where the target's gloss simply does not contain the query's descriptive words — that is a
property of terse ORST-style glosses, not a ranking bug, and matches R8's finding. Query set not curated.

## Q3 — S1b third attempt: STOPPED and DELETED (per G2)

Third and final attempt at the dense-alphabet trie. Re-measured on the current vocab:
**byte 59.1 s → symbol 8.4 s = 7.02×** cold build, arrays 16.78 → 8.39 MB — but the byte-identical
differential **still FAILS (26 mismatches, deterministic)**, same base-region invariant class as R8 (a
longer word matches only its shorter prefix because a transition is dropped under dense-id relocation).
The fix remains a base-allocation rework, out of scope. Per the **G2 deadline rule** (three stops is
enough), `symbol_trie.rs` (439 lines) and its two `examples/s1b_*` harnesses were **deleted** this round;
the full three-attempt record + numbers + root cause are preserved in `BENCHMARKS.md` §3.3. Production
byte-keyed `datrie.rs` unchanged. Build has 0 warnings; 93 tests (the 2 spike unit tests went with it).

## X1 — Wikidata Lexemes for Thai: MEASURED THIN → not integrated (stop rule)

Wikidata Lexemes is CC0 and bulk-downloadable (no scraping), so it would be a legitimate independent
source to raise the 84%-Wiktionary / 0.68%-multi-source figures. **But Thai coverage is negligible.**
Measured live via the Wikidata Query Service (`query.wikidata.org/sparql`, language = wd:Q9217, Thai):

| quantity | count |
|---|---|
| Thai lexemes (total) | **29** |
| Thai lexemes with ≥1 sense | **27** |
| Thai senses (total) | **46** |

The lemmas are basic words we already have (หมา, หมู, แมว, กิน, น้ำ, ข้าว, มารดา, …). Against our
**72,175-word** vocabulary and **158,045** relation pairs, 29 lexemes / 46 senses cannot materially move
the multi-source set — it would add a rounding-error number of pairs, all for words already covered by
Wiktionary + WordNet.

**Decision: do NOT integrate** (per the X1 stop rule — a measured negative closes the question). Wikidata's
Thai Lexeme project is simply too young. The 84%-Wiktionary / 0.68%-multi-source concentration remains a
**data** limitation with no clean, licence-safe fix available today: RID and ศัพท์บัญญัติ are settled
no-scrape (NEXT_STEPS_R8 §2), and Wikidata is empty. This is stated honestly in the pitch rather than
papered over. Query service was reachable; the result is thin, not a fetch failure.
