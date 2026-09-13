#!/usr/bin/env bash
# verify_pitch.sh (Round 7 T4) — a regression test for the PITCH.md demo.
#
# Stale demo material has bitten twice (รถยนต์→ยานยนต์ vanished in R5B; บ้าน's
# ranking degraded in R6; the ⚠ flag flipped in R7). Both earlier breaks were
# found only because a reviewer happened to run them. This asserts each demo
# claim in PITCH.md §3 against the LIVE engine and fails loudly with a diff.
#
# Each assertion is the demo's TEACHING POINT (a word is present with a given
# source / flag), NOT an exact rank — ranks are allowed to move, the story must
# still be true. Runnable from wacha/. No network.
#
#   cd wacha && bash scripts/verify_pitch.sh
set -u
cd "$(dirname "$0")/.." || exit 1
CLI="./target/release/wacha"
DATA="../data"
fail=0

# lookup <word> -> full CLI output cached in $OUT
run() { OUT="$("$CLI" --data "$DATA" lookup "$1" 2>/dev/null)"; }
seg() { SEG="$("$CLI" --data "$DATA" segment "$1" 2>/dev/null | sed 's/^segmented: //')"; }

# assert_related <query> <related-word> <expected-source-substr> <flag: warn|noflag>
assert_related() {
  local q="$1" w="$2" src="$3" flag="$4"
  run "$q"
  # the line for the related word (starts with "  N. <word>  ...")
  local line
  line="$(printf '%s\n' "$OUT" | grep -E "^[[:space:]]+[0-9]+\. ${w}([[:space:]]|$)" | head -1)"
  if [ -z "$line" ]; then
    echo "FAIL [$q]: related word '$w' not present"; fail=1; return
  fi
  if ! printf '%s' "$line" | grep -qF "$src"; then
    echo "FAIL [$q → $w]: expected source containing '$src', got: $(printf '%s' "$line" | sed 's/^ *//')"; fail=1; return
  fi
  local haswarn="noflag"
  printf '%s' "$line" | grep -q "⚠" && haswarn="warn"
  if [ "$haswarn" != "$flag" ]; then
    echo "FAIL [$q → $w]: expected flag '$flag', got '$haswarn' — line: $(printf '%s' "$line" | sed 's/^ *//')"; fail=1; return
  fi
  echo "ok   [$q → $w] src~'$src' flag=$flag"
}

assert_seg() {
  seg "$1"
  if [ "$SEG" != "$2" ]; then
    echo "FAIL seg [$1]: expected '$2', got '$SEG'"; fail=1; return
  fi
  echo "ok   seg [$1] = $SEG"
}

echo "=== PITCH.md §3 demo regression (verify 2026-09-14, R7) ==="
# คำที่ 1 — ครู: อาจารย์ is a hand-verified seed relation.
assert_related "ครู" "อาจารย์" "ตรวจแล้ว" "noflag"
# คำที่ 2 — สุนัข: หมา is a seed relation (both confirmed).
assert_related "สุนัข" "หมา" "ตรวจแล้ว" "noflag"
# คำที่ 3 — สนาม: เขตข้อมูล is an ORST ศัพท์บัญญัติ equivalent (closing beat).
assert_related "สนาม" "เขตข้อมูล" "ศัพท์บัญญัติ" "noflag"
# คำที่ 4 — ข้อหา: มลทิน is an isolated WordNet pair. Post-R7 it is CONFIRMED
# (isolated pairs measured 80%, no longer warned) — the flag-reversal story.
assert_related "ข้อหา" "มลทิน" "WordNet" "noflag"
# A genuine ⚠ (band C) demo word — the flag must fire on a single-source synset.
assert_related "วงศ์ตระกูล" "วงศ์วานว่านเครือ" "WordNet" "warn"
# โบนัส — segmentation demo.
assert_seg "เด็กน้อยรักสุนัข" "เด็กน้อย | รัก | สุนัข"

echo "---"
if [ "$fail" -eq 0 ]; then
  echo "PITCH regression: ALL PASS"
else
  echo "PITCH regression: FAILURES above — update PITCH.md §3 until this passes."
  exit 1
fi
