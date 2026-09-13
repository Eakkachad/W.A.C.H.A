# Round 5B — Finish R5 Unattended

**Written 2026-09-13 by the reviewing agent.** Supersedes the unfinished tasks in `NEXT_STEPS_R5.md`
(Tasks 1, 2, 3, 5, 7 are **done and verified**; Task 4 is **done but rejected on review** — see A1).

**You will run this whole document unattended, with no human checkpoint.** That changes the rules:

- **Never silently substitute an approach.** If something specified here does not work, do the documented
  fallback; if there is no fallback, **STOP that task, leave the previous commit intact, and write the
  reason into `VERIFY_R5.md`.** Do not improvise a replacement. *(This rule exists because Task 4
  silently replaced Personalized PageRank with edge counting — see A1. That is the exact failure mode to
  avoid.)*
- **Commit per task.** Every commit must leave `wacha-web --data ../data` serving working lookups. Never
  leave a broken tree.
- **One task's failure must not block the others.** Phases are ordered so that stopping at any point
  still leaves a demoable product.
- **Everything you claim must come from a command that is in `scripts/verify_r5.sh`** (Task A3). If a
  number is not printed by that script, do not put it in a document.

Standing guardrails from `NEXT_STEPS_R5.md` §2 all still apply — especially **do not mass-scrape ORST**
and **do not touch the `katgpt-rs` repo**.

---

# PHASE A — Fix Task 4 (blocking; do first, in order)

## A1 — Restore Personalized PageRank on the sense-scoped graph

**What is wrong.** Task 4 (`fa246bb`) achieved its structural goal — sense-scoped edges work, `ครอบครัว`
no longer returns `บ้านเกิด`, provenance labels are correct — but it **replaced the ranking algorithm**.
`relations.rs:304` now reads `score: count as f32`, and `grep personalized_pagerank src/relations.rs`
returns nothing. `graph.rs:267` and `graph.rs:342` are now dead code.

Why that is not acceptable: `PITCH.md:33` describes the architecture as "Personalized PageRank + BFS",
and `PITCH.md:81` answers the judges' "AI อยู่ตรงไหน" question with "graph reasoning (PageRank/BFS)".
Commit `c733acc` corrected the FolkRank citation for a formula that is no longer executed. The product
must do what the pitch says it does.

Observable symptom: scores are now integers (1.000 / 2.000 / 3.000 / 4.000) instead of continuous values
(previously 9.129, 8.912, 6.890). `บ้าน` returns `กระท่อม, กว้าน, คฤห, คฤหัสถ์, คฤหา, คฤหาสน์` all tied
at 2.000 and ordered alphabetically, so archaic forms crowd out common words.

**Important hypothesis to check first.** Adding Sense nodes grew the graph substantially. It is plausible
Task 4 switched to counting because full-graph power iteration became too slow per query. **Measure
before you assume.** If that is the cause, the sanctioned fix is bounded local PPR (below), not counting.

**What to do.**

1. Report current graph size (entities, triples) and the shape after sense nodes were added.
2. Restore the documented ranking, running on the **sense-scoped** edge set:
   - **Global PageRank `π`** — query-independent, so compute it **once at engine build time** and cache
     it in the engine struct. Do not recompute per query.
   - **Personalized PageRank `π_q`** — teleport vector concentrated on the query Word node.
   - **Score** = `log(π_q(e)) − log(π(e))`, the FolkRank-variant already documented in `graph.rs` and
     `BIBLE.md` §6.4. Keep the correction; it is the thing the citation now correctly describes.
3. **Performance requirement: p95 lookup latency ≤ 200 ms warm.** If full-graph `π_q` exceeds that,
   use **bounded local PPR** — restrict the power iteration to the BFS neighbourhood within
   `max_hops` of the query node (the subgraph you already compute for explanations), rather than
   iterating over all entities. This is a standard, legitimate technique. **If you use it, document it**
   in `BIBLE.md` §6.4 as a bounded approximation, with the measured latency. Do not present an
   approximation as the exact computation.
4. **Ranking must not be purely structural.** Restore word-frequency (`tnc_freq.txt`) as a **documented
   tiebreaker** for near-equal scores, so `คฤห`/`คฤหา` cannot outrank common words. State the rule in
   code comments and `BIBLE.md`: PPR score first, frequency only to break ties.

**Acceptance criteria** (all must appear in `VERIFY_R5.md`):
- `grep -rn personalized_pagerank src/relations.rs` returns at least one **caller**. Paste it.
- A test asserting the ranking is **not** count-based: find two results for one query that have the same
  number of connecting edges but different scores, and assert `score_a != score_b`. Name it
  `ranking_is_not_edge_count`.
- A test asserting `graph.rs`'s PPR is reachable from `Engine::lookup` (e.g. via a counter or a direct
  call-path test). Name it `lookup_uses_personalized_pagerank`.
- `lookup บ้าน`, `lookup ครู`, `lookup ครอบครัว` pasted, showing **continuous** scores.
- `ครอบครัว` still does **not** return `บ้านเกิด` (the Task 4 guarantee must survive).
- Measured p95 warm lookup latency over the 20 seed words + 20 random words. Report the number.

**STOP condition:** if you cannot restore PPR within the latency budget even with bounded local PPR, do
**not** ship counting. Stop, keep Task 4's commit, and write the measurements and what you tried into
`VERIFY_R5.md`. A documented blocker is a good outcome; a silent substitution is not.

---

## A2 — Measure recall, not just precision

**What is wrong.** Task 4 reports "false 2-hop links 150,018 → 0". That is a one-sided metric. Cutting
cross-sense paths also removes some **true** relations that only existed through those paths.

Confirmed example: `รถยนต์ → ยานยนต์` is gone. `ยานยนต์`+`รถ` share synset `03791235-n`; `รถยนต์`+`รถ`
share a different one, so the 2-hop path is correctly cut — but `ยานยนต์`/`รถยนต์` genuinely are close
in Thai. Precision went up; recall went down. This project does not report one without the other.

**What to do.**
1. Build a gold set from work already done: the **47 audited pairs** from the Round 4 audit
   (`wacha/data/seed_wordnet_audit_2026-09-13.md`), which are already marked KEEP/CUT by hand.
2. Measure against the current build: of the pairs marked **KEEP**, how many does the system still
   return? Of those marked **CUT**, how many are correctly absent?
3. Report as a pair: **recall on KEEP pairs** and **precision proxy on CUT pairs**, before and after
   Task 4 (use `git stash`/a checkout of `c733acc` to get the "before" numbers, then return to HEAD).
4. Add the false-pair count as a **test that recounts from the live graph and asserts 0**, rather than a
   number quoted in prose. Name it `no_cross_sense_two_hop_pairs`. This is what makes the claim
   checkable in one command instead of requiring a reviewer to re-derive it.

**Acceptance criteria:** a small table in `VERIFY_R5.md` — KEEP-recall and CUT-absence, before vs after,
with the actual counts. If recall dropped materially, say so plainly; that is a real finding, not a
failure.

---

## A3 — `scripts/verify_r5.sh` (do this before Phase B)

A single script that prints **every** number any R5 document claims, so review is one command.

Must print, each clearly labelled:
1. `cargo test --release` summary line (test counts).
2. Graph size: entities, triples, sense nodes.
3. Definition coverage: raw (`% of words_th.txt`), union (`% of union vocab`), and
   **frequency-weighted at top-100 / top-1000 / top-5000** using `tnc_freq.txt`. All three denominators
   must be printed explicitly next to each number.
4. Cross-sense 2-hop false-pair count (expect 0).
5. KEEP-recall / CUT-absence from A2.
6. p95 warm lookup latency.
7. Cold vs warm engine start time.
8. `lookup` output for: `ครู`, `บ้าน`, `ครอบครัว`, `รถยนต์`, `ปัญญาประดิษฐ์`, `สนาม`.
9. Confirmation lines: `git -C ../katgpt-rs status --short` is empty; `grep -c personalized_pagerank
   src/relations.rs` ≥ 1.

The script must be runnable from `wacha/` and must not require network access.

---

# PHASE B — New capability (independent; skip on failure, do not block Phase C)

## B1 — Task 6: ศัพท์บัญญัติ demo subset

**Re-read `NEXT_STEPS_R5.md` §2.1 first. Demo subset only.** Hard limits for this unattended run:
**≤ 300 terms total, ≥ 500 ms between requests, single-threaded, stop on the first HTTP error or any
non-200 response, and never re-fetch a term already cached** in `data/coined_word_cache/`.

If the site is unreachable or returns errors, **skip B1 entirely**, note it in `VERIFY_R5.md`, and move
to B2. Do not retry in a loop. Do not raise the limits.

Implementation details (endpoints, record structure, the 40-discipline list, the missing-ธรณีวิทยา trap)
are in `NEXT_STEPS_R5.md` §1.2 — follow them exactly.

Emit one `Sense` per (Thai term, discipline), with `subject` set, and
`Provenance { source: CoinedWord, license: OrstEducational, confidence: Confirmed }`.
**Add `RelationSource::CoinedWord`** with display tag `"ศัพท์บัญญัติ (ราชบัณฑิตยสภา)"` — and while you
are there, confirm `classify_source` reads from `Provenance.source` rather than inferring from graph
shape. (Inference is what caused the two provenance bugs already fixed this round; a third source must
not reintroduce it.)

**Acceptance:** `lookup สนาม` shows discipline-tagged equivalents; a view for `field` reproducing the
`NEXT_STEPS_R5.md` §1.2 table; term count fetched; confirmation that a second run makes zero network
requests.

## B2 — Task 8: RID importer stub + `COMPETITION_DAY.md`

Per `NEXT_STEPS_R5.md` Task 8. Build against **hand-copied fixtures** in `wacha/tests/fixtures/rid/`
(5–10 entries — this is fixture use for testing, not harvesting). Cover multi-sense entries, Thai-numeral
homographs, `[POS]` markers, `(สาขาวิชา)` tags, `(ป. …; ส. …)` etymology, ลูกคำ, and `ดู` cross-refs.

Then write `COMPETITION_DAY.md`: where to drop the organizer's file, which command to run, how to
regenerate the trie cache, how to spot-check 5 words, and which adapter function to edit if their format
differs.

**Acceptance:** fixture-driven tests pass; you execute `COMPETITION_DAY.md` end-to-end against the
fixtures and record the actual elapsed time. Target < 10 minutes; if over 20, simplify the runbook and
say so.

---

# PHASE C — Make the documents true (do last; needs final numbers)

## C1 — Task 9: UI + API licence accuracy

Per `NEXT_STEPS_R5.md` Task 9. Show per sense: POS, สาขาวิชา, register, ลักษณนาม, examples, and a
source + licence badge. Preserve existing HTML escaping on every user-input insertion point — re-run the
Round 3 adversarial battery (11 cases) and confirm all pass.

`wacha/API.md`: per-field licence table, and **remove any implication that RID content is or will be
open data** (`dictionary.orst.go.th` is educational/non-commercial, copyright ORST + NECTEC). State that
the combined dataset is CC BY-SA because of the Kaikki layer.

## C2 — Task 10: re-measure and fix the pitch

Every number in `PITCH.md`, `PITCH_DECK.md`, `BIBLE.md` §8 and `PROGRESS.md` is stale. Update them **from
`scripts/verify_r5.sh` output only**.

Three specific corrections that are not just number swaps:

1. **`PITCH.md` demo word 3 is dead.** Lines 118–119 script `รถยนต์ → ยานยนต์ [unverified ⚠]`, which no
   longer exists (A2). Replace that beat. The stronger replacement is the sense-scoping story itself:
   *"เราพบว่าระบบเราเองสร้างลิงก์ข้ามความหมายผิด N คู่ แล้วแก้ที่โครงสร้างข้อมูล ไม่ใช่ไล่แก้ทีละคำ"* —
   pick a live word that still shows a ⚠ flag and verify it on the current build before scripting it.
2. **Coverage must be stated with its denominator.** Report raw / union / frequency-weighted together.
   Do not quote the frequency-weighted number alone — that would be the same overclaiming this project
   exists to avoid.
3. **`BIBLE.md` §6.4** must describe the ranking that actually runs after A1, including the bounded-PPR
   approximation if one was used.

Then re-verify `PITCH.md`'s demo word list by running every word in it and pasting the output.

---

# PHASE D — Optional, only if A–C are all complete

## D1 — Task 11: segmentation accuracy

Per `NEXT_STEPS_R5.md` Task 11. Switch greedy → maximal matching, evaluate on `pythainlp/wisesight1000`
(CC0, ~74 kB, char-level `is_beginning`), report boundary-F1 as per-sample mean±std.

**⚠️ Do not quote newmm's 0.73 TNHC figure as ours** — that is maximal matching's number, not greedy
longest-match's. Measure our own or report nothing.

If you change the segmenter, the trie cache invalidation from Task 5 must trigger — verify it does, and
re-check the demo sentences in `PITCH.md` for regressions before committing.

---

# Final step — write `VERIFY_R5.md`

A single report at the repo root containing, in this order:

1. **Status table** — every task in this document: done / skipped / stopped, with the commit hash.
2. **Full `scripts/verify_r5.sh` output**, pasted raw.
3. **Deviations** — anything you did differently from this plan, and why. Be explicit; a documented
   deviation is fine, an undocumented one is the problem.
4. **Stopped/skipped tasks** — what blocked them and what you tried.
5. **Known-stale claims** — anything in any document you believe may no longer be true but did not have
   the scope to fix.

Also append the usual dated entry to `PROGRESS.md` and update its status board.

---

## Order of execution

**A1 → A2 → A3 → B1 → B2 → C1 → C2 → (D1 only if time).**

A1 is blocking: do not start Phase B until the ranking is restored or formally stopped with measurements.
B1 may be skipped on any network failure. C2 must run last because it consumes the final numbers.

**If you have to choose:** A1, A2, A3, C2 matter more than B1 and D1. A product whose pitch accurately
describes it beats a product with one more data source and a pitch that does not.
