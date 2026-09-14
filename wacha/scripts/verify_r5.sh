#!/usr/bin/env bash
# verify_r5.sh — prints EVERY number any Round 5 document claims, so a review is
# one command. Runnable from wacha/. No network access.
#
#   cd wacha && bash scripts/verify_r5.sh
#
# Rule (R5B): if a number is not printed by this script, it must not appear in
# any R5 document. Numbers here come straight from the built binary + tests.
set -u
cd "$(dirname "$0")/.." || exit 1   # -> wacha/
DATA="../data"
CLI="./target/release/wacha"

hr() { printf '\n=== %s ===\n' "$1"; }

hr "0. build (release)"
cargo build --release 2>&1 | tail -1

hr "1. cargo test --release (summary)"
# Full suite; print only the result lines so review is fast.
cargo test --release 2>&1 | grep -E "test result:" || echo "TESTS DID NOT REPORT — investigate"

hr "2. graph size + coverage (CLI stats)"
# entities / triples / sense nodes / raw + frequency-weighted coverage
"$CLI" --data "$DATA" stats 2>/dev/null

hr "3. cross-sense false-pair count + KEEP-recall / CUT-absence (CLI audit)"
# cross_sense_pairs must be 0; then the 47-pair audit summary (detail suppressed)
"$CLI" --data "$DATA" audit 2>/dev/null | grep -vE "^  \["

hr "4. cold vs warm engine start time"
# Cold = delete the trie cache and time a stats call (rebuilds the ~62k trie).
# Warm = the cache now exists; time a second stats call. Use bash's built-in
# TIMEFORMAT (portable across macOS/Linux; /usr/bin/time -p output differs).
CACHE="$DATA/words_th.datrie.cache"
rm -f "$CACHE"
TIMEFORMAT='%R'
COLD=$( { time "$CLI" --data "$DATA" stats >/dev/null 2>/dev/null; } 2>&1 )
WARM=$( { time "$CLI" --data "$DATA" stats >/dev/null 2>/dev/null; } 2>&1 )
echo "cold start (trie rebuilt from scratch): ${COLD}s"
echo "warm start (trie loaded from cache):    ${WARM}s"
unset TIMEFORMAT

hr "5. p95 warm lookup latency (20 seed + 20 random words, via HTTP server)"
# True warm latency = query an already-loaded in-process engine. The CLI reloads
# per call (~1.4s cache load), which measures start-up, not lookup — so we time
# the web server instead: build engine once, then time /api/lookup requests.
PORT=8097
lsof -ti:$PORT 2>/dev/null | xargs kill 2>/dev/null; sleep 0.3
./target/release/wacha-web --data "$DATA" --host 127.0.0.1 --port $PORT >/tmp/verify_web.log 2>&1 &
WEB_PID=$!
for _ in $(seq 1 120); do curl -s "http://127.0.0.1:$PORT/healthz" >/dev/null 2>&1 && break; sleep 0.5; done
python3 - "$PORT" "$DATA" <<'PY'
import urllib.parse, urllib.request, time, random, sys, statistics
port, data = sys.argv[1], sys.argv[2]
seed = ["ครู","บ้าน","แมว","สุนัข","หมา","ครอบครัว","รถยนต์","อาหาร","น้ำ","ดิน",
        "ไฟ","ลม","ต้นไม้","ดอกไม้","โรงเรียน","หนังสือ","เด็ก","ผู้ใหญ่","สวัสดี","ขอบคุณ"]
words = [w.strip() for w in open(f"{data}/words_th.txt", encoding="utf-8") if w.strip()]
random.seed(11); rand = random.sample(words, 20)
base = f"http://127.0.0.1:{port}/api/lookup?q="
for w in (seed + rand)[:5]:   # warm up
    urllib.request.urlopen(base + urllib.parse.quote(w), timeout=10).read()
lat = []
for w in seed + rand:
    t = time.perf_counter()
    urllib.request.urlopen(base + urllib.parse.quote(w), timeout=10).read()
    lat.append((time.perf_counter() - t) * 1000)
lat.sort()
p95 = lat[int(len(lat) * 0.95) - 1]
print(f"n={len(lat)} mean={statistics.mean(lat):.1f}ms p50={statistics.median(lat):.1f}ms "
      f"p95={p95:.1f}ms max={max(lat):.1f}ms  (warm, in-process engine)")
PY
kill $WEB_PID 2>/dev/null

hr "6. lookup output for the 6 review words"
for w in ครู บ้าน ครอบครัว รถยนต์ ปัญญาประดิษฐ์ สนาม; do
  echo "----- $w -----"
  "$CLI" --data "$DATA" lookup "$w" 2>/dev/null | grep -vE "^$" | head -16
done

hr "7. confirmation lines"
echo -n "katgpt-rs working tree status (empty = clean): "
git -C ../../katgpt-rs status --short 2>/dev/null | head -1
echo "(if the line above is blank, katgpt-rs is untouched)"
echo -n "personalized_pagerank occurrences in relations.rs: "
grep -c "personalized_pagerank" src/relations.rs

hr "done"

hr "8. pitch regression (T4) — PITCH.md §3 demo claims vs live engine"
bash "$(dirname "$0")/verify_pitch.sh" || echo "PITCH REGRESSION FAILED — fix PITCH.md §3"

hr "9. WASM smoke test — the offline flagship must still instantiate and answer"
# Why this exists: R7 verified the WASM build (W2), then S2 added Instant::now()
# to the engine build path, which panics on wasm32. Nothing re-checked WASM, so
# R7 shipped with the flagship trapping on init. This catches that class of bug.
# Node is enough for traps/panics/wrong output; it does NOT replace the
# real-browser + offline-toggle check on the pre-event human checklist.
WASM_DIR="$(cd "$(dirname "$0")/../../wacha-wasm" 2>/dev/null && pwd)"
if [ -z "$WASM_DIR" ] || [ ! -f "$WASM_DIR/smoke.mjs" ]; then
  echo "SKIP: wacha-wasm/smoke.mjs not found"
elif ! command -v node >/dev/null 2>&1; then
  echo "SKIP: node not installed (WASM smoke test not run)"
elif [ ! -f "$WASM_DIR/web/wacha_wasm.wasm" ]; then
  echo "SKIP: web/wacha_wasm.wasm not built — run 'cd wacha-wasm && bash web/build.sh'"
else
  node "$WASM_DIR/smoke.mjs" "$WASM_DIR/web/wacha_wasm.wasm" \
    || echo "WASM SMOKE FAILED — the offline flagship is broken, do not ship"
fi

hr "10. ranking-regression guard (G1) — the one hole in the safety net until R9"
# Why this exists: ranking is band -> PPR -> freq, empirically tuned, and it
# broke TWICE (R6: เรือน fell out; R7: tier order promoted the 55% tier) — both
# caught only by a human happening to look. This guards it automatically.
#
# Metric: automated p@5 over the committed 47-pair KEEP gold set — for every
# audited KEEP pair whose query has >=5 related, is the KEEP co-member in the
# query's TOP-5 ranked results? This is a RANKING metric (a genuine relation
# dropping out of the top is exactly the R6/R7 failure), computed with no human
# judgment, so it can gate CI. It is DISTINCT from the hand-audited precision@5
# in BENCHMARKS §4.1 (that one judges whether the returned top-5 are *good*, and
# needs a rater). Threshold 50%: the shipped band->PPR->freq scores 51.3%, while
# the two known-worse orderings score below it (freq-primary 48.7%, raw-tier
# 46.2%) — so a regression toward either trips this guard. Prints the number
# every run so slow drift is visible, not just a threshold breach.
"$CLI" --data "$DATA" rankguard 50 2>&1 | grep -vE "^loaded|^vocab_hash|^segmenter cache|^global PageRank" \
  || echo "RANK GUARD FAILED — ranking regressed below threshold, DO NOT SHIP"
