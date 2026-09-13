#!/usr/bin/env bash
# Fetch the Kaikki (Wiktionary) Thai dataset into data/kaikki_th.jsonl.
# Round 5, Task 3. The raw download is NOT committed (gitignored).
#
# Strategy: try the Thai-only file first (small, ~78 MB). It is marked DEPRECATED
# upstream and may 404 — if so, fall back to the all-languages raw dump and
# stream-filter to lang_code == "th".
set -euo pipefail
cd "$(dirname "$0")/.."           # -> wacha/
# The shared data dir the CLI reads via `--data ../data` is dict-hackathon/data.
DATA_DIR="../data"
mkdir -p "$DATA_DIR"
OUT="$DATA_DIR/kaikki_th.jsonl"
THAI_URL="https://kaikki.org/thwiktionary/ไทย/kaikki.org-dictionary-ไทย.jsonl"
RAW_URL="https://kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz"

if [ -f "$OUT" ]; then
  echo "already have $OUT ($(wc -l <"$OUT" | tr -d ' ') lines) — delete it to re-fetch"
  exit 0
fi

echo "trying Thai-only file…"
if curl -fsSL "$THAI_URL" -o "$OUT.tmp"; then
  mv "$OUT.tmp" "$OUT"
  echo "fetched Thai-only file: $(wc -l <"$OUT" | tr -d ' ') lines"
  exit 0
fi

echo "Thai-only file unavailable (likely 404 — it is deprecated). Falling back to raw dump…"
echo "streaming $RAW_URL and filtering lang_code == th (this downloads ~70 MB gz)…"
curl -fsSL "$RAW_URL" \
  | gunzip -c \
  | python3 -c '
import sys, json
kept = 0
for line in sys.stdin:
    try:
        o = json.loads(line)
    except Exception:
        continue
    if o.get("lang_code") == "th":
        sys.stdout.write(line)
        kept += 1
sys.stderr.write(f"kept {kept} Thai entries\n")
' > "$OUT.tmp"
mv "$OUT.tmp" "$OUT"
echo "fetched (filtered): $(wc -l <"$OUT" | tr -d ' ') lines"
