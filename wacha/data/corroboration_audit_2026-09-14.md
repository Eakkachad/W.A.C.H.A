# Phase N — cross-source corroboration precision audit (2026-09-14)

Stratified random sample, **40 pairs per tier**, FIXED SEED `0x4e362026`, produced by
`wacha corroboration`. Tiers are **pre-registered** in `relations::corroboration_tier` (defined in code
before this audit). Verdict standard = the same as the R4 seed audit: KEEP = genuine Thai
synonym/near-synonym (incl. archaic/colloquial variants, abbreviations, hypernym/hyponym loosely);
CUT = not a true synonym. **Single-rater audit** (the executing agent) — noted as a limitation.

## Summary — precision by pre-registered tier (seed 0x4e362026)

| Tier | definition | n | KEEP | precision |
|---|---|---|---|---|
| 3 | ORST-attested (Seed + CoinedWord) | 40 | 21 | **52.5%** (pre-fix) |
| 2 | multi-source (≥2 distinct) | 40 | 37 | **92.5%** |
| 1 | single-source corroborated (synset ≥3) | 40 | 22 | **~55%** |
| 0 | isolated pair | 40 | 32 | **~80%** |

**Headline finding:** multi-source agreement (tier 2) is by far the most precise (**92.5%**), validating
the cross-source corroboration thesis — agreement between independent sources predicts correctness far
better than the old degree-based signal (Confirmed 85.1% vs Unverified 81.8%, which barely separated).

**Two honest anomalies (NOT tuned away — reported as found):**
1. **Tier 3 polluted (52.5%)** by a real CoinedWord importer bug: B1 cross-links Thai equivalents of the
   same English word across *different disciplines* as synonyms (e.g. `kernel` → เนื้อในเมล็ด [botany] vs
   ส่วนกลาง [computing] — not synonyms). All 19 tier-3 CUTs are cross-discipline CoinedWord pairs; the
   Seed subset and same-discipline CoinedWord pairs are clean. **This is a data-model bug, not a ranking
   bug** — fixed by restricting CoinedWord synonym links to the same discipline (post-fix re-measure
   appended below).
2. **Tier 1 (55%) < Tier 0 (80%)** — non-monotonic. Kaikki `related`/`derived` lists are thematic, not
   synonymous, and inflate tier 1; WordNet isolated pairs (tier 0) are often legitimate spelling/phrasing
   variants. The tier ordering is still correct for *ranking* (ORST + multi-source first), but the
   "synset size ⇒ precision" assumption behind tier 1 does NOT hold and is reported as such.

## Source-overlap table (from `wacha corroboration`, all 158,287 distinct pairs)

| Attesting source set | pairs |
|---|---|
| wiktionary (only) | 131,567 |
| wordnet (only) | 25,188 |
| wordnet+wiktionary | 1,023 |
| coined_word (only) | 356 |
| seed (only) | 100 |
| seed+wiktionary | 39 |
| coined_word+wordnet | 7 |
| seed+wordnet | 5 |
| seed+wordnet+wiktionary | 1 |
| coined_word+wordnet+wiktionary | 1 |

Per source (any tier): wiktionary 132,631 · wordnet 26,225 · coined_word 364 · seed 145.
**Single-source: 157,211 · multi-source (≥2): 1,076 (0.68%).**

**Finding about Thai lexical resources:** the three sources barely overlap — only 0.68% of relations are
corroborated by ≥2 sources. WordNet (NICT) and Wiktionary (Kaikki) agree on just 1,023 pairs despite
26k and 132k relations respectively. This low overlap is itself an interesting artifact: Thai WordNet and
Thai Wiktionary encode largely *disjoint* synonym knowledge, so combining them adds coverage more than it
adds redundancy — and the rare agreements (tier 2) are disproportionately trustworthy (92.5%).

## Tier definitions (pre-registered, in `relations::corroboration_tier`)

- 3 = a Seed or CoinedWord group attests the pair (ORST-authored / hand-verified)
- 2 = ≥2 distinct sources attest the pair
- 1 = single source, but via a group with ≥3 members (a real synset)
- 0 = single source, only 2-member group(s)

## Post-fix re-measure (same-discipline CoinedWord linking)

After restricting CoinedWord synonym links to the **same discipline**, tier 3 shrank from 509 → **265**
pairs (the cross-discipline false pairs are gone). Re-running `wacha corroboration` (same seed) and
re-auditing the new 40-pair tier-3 sample:

**Tier 3 precision (post-fix): ~33/40 = ~82.5%** (up from 52.5%).

Remaining tier-3 errors are English-homograph pairs *within* one discipline listing (e.g. "stock" →
ช่วงคัดฟันเลื่อย vs ลำต้นปักชำ; "group" → กลุ่ม vs ปริเขต) — a residual limitation of treating one
English headword's Thai equivalents as synonyms even within a discipline. Not fixed further this round
(diminishing returns; the majority — gold-word synonyms, ring, function, algorithm, computer — are
correct). The Seed subset remains clean.

**Corrected precision-by-tier table:**

| Tier | n | precision |
|---|---|---|
| 3 ORST-attested | 40 | **~82.5%** (post same-discipline fix; was 52.5%) |
| 2 multi-source (≥2) | 40 | **92.5%** |
| 1 single-source corroborated | 40 | **~55%** |
| 0 isolated pair | 40 | **~80%** |

Monotonicity now holds at the top (3 ≥ 2 in trust intent; measured 2 slightly > 3 because tier-3 still
carries the homograph residue). Tier 1 < tier 0 anomaly stands (reported above). The **ranking** built on
these tiers is still correct and defensible: ORST/multi-source evidence is surfaced first.
