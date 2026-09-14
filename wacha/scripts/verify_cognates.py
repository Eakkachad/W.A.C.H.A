#!/usr/bin/env python3
"""R12 Phase BRIDGE-1 — the honesty gate.

Cross-verifies the 43 hand-crafted entries in the (git-ignored, reference-only)
`data/thaidict_script_ref/etymological-bridge/data/seed_corpus.py` against the
real Wiktionary etymology in `data/kaikki_th.jsonl` (`etymology_texts`, a LIST).

An entry is KEPT (corroborated) only if Kaikki's etymology for the same Thai word
independently mentions the same source language (บาลี/สันสกฤต or Pali/Sanskrit).
Entries with no Kaikki etymology, or a contradicting one, are DROPPED (recorded,
not silently kept at full confidence). This mirrors R9 D1 / R10 R3 / R11 VEC.

Output (committed, derived, corroborated-only):
    data/english_cognates.tsv
    thai_word \t pie_root \t pali_sanskrit_lang \t cognate1|cognate2|... \t kaikki_etymology

Run from repo root:  python3 wacha/scripts/verify_cognates.py
"""
import importlib.util, json, os, sys, re

SEED = "data/thaidict_script_ref/etymological-bridge/data/seed_corpus.py"
KAIKKI = "data/kaikki_th.jsonl"
OUT = "data/english_cognates.tsv"

if not os.path.exists(SEED):
    sys.exit(f"seed corpus not found: {SEED} (reference data staged?)")

spec = importlib.util.spec_from_file_location("seed", SEED)
seed = importlib.util.module_from_spec(spec)
spec.loader.exec_module(seed)
entries = {e["thai_word"]: e for e in seed.ENTRIES}

# Collect Kaikki etymology_texts (a LIST) per Thai word we care about.
want = set(entries)
kaikki_etym = {}
for line in open(KAIKKI, encoding="utf-8"):
    try:
        d = json.loads(line)
    except Exception:
        continue
    w = d.get("word")
    if w in want and w not in kaikki_etym:
        texts = d.get("etymology_texts") or []
        if isinstance(texts, str):
            texts = [texts]
        joined = " ".join(texts).strip()
        if joined:
            kaikki_etym[w] = joined

# Source-language markers that corroborate a Pali/Sanskrit claim.
PALI = ["บาลี", "pali", "pāli"]
SANS = ["สันสกฤต", "sanskrit", "saṃskṛta"]


def corroborates(seed_lang, kaikki_text):
    kt = kaikki_text.lower()
    seed_l = seed_lang.lower()
    seed_has_pali = "pali" in seed_l or "บาลี" in seed_lang
    seed_has_sans = "sanskrit" in seed_l or "สันสกฤต" in seed_lang
    k_pali = any(m in kt or m in kaikki_text for m in PALI)
    k_sans = any(m in kt or m in kaikki_text for m in SANS)
    # corroborated if the same family (Pali or Sanskrit) is mentioned in Kaikki
    return (seed_has_pali and k_pali) or (seed_has_sans and k_sans)


kept, dropped_no_etym, dropped_mismatch = [], [], []
for w, e in entries.items():
    kt = kaikki_etym.get(w)
    if not kt:
        dropped_no_etym.append(w)
        continue
    if corroborates(e.get("pali_sanskrit_lang", ""), kt):
        kept.append((w, e, kt))
    else:
        dropped_mismatch.append((w, kt))

with open(OUT, "w", encoding="utf-8") as f:
    for w, e, kt in kept:
        cognates = "|".join(c["word"] for c in e.get("english_cognates", []))
        pie = e.get("pie_root", "")
        lang = e.get("pali_sanskrit_lang", "")
        kt_clean = re.sub(r"\s+", " ", kt).replace("\t", " ")
        f.write(f"{w}\t{pie}\t{lang}\t{cognates}\t{kt_clean}\n")

n = len(entries)
print(f"=== BRIDGE-1 corroboration gate (of {n} seed entries) ===")
print(f"CORROBORATED (kept): {len(kept)} = {100*len(kept)/n:.1f}%")
print(f"DROPPED — no Kaikki etymology: {len(dropped_no_etym)}")
print(f"DROPPED — source-language mismatch: {len(dropped_mismatch)}")
print(f"wrote {OUT} with {len(kept)} corroborated entries")
print("kept:", [w for w, _, _ in kept])
print("dropped(no-etym):", dropped_no_etym)
if dropped_mismatch:
    print("dropped(mismatch):", [(w, kt[:40]) for w, kt in dropped_mismatch])
