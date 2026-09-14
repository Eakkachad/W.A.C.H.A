#!/usr/bin/env python3
"""R10 Phase R1 — reshape the organizer's real RID excerpt to the block grammar
RidImporter already parses (wacha/tests/fixtures/rid/sample_entries.txt).

Source (git-ignored, ORST-educational, never committed raw):
    data/official/พจนานุกรม ฉบับราชบัณฑิตยสภาน/DICT_2554 (ก_ซ).xlsx
    sheet "หน้าหลัก", 13,395 rows.

Output (a small derived text projection, committed like kaikki_th.jsonl):
    data/rid/dict_2554.txt

Grammar emitted per entry block (blank line between blocks):
    HEADWORD: <headword> [<homograph Thai numeral, only if a real homograph>]
    READING:  [<read>]                      (omitted if read is NaN)
    ETYM:     (<etymology>)                  (omitted if empty)
    SUBENTRIES: w1, w2, ...                  (omitted if none)
    SENSE: [<pos>] (<subject_field>) <register> <definition>   (>=1 per entry)

Column semantics discovered by direct inspection of the real file:
  - `headword_sense` holds the homograph numeral (๑..๘) — a real homograph, not
    a plain sense counter (spot-checked on กก ๑/๒, genuinely distinct entries).
  - `pos` values already carry the trailing dot exactly as Pos::marker() prints
    (น. ก. ว. ส. บ. อ.) EXCEPT สัน. — the enum's marker is `สัน` (no dot). We emit
    the pos verbatim from the file; RidImporter maps it via Pos::from_marker,
    which we mirror below so สัน. is normalized to สัน (the one real mismatch).
  - `แม่คำ_0 ; ลูกคำ_1` is a parent-reference column:
        "0"          -> this row is a แม่คำ (head entry)
        "1"          -> ลูกคำ of the most recent preceding "0" row
        "<number>"   -> ลูกคำ whose parent is the row whose `number` == <number>
    We group ลูกคำ under their parent head row's SUBENTRIES.
"""
import sys, glob, math, re
import pandas as pd

SRC = glob.glob("data/official/พจนานุกรม*/DICT_2554*.xlsx")
if not SRC:
    sys.exit("ERROR: DICT_2554 xlsx not found under data/official/ — is data/official/ staged?")
SRC = SRC[0]
OUT = "data/rid/dict_2554.txt"

# Pos markers accepted by RidImporter::from_marker. Only สัน. differs (enum=สัน).
POS_NORMALIZE = {"สัน.": "สัน"}  # sole marker-vs-source mismatch, per NEXT_STEPS_R10


def clean(v):
    if v is None:
        return ""
    if isinstance(v, float) and math.isnan(v):
        return ""
    s = str(v).strip()
    if s.lower() == "nan":
        return ""
    # The source embeds presentation markup (italic scientific names, entities)
    # in definition text — strip it so it doesn't leak into CLI/JSON output.
    s = re.sub(r"</?[a-zA-Z][^>]*>", "", s)          # <i>...</i>, <sup>, etc.
    s = s.replace("&nbsp;", " ").replace("&amp;", "&")
    s = re.sub(r"\s+", " ", s).strip()
    return s


def main():
    df = pd.read_excel(SRC, sheet_name="หน้าหลัก", dtype=str)
    df = df.reset_index(drop=True)
    mk = "แม่คำ_0 ; ลูกคำ_1"

    # Pass 1: index rows by their `number` so a ลูกคำ can find its parent row.
    num_to_idx = {}
    for i, row in df.iterrows():
        n = clean(row.get("number"))
        if n:
            num_to_idx[n] = i

    # Pass 2: assign each ลูกคำ headword to its parent head-row index.
    subentries = {}  # parent_idx -> [child headword, ...]
    last_head_idx = None
    for i, row in df.iterrows():
        rel = clean(row.get(mk))
        hw = clean(row.get("headword"))
        if rel == "0" or rel == "":
            last_head_idx = i
            continue
        parent_idx = None
        if rel == "1":
            parent_idx = last_head_idx
        elif rel in num_to_idx:
            parent_idx = num_to_idx[rel]
        if parent_idx is not None and hw:
            subentries.setdefault(parent_idx, []).append(hw)

    # Pass 3: group consecutive rows sharing the same (headword, homograph) into
    # one entry with multiple SENSE lines (RID lists each sense as its own row).
    blocks = []
    i = 0
    n = len(df)
    while i < n:
        row = df.iloc[i]
        hw = clean(row.get("headword"))
        if not hw:
            i += 1
            continue
        homo = clean(row.get("headword_sense"))
        # gather all consecutive rows with same headword+homograph (multi-sense)
        senses = []
        etym = clean(row.get("etymology")) or clean(row.get("etymology.1"))
        reading = clean(row.get("read"))
        head_idx = i
        while i < n:
            r = df.iloc[i]
            if clean(r.get("headword")) != hw or clean(r.get("headword_sense")) != homo:
                break
            pos = clean(r.get("pos"))
            pos = POS_NORMALIZE.get(pos, pos)
            subj = clean(r.get("subject_field"))
            reg = clean(r.get("register"))
            definition = clean(r.get("definition"))
            if not definition:
                i += 1
                continue
            parts = []
            if pos:
                parts.append(f"[{pos}]")
            if subj:
                parts.append(f"({subj})")
            if reg:
                parts.append(f"{{{reg}}}")
            parts.append(definition)
            senses.append(" ".join(parts))
            if not etym:
                etym = clean(r.get("etymology")) or clean(r.get("etymology.1"))
            i += 1

        if not senses:
            continue

        lines = []
        head = f"HEADWORD: {hw}"
        if homo:
            head += f" {homo}"
        lines.append(head)
        if reading:
            lines.append(f"READING: [{reading}]")
        if etym:
            lines.append(f"ETYM: ({etym})")
        subs = subentries.get(head_idx)
        if subs:
            lines.append("SUBENTRIES: " + ", ".join(dict.fromkeys(subs)))  # dedupe, keep order
        for s in senses:
            lines.append(f"SENSE: {s}")
        blocks.append("\n".join(lines))

    import os
    os.makedirs("data/rid", exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write("\n\n".join(blocks) + "\n")

    n_sub = sum(1 for b in blocks if "SUBENTRIES:" in b)
    print(f"wrote {OUT}: {len(blocks)} entries ({n_sub} with SUBENTRIES) from {len(df)} rows")


if __name__ == "__main__":
    main()
