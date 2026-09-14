# VERIFY_R12.md — Round 12 verification report

**Run:** unattended, 2026-09-14 night. Two things: (1) a **deterministic intent-detection layer** so the
product becomes "type anything into one box" instead of "click a card, then type"; (2) reviving the
**PIE/English-cognate BRIDGE** feature R11 cut, now that its source data (`data/thaidict_script_ref/`,
git-ignored) is reachable. Standing rules from R5B–R11, followed exactly: one commit per phase, every
number from something actually run, no silent substitution, **never touch `katgpt-rs`** (settled), intent
stays rule-based + the R11 `thai2fit_wv` fallback (NO new ML/classifier), `cargo test` green after every
phase, BRIDGE-1 is the honesty gate (mirrors R9 D1 / R10 R3 / R11 VEC). Order INTENT-1→2→3→BRIDGE-1→2→3;
must-do floor INTENT.

**Headline:** all six phases landed — nothing cut. The must-do INTENT floor makes the "one box, many jobs"
concept real; BRIDGE cleared its honesty gate at **38/43 (88.4%)** corroborated and shipped at full strength.

---

## 1. Status table

| Phase | State | Commit | Note |
|---|---|---|---|
| **INTENT-1** rule-table classifier | ✅ done | `a54463c` | Pure `classify_intent` (6 intents), keyword rules + reason; 22/22 regression fixture. No ML. |
| **INTENT-2** thai2fit centroid fallback | ✅ done | `634e401` | Reuses R11 vectors; fires only on no-rule. Threshold 0.20 (swept). **6/10** held-out correct — honest long-tail number. |
| **INTENT-3** web route + one-box UI | ✅ done | `a794043` | `/api/intent` + free-text box + "เราคิดว่าคุณอยาก [X] — ใช่ไหม?" line; 5 cards kept as override. |
| **BRIDGE-1** honesty gate | ✅ done | `beb89da` | Cross-verified 43 seed vs Kaikki `etymology_texts`: **38/43 = 88.4%** kept; 4 no-etym + 1 mismatch dropped. |
| **BRIDGE-2** data model | ✅ done | `beb89da` | `Entry.english_cognates: Vec<EnglishCognate>`; TSV attached to 45 headwords. |
| **BRIDGE-3** naming-flow render | ✅ done | `beb89da` | Cognate line (CLI + web) for verified words only; native words render clean. |
| Deliverables | ✅ done | `<this>` | This file + `BENCHMARKS.md` + `PROGRESS.md` + `MENTOR_BRIEF.md`. |

**Final certification (measured this session):** 121 lib + 4 poc + 1 alloc tests pass, 0 warnings;
`verify_pitch.sh` ALL PASS; rank guard 51.3%; audit intact; `katgpt-rs` untouched; `README.md`/`.gitignore`
not touched; data unchanged (76,649 words / 40,681 entries).

---

## INTENT-1 — deterministic rule table (no ML, settled design)

New `wacha/src/intent.rs`: `classify_intent(query) -> IntentGuess { intent, reason, confidence }`. `Intent`
mirrors the 5 web modes + `General`. Layer-1 keyword rules in priority order (Translit-keyword → Naming →
Roots → Specialized → Writing → bare-Latin→Translit → else General), each with a human-readable Thai reason.
No trained classifier — every decision is a literal rule, testable per-rule.

**Verified:** 5 unit tests — per-rule hit + near-miss, Latin→Translit, empty→General, reason populated, and
a **table-driven regression fixture of 22 real-shaped queries (22/22 correct)** checked in so a future rule
change can't silently break a previously-correct classification.

## INTENT-2 — thai2fit_wv centroid fallback (long tail)

Only invoked when INTENT-1 finds no keyword (Confidence::Default). Averages the query's in-vocab word
vectors (R11 thai2fit, no new dependency), cosine-compares against 5 intent seed-word centroids, picks the
nearest above `INTENT_VEC_THRESHOLD`.

**Threshold picked empirically:** a sweep over 0.10–0.30 on 10 hand-written held-out **no-keyword** queries.
The fallback fired on 9/10 and classified **6/10 correct**, stable across 0.10–0.25 (0.30 fires one fewer).
Chose **0.20**. This is an honest ~60–67% on the long tail — exactly the descriptive queries keywords miss;
the misses concentrate where the word2vec centroid over-attracts to Naming/Translit (reported, not hidden).
A wrong vector guess still degrades gracefully (routes to a mode with the confirmation line + one-tap
correction), never worse than today's General. Test: `intent_fallback_inert_without_vectors`.

## INTENT-3 — one free-text box + transparency (cards kept)

`web.rs`: `/api/intent?q=` returns `{query, intent, label, reason, confidence}`. `index.html`: a free-text
box at the top of the job menu (front door) — type anything, no card required. On submit it calls
`/api/intent`, `setMode(guess)`, runs that mode's render path (awaited so it isn't wiped), and prepends a
dismissable **"เราคิดว่าคุณอยาก [X] — ใช่ไหม?"** line (with "เดาจากความหมาย" when the vector fallback fired)
and one-tap links to the other modes. Fetch failure degrades to general search — never a blank/error page.
**The 5 R11 job cards are kept exactly as-is** (verified: 5 `data-mode` cards present) — an explicit override.

**Verified live:** อยากตั้งชื่อลูก→naming (rule), computer→translit (rule), รากศัพท์→roots (rule),
แมว→general (default, still returns its entry — usable, not blank), คำเรียกลูกแบบเพราะๆ→naming (vector 0.38).

## BRIDGE-1 — the honesty gate (do not skip)

`wacha/scripts/verify_cognates.py` parses the now-available `seed_corpus.py` (43 entries, git-ignored
reference) and cross-checks each Thai word against `kaikki_th.jsonl`'s real `etymology_texts` (a LIST field).
An entry is **kept only if** Kaikki independently mentions the same source language (บาลี/สันสกฤต).

**MEASURED: 38/43 = 88.4% corroborated** → well above the 1/3 droppable floor, so the feature ships at full
strength. Dropped, recorded (not silently kept):
- **4, no Kaikki etymology:** สัปต, อัฐ, นวัตกรรม, อัศวะ.
- **1, source-language mismatch:** ตรี — Kaikki says it is borrowed from **Khmer** (เขมร ត្រី), not Sanskrit;
  the gate correctly caught the contradiction and dropped it. This is exactly what the gate is for.

Output: the committed, derived `data/english_cognates.tsv` (38 corroborated rows only). The spot-check words
มารดา and ศูนย์ both pass.

## BRIDGE-2 — data model

`Entry` gains `english_cognates: Vec<EnglishCognate>` where `EnglishCognate { word, pie_root (display-only),
corroboration (the Kaikki text) }`. `load_from_dir_opts` attaches the TSV to matching headwords (45 entry
attachments across homographs). `EntryView` exposes `(word, pie_root)` pairs.

## BRIDGE-3 — honest surfacing in the naming flow

CLI `print_lookup`, web `lookup_json`, and `render()` show a cognate line **only for verified words**, with
an honest note ("เฉพาะคำที่มีรากอินโด-ยูโรเปียน (ส่วนใหญ่ของคำไทยแท้ไม่มี)"). Verified end-to-end:
- `wacha lookup มารดา` → "English cognates via PIE *méh₂tēr): mother, maternal, matriarch, matrix"
- `wacha lookup ศูนย์` → "PIE *ḱewH-): zero, cipher, cave, cavity"
- native **หมา** renders its complete profile with **no cognate line and no visual gap** (the majority case).

Test: `english_cognates_reach_entry_view_and_native_has_none`.

---

## Deviations / notes

- **INTENT-2 seed phrases are single words**, not multi-word phrases, because thai2fit is a word2vec model
  (word-level vectors); multi-word seeds are averaged the same way as the query. Documented here.
- **Nothing cut this round.** BRIDGE's droppable checkpoint (corroboration < 1/3) was not triggered (88.4%).
- The `seed_corpus.py` reference and the thai2fit vector blob remain **git-ignored**; only the corroborated
  derived `english_cognates.tsv` is committed.
