# Round 9 — Close Every Known Weakness Before Feature Work

**Written 2026-09-14 by the reviewing agent.** Same unattended rules as `R5B`–`R8`: never silently
substitute an approach, commit per task, every claimed number comes from a script in the repo, **do not
touch `katgpt-rs`**, and finishing fewer tasks cleanly beats leaving several half-done.

**Documentation policy unchanged:** all docs get one final pass after the system is done. Do **not** fix
staleness here. The exception remains a statement that is actually *false* — fix that immediately.

---

## 0. Health check (measured 2026-09-14 — this is a strong base, not a rescue)

| Signal | Value |
|---|---|
| Build warnings | **0** |
| Tests | **94 + 4**, full suite in **6.3 s** |
| Code size | **8,300 lines**, largest file `relations.rs` 1,487 |
| Safety net | 94 tests + pitch regression + WASM smoke + `verify_r5.sh` §1–9 |

The architecture is ready to extend — `Importer` is a real seam (`rid.rs` 345 lines and
`coined_word.rs` 373 were added later without restructuring anything), and `Entry`/`Sense` already matches
RID's shape so day-of data drops in. This round is about closing gaps, not repairing damage.

---

# PHASE G — Guards (do first; nothing else starts until these are in)

## G1 — Ranking regression guard (the one real hole in the safety net)

`grep -c "patk|precision@5" scripts/verify_r5.sh` → **0**. Everything is guarded except ranking quality —
and ranking is the most fragile thing we have, because it is an empirically-tuned band → PPR → freq
combination whose behaviour shifts whenever the band composition changes (a new source, a new relation
type, more data).

**It has already broken twice, and both times only a human spotted it:** R6 (`คห`, corpus frequency 39,
outranked `เรือน` at 3,469, which fell out of the list entirely) and R7 (tier order promoted the 55%
tier above the 80% tier, contradicting our own audit in the same commit).

**What to do.** Add `wacha patk` to `verify_r5.sh` as **§10** with a hard threshold — fail loudly if
**p@5 drops below 75%** (current: 79.3%, held-out seed `0x52372026`). Print the number every run so a
slow drift is visible, not just a threshold breach.

**Acceptance:** `verify_r5.sh` prints p@5 and fails when it is below threshold; demonstrate the failure
path by temporarily forcing a bad ranking (do not commit that), then restore.

## G2 — Put a deadline on `symbol_trie.rs`

439 lines, compiled via `pub mod symbol_trie`, called by nothing — kept deliberately as the S1b record,
which was right. But dead code rots, and it is now the third-largest module in the crate.

**Decision rule:** Q3 below is the **third and final** attempt. If Q3 does not pass its gates, delete
`symbol_trie.rs` in the same round and keep the findings in `BENCHMARKS.md` §3.3. Do not carry it into a
fourth round.

---

# PHASE D — Deploy and verify on real devices

## D1 — ⚠️ Licence audit of the WASM artifact **before** any public deploy

**Do this before D2, and stop if the answer is unclear.**

The WASM build embeds `WORDS_TH` (CC0), the segmenter cache, `DEFS_BLOB` (29,601 definitions),
`REVERSE_IDX`, `PAGERANK_BLOB`, seed entries and embedded WordNet. Publishing it to a public URL is
**redistribution**, so every embedded byte must be redistributable:

| Source | Licence | Redistributable? |
|---|---|---|
| LEXiTRON `words_th.txt` | CC0-1.0 | Yes |
| Thai WordNet | NICT permissive | Yes, with the copyright notice |
| Kaikki / Wiktionary definitions | CC BY-SA + GFDL | Yes, **with attribution and share-alike stated** |
| **ศัพท์บัญญัติ (ORST)** | **no licence stated on the site** | **Unknown — this is the blocker** |

**What to do.** Determine exactly whether ORST ศัพท์บัญญัติ content is inside `DEFS_BLOB` /
`REVERSE_IDX` / the relation data compiled into the artifact. Then:

- If it is **not** present → proceed to D2, and make sure the page carries CC BY-SA attribution for the
  Wiktionary layer and the NICT notice for WordNet.
- If it **is** present → **do not deploy publicly.** Either build a public variant with the ORST layer
  excluded (the `Importer` seam makes this a build-time choice), or keep the deploy private/unlisted and
  test on a phone over the tailnet instead.

**Acceptance:** a short written answer in `VERIFY_R9.md` naming which sources are in the artifact and the
licence basis for publishing each. This is exactly the kind of question a ราชบัณฑิตยสภา judge could ask,
so the answer is worth having regardless.

## D2 — Deploy the offline build and test on a real phone

Conditional on D1. Static hosting (GitHub Pages or equivalent) — the artifact is pure static files, which
is itself part of the pitch (*"ORST could host this for free, forever"*).

This unblocks the check that is still genuinely untested. The browser check passed on desktop over the
tailnet on 2026-09-14 (definitions ✓, reverse ✓, **offline ✓**), but the phone answer was
*"น่าจะได้"* — i.e. **not tested**. It matters because:
- the artifact is **18 MB**; a tailnet LAN transfer is nothing like a real mobile connection
- iOS Safari handles service workers differently from desktop Chrome
- PWA icons now exist (`a0c1363`), so **Add to Home Screen** can finally be tested — and an installed
  วาจา icon on a judge's home screen is a strong demo beat

**Acceptance:** real numbers from a real phone on a real network — time-to-first-lookup cold, then with
the service worker warm, then in airplane mode; whether Add to Home Screen works and what the icon looks
like. Record them in `BENCHMARKS.md`. **The number quoted on stage must be this one, not node's 48 ms.**

---

# PHASE Q — Close the quality gaps

## Q1 — Maximal matching: close the word-level F1 gap honestly

D2 (R8) measured our word-level F1 at **0.6611**, against newmm's published **0.74** on Wisesight-1000.
We are behind on the like-for-like metric.

**The gap is largely algorithmic and therefore closable:** newmm is **maximal matching** (minimise the
number of words via DP over the trie); we run **greedy longest-match**. Implementing maximal matching over
the existing trie should recover most of that difference, and it stays 100 % deterministic and modelless.

**What to do.** Implement maximal matching as a second segmentation mode, keep greedy available, measure
**both** on wisesight1000 under the AttaCut protocol, and ship whichever wins. Report both numbers either
way.

**Gates:** the `PITCH.md` demo sentences must still segment correctly (`verify_pitch.sh` must pass), and
if the mode changes, the trie cache and WASM artifact must be rebuilt and re-verified.

**Honesty rule, unchanged:** report **our** measured numbers. Even if we reach 0.74, that is our number on
our word list — do not claim to have reproduced newmm.

## Q2 — Reverse dictionary v2

Currently fails on real queries. Verified by the reviewer on four queries never shown to the implementer:
`ที่เก็บเงินของรัฐ` → `คลัง` ✓ and `ความรู้สึกเสียใจอย่างมาก` → `น้ำตาตกใน`/`สะอื้น` ✓, but
`สัตว์เลี้ยงสี่ขาเห่าได้` → `จตุร, จัตวา, ๔` ✗ and `เครื่องมือสำหรับเขียนหนังสือ` →
`โกรกกราก, ไฮโกรมิเตอร์` ✗.

**The cause is structural, not a bug.** `สุนัข`'s gloss is *"สัตว์เลี้ยงลูกด้วยนมชนิดหนึ่ง เลี้ยงไว้เฝ้าบ้าน; หมา"* —
it contains neither `เห่า` nor `สี่ขา`, so BM25 over terse glosses cannot reach it. Meanwhile rare query
terms like `สี่` get high IDF and pull in words *defined as* "four".

**What to do — enrich the document, then damp the noise:**
1. **Index more text per entry:** the sense's **usage examples** (10,154 available and currently unindexed),
   the entry's synonyms/related words, and ศัพท์บัญญัติ English equivalents. A richer document is the
   single biggest lever here.
2. **Damp single-term dominance** so one rare query word cannot carry a hit on its own — require coverage
   of more of the query, or add a coordination/coverage factor to the BM25 score.
3. Re-run the **same four reviewer queries plus at least six new ones** and report the full result set,
   good and bad.

**Acceptance:** report before/after on all ten queries. **Do not curate the query set** — R8 reported this
failure honestly in an "Honest results (un-curated)" section and that was the right call; keep it. If the
acceptance query still fails after enrichment, say so and explain why.

## Q3 — S1b, third and final attempt

Two stops so far, both correct: R6 6.9× with 23 differential mismatches, R8 6.96× with 8,990 (two real
defects fixed en route, root cause isolated to *base-region overlap with dense ids*).

Worth one more attempt because the payoff is concrete: **day-of ingestion drops from 61 s to roughly 9 s**,
which is the difference between a tense pause and a comfortable live demo — and A3 measured the build as
**super-linear** (1× = 25 s, 10× = 2,493 s), so this also governs how much data we can ever add.

**Gates unchanged:** ≥3× cold build **and** byte-identical segmentation across the full vocabulary plus
the `PITCH.md` sentences. **Per G2: if it does not pass, delete `symbol_trie.rs` this round** and keep the
measurements in `BENCHMARKS.md`.

Note the interaction: if Q1 ships maximal matching, run the differential against the **new** segmentation
mode, not the old one.

---

# PHASE X — Optional: a fifth source (only if everything above is done)

## X1 — Investigate Wikidata Lexemes for Thai

Everything above leaves two data weaknesses that are **not** closable by code: the relation graph is
**~84 % Wiktionary-derived**, and only **0.68 %** of pairs are multi-source (which is what makes the 96 %
band-A precision apply to so little).

Wikidata Lexemes is **CC0**, bulk-downloadable, and requires scraping nobody — so unlike RID or
ศัพท์บัญญัติ it is a legitimate way to add an independent source. **But I have not verified Thai coverage,
so treat this as an investigation, not a commitment:** measure how many Thai lexemes and senses actually
exist, how many overlap our existing 158,045 pairs, and only integrate if it materially increases the
multi-source set.

**Stop rule:** if Thai coverage is thin, write the numbers into `VERIFY_R9.md` and stop. A measured
negative result closes the question properly.

**Do not** attempt to fix the 84 % figure by scraping ศัพท์บัญญัติ or RID at scale — that decision is
settled (`NEXT_STEPS_R8.md` §2) and unchanged.

---

## Order

```
G1 → G2 → D1 → D2 → Q1 → Q2 → Q3 → X1
```

- **G1 before everything** — it is 15 minutes and it guards the thing most likely to break during the rest
  of this round.
- **D1 before D2, and stop at D1 if the licence answer is unclear.** Publishing someone else's content
  publicly, days before their own competition, is not a risk worth taking for a demo convenience.
- **Q1 and Q2 are the highest-value quality work.** Q1 closes a number a NECTEC judge will know; Q2 closes
  the gap most visible to anyone who tries the product.
- **Droppable:** X1, and Q3 if the night runs short — but if Q3 is attempted and fails, G2's deletion rule
  applies in the same round.

**Deliverable:** `VERIFY_R9.md` in the established format (status table, raw script output, deviations,
stopped/skipped, known-stale claims), `BENCHMARKS.md` updated, dated `PROGRESS.md` entry.

**Standing instruction, now three rounds running:** if a task cannot be finished cleanly, stop it, commit
what is green, and write up why. Every time that rule has been applied it produced a better outcome than
pushing through would have.
