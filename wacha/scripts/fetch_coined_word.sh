#!/usr/bin/env bash
# fetch_coined_word.sh — polite, bounded fetch of a DEMO SUBSET of ศัพท์บัญญัติ
# (coined-word.orst.go.th) term equivalences, one English query at a time.
#
# HARD LIMITS (R5B / NEXT_STEPS_R5.md §2.1 — do not raise):
#   * demo subset only — the curated TERMS list below (well under 300)
#   * >= 500 ms between requests, single-threaded
#   * STOP on the first non-200 response (no retry loop)
#   * never re-fetch a term already cached in data/coined_word_cache/
#
# Usage:  cd wacha && bash scripts/fetch_coined_word.sh
# Writes: data/coined_word_cache/<query>.html  (one raw HTML fragment per query)
set -u
cd "$(dirname "$0")/.." || exit 1        # -> wacha/
CACHE="../data/coined_word_cache"
mkdir -p "$CACHE"
ENDPOINT="https://coined-word.orst.go.th/func_lookup.php"
DELAY=0.6                                 # >= 500 ms

# Curated demo English queries (book_id=0 = all disciplines). Chosen to show the
# many-to-many, discipline-disambiguated value: field (the §1.2 showcase),
# computer/IT, maths, law, linguistics, science.
TERMS=(
  field computer algorithm memory network data function variable
  set group ring matrix vector domain kernel field-effect
  language grammar phoneme morpheme syntax semantics
  law contract liability jurisdiction evidence
  energy force mass velocity acceleration field-strength
  cell tissue organ gene protein enzyme
)

fetched=0
skipped=0
for q in "${TERMS[@]}"; do
  out="$CACHE/$q.html"
  if [ -s "$out" ]; then
    skipped=$((skipped + 1))
    continue                              # never re-fetch a cached term
  fi
  status=$(curl -s -o "$out.tmp" -w "%{http_code}" --max-time 20 \
    -X POST "$ENDPOINT" \
    --data "word=${q}&funcName=lookupWord&book_id=0&status=lookup&loc=" 2>/dev/null)
  if [ "$status" != "200" ]; then
    echo "STOP: '$q' returned HTTP $status (non-200). Halting — no retry."
    rm -f "$out.tmp"
    break
  fi
  mv "$out.tmp" "$out"
  fetched=$((fetched + 1))
  echo "fetched $q ($(wc -c < "$out" | tr -d ' ') bytes)"
  sleep "$DELAY"
done

echo "---"
echo "fetched=$fetched  skipped(cached)=$skipped  cache_dir=$CACHE"
echo "cached files: $(ls "$CACHE"/*.html 2>/dev/null | wc -l | tr -d ' ')"
